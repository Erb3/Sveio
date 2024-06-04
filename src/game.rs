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

#[derive(Serialize, Debug, Clone)]
pub struct Player {
	pub username: Username,
	pub score: u64,
}

pub type Guesses = HashMap<Sid, packets::GuessMessage>;
pub type PlayerMap = HashMap<Sid, Player>;

pub struct GameState {
	pub guesses: Guesses,
	pub leaderboard: PlayerMap,
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
							error: Some("Bad username".to_string()),
						},
					)
					.unwrap();

				socket.disconnect().unwrap();
				return;
			}

			if state
				.lock()
				.unwrap()
				.leaderboard
				.clone()
				.into_iter()
				.any(|v| v.1.username.0 == data.username)
			{
				socket
					.emit(
						"join-response",
						packets::JoinResponsePacket {
							ok: false,
							error: Some("Username taken".to_string()),
						},
					)
					.unwrap();

				socket.disconnect().unwrap();
				return;
			}

			state.lock().unwrap().leaderboard.insert(
				socket.id,
				Player {
					username: Username(data.username.clone()),
					score: 0,
				},
			);

			socket
				.emit(
					"join-response",
					packets::JoinResponsePacket {
						ok: true,
						error: None,
					},
				)
				.unwrap();

			socket.join("PRIMARY").unwrap();

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

			for guess in state.guesses.clone() {
				let packet = guess.1;
				let distance =
					target.distance_to(&geoutils::Location::new(packet.lat, packet.long));
				let points = utils::calculate_score(distance.unwrap().meters() / 1000.0);

				if let Some(existing_player) = state.leaderboard.get(&guess.0) {
					let mut p = existing_player.to_owned();
					p.score += points;
					state.leaderboard.insert(guess.0, p);
				}
			}

			let solution = packets::SolutionPacket {
				location: city,
				guesses: state.guesses.clone(),
				leaderboard: state.leaderboard.clone(),
			};

			io.to("PRIMARY")
				.emit("solution", solution)
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
		io.to("PRIMARY")
			.emit("newTarget", target_message)
			.expect("Unable to broadcast new target");

		last_city = Some(city);
	}
}
