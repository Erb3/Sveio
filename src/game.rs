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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GuessMessage {
    lat: f32,
    long: f32,
}

pub type Guesses = HashMap<Sid, GuessMessage>;
type EncapsulatedGuesses = Arc<Mutex<Guesses>>;

#[derive(Serialize)]
struct Solution {
    location: datasource::City,
    guesses: Guesses,
}

pub fn on_connect(socket: SocketRef) {
    info!("🆕 Client connected with ID {}", socket.id);

    socket.on(
        "guess",
        |_socket: SocketRef, Data::<GuessMessage>(data), guesses: State<EncapsulatedGuesses>| {
            info!("Received message: {:?}", data);
            guesses.lock().unwrap().insert(_socket.id, data);
        },
    );
}

pub async fn game_loop(cities: Vec<datasource::City>, io: SocketIo, guesses: EncapsulatedGuesses) {
    let mut interval = time::interval(Duration::from_secs(5));
    let mut last_city: Option<&datasource::City> = None;

    loop {
        interval.tick().await;

        if let Some(city) = last_city.cloned() {
            let solution = Solution {
                location: city,
                guesses: guesses.lock().unwrap().clone(),
            };
            io.emit("solution", solution)
                .expect("Unable to broadcast solution");
        }

        interval.tick().await;

        let city: &datasource::City = cities.get(thread_rng().gen_range(0..cities.len())).unwrap();
        let anonymized_target = datasource::AnonymizedCity {
            name: &city.name,
            country: &city.country,
        };

        info!("New location: {}", city.clone().country);
        guesses.lock().unwrap().clear();
        io.emit("newTarget", anonymized_target)
            .expect("Unable to broadcast new target");

        last_city = Some(city);
    }
}
