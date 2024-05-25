use geoutils::Location;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use socketioxide::extract::{Data, SocketRef, State};
use socketioxide::socket::Sid;
use socketioxide::SocketIo;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;
use tracing::info;

use crate::datasource;
use crate::utils;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GuessMessage {
    lat: f32,
    long: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct JoinMessage {
    game: String,
    username: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct Username(String);

pub type Guesses = HashMap<Sid, GuessMessage>;
pub type Leaderboard = HashMap<Sid, (Username, u64)>;
pub struct GameState {
    pub guesses: Guesses,
    pub leaderboard: Leaderboard,
}
type EncapsulatedGameState = Arc<Mutex<GameState>>;

#[derive(Serialize)]
struct SolutionPacket {
    location: datasource::City,
    guesses: Guesses,
    leaderboard: Leaderboard,
}

pub fn on_connect(socket: SocketRef) {
    info!("🆕 Client connected with ID {}", socket.id);

    socket.on(
        "join",
        |socket: SocketRef, Data::<JoinMessage>(data), state: State<EncapsulatedGameState>| {
            state
                .lock()
                .unwrap()
                .leaderboard
                .insert(socket.id, (Username(data.username.clone()), 0));

            socket.emit("join-response", "{\"ok\": true}").unwrap();
            info!(
                "🪪  Client with ID {} set username to {}",
                socket.id, data.username
            );
        },
    );

    socket.on(
        "guess",
        |socket: SocketRef,
         Data::<GuessMessage>(data),
         game_state: State<EncapsulatedGameState>| {
            info!("Received message: {:?}", data);
            game_state.lock().unwrap().guesses.insert(socket.id, data);
        },
    );

    socket.on_disconnect(|s: SocketRef, state: State<EncapsulatedGameState>| {
        state.lock().unwrap().leaderboard.remove(&s.id);
        info!("🚪 User {} disconnected.", s.id);
    });
}

pub async fn game_loop(
    cities: Vec<datasource::City>,
    io: SocketIo,
    game_state: EncapsulatedGameState,
) {
    let mut interval = time::interval(Duration::from_secs(5));
    let mut last_city: Option<&datasource::City> = None;

    loop {
        interval.tick().await;

        if let Some(city) = last_city.cloned() {
            let mut state = game_state.lock().unwrap();
            let target = Location::new(city.latitude, city.longitude);

            for player in state.guesses.clone() {
                let packet = player.1;
                let distance =
                    target.distance_to(&geoutils::Location::new(packet.lat, packet.long));
                let points = utils::calculate_score(distance.unwrap().meters() / 1000.0);
                let existing_player = state.leaderboard.get(&player.0).unwrap().to_owned();
                state
                    .leaderboard
                    .insert(player.0, (existing_player.0, existing_player.1 + points));
            }

            let solution = SolutionPacket {
                location: city,
                guesses: state.guesses.clone(),
                leaderboard: state.leaderboard.clone(),
            };

            io.emit("solution", solution)
                .expect("Unable to broadcast solution");
        }

        interval.tick().await;

        let city: &datasource::City = cities.get(thread_rng().gen_range(0..cities.len())).unwrap();
        let target_message = datasource::AnonymizedCity {
            name: &city.name,
            country: &city.country,
        };

        info!("📍 New location: {}, {}", &city.name, &city.country);
        game_state.lock().unwrap().guesses.clear();
        io.emit("newTarget", target_message)
            .expect("Unable to broadcast new target");

        last_city = Some(city);
    }
}
