use crate::{datasource, packets, utils};
use geoutils::Location;
use rand::{thread_rng, Rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use socketioxide::extract::{Data, SocketRef, State};
use socketioxide::socket::Sid;
use socketioxide::SocketIo;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;
use tracing::info;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct Username(String);

pub type Guesses = HashMap<Sid, packets::GuessMessage>;
pub type Leaderboard = HashMap<Sid, (Username, u64)>;
pub struct GameState {
	pub guesses: Guesses,
	pub leaderboard: Leaderboard,
}
type EncapsulatedGameState = Arc<Mutex<GameState>>;

pub fn on_connect(socket: SocketRef) {
	info!("🆕 Client connected with ID {}", socket.id);

	socket.on(
		"join",
		|socket: SocketRef,
		 Data::<packets::JoinMessage>(data),
		 state: State<EncapsulatedGameState>| {
			let username_regex = Regex::new(r"^[A-Za-z0-9 _-]{1,32}$").unwrap();

			if !username_regex.is_match(&data.username) {
				socket
					.emit(
						"join-response",
						packets::JoinResponsePacket {
							ok: false,
							error: Some("Bad username!".to_string()),
						},
					)
					.unwrap();

				socket.disconnect().unwrap();
				return;
			}

			state
				.lock()
				.unwrap()
				.leaderboard
				.insert(socket.id, (Username(data.username.clone()), 0));

			socket
				.emit(
					"join-response",
					packets::JoinResponsePacket {
						ok: true,
						error: None,
					},
				)
				.unwrap();
			info!(
				"🪪  Client with ID {} set username to {}",
				socket.id, data.username
			);
		},
	);

	socket.on(
		"guess",
		|socket: SocketRef,
		 Data::<packets::GuessMessage>(data),
		 game_state: State<EncapsulatedGameState>| {
			info!("📬 Received message: {:?}", data);
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

				if let Some(existing_player) = state.leaderboard.get(&player.0) {
					let p = existing_player.to_owned();
					state.leaderboard.insert(player.0, (p.0, p.1 + points));
				}
			}

			let solution = packets::SolutionPacket {
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
