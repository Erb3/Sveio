use crate::{datasource, packets, state, utils};
use chrono::Utc;
use geoutils::Location;
use regex::Regex;
use socketioxide::extract::{Data, SocketRef, State};
use socketioxide::SocketIo;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{debug, info};

pub struct GameOptions {
	pub datasource: datasource::Datasource,
}

pub fn on_connect(socket: SocketRef) {
	debug!("🆕 Client connected with client id {}", socket.id);

	socket.on(
		"join",
		|socket: SocketRef, Data::<packets::JoinMessage>(data), state: State<state::GameState>| async move {
			let username_regex = Regex::new(r"^[A-Za-z0-9 _-]{1,32}$").unwrap();

			if !username_regex.is_match(&data.username) {
				socket
					.emit(
						"kick",
						packets::DisconnectPacket {
							message: "Bad username".to_string(),
						},
					)
					.unwrap();

				socket.disconnect().unwrap();
				return;
			}

			if state.is_username_taken(data.username.clone()).await {
				socket
					.emit(
						"kick",
						packets::DisconnectPacket {
							message: "Username taken".to_string(),
						},
					)
					.unwrap();

				socket.disconnect().unwrap();
				return;
			}

			state
				.insert_player(socket.id, state::Player::new(data.username.clone()))
				.await;

			socket.emit("join-response", "").unwrap();
			socket.join("PRIMARY").unwrap();

			info!(
				"🪪  Client with ID {} set username to {}",
				socket.id, data.username
			);
		},
	);

	socket.on(
		"guess",
		|socket: SocketRef, Data::<packets::GuessPacket>(data), state: State<state::GameState>| async move {
			debug!("📬 Received message: {:?}", data);
			state.insert_guess(socket.id, data).await;
			state.update_last_packet(socket.id).await;
		},
	);

	socket.on_disconnect(|s: SocketRef, state: State<state::GameState>| async move {
		state.remove_player(s.id).await;
		debug!("🚪 User {} disconnected.", s.id);
	});
}

pub async fn game_loop(opts: GameOptions, io: Arc<SocketIo>, state: state::GameState) {
	let mut interval = time::interval(Duration::from_secs(5));
	let mut last_city: Option<datasource::City> = None;
	let mut index = 0;

	loop {
		interval.tick().await;

		if let Some(city) = last_city {
			let target = Location::new(city.latitude, city.longitude);

			for guess in state.get_guesses().await {
				let packet = guess.1;
				let distance =
					target.distance_to(&geoutils::Location::new(packet.lat, packet.long));
				let points = utils::calculate_score(distance.unwrap().meters() / 1000.0);

				if let Some(existing_player) = state.get_player(guess.0).await {
					let mut p = existing_player.to_owned();
					p.score += points;
					state.insert_player(guess.0, p).await;
				}
			}

			let solution = packets::SolutionPacket {
				location: city,
				guesses: state.get_guesses().await,
				leaderboard: state.get_players().await,
			};

			io.to("PRIMARY")
				.emit("solution", solution)
				.expect("Unable to broadcast solution");
		}

		interval.tick().await;

		let city: &datasource::City = opts.datasource.cities.get(index).unwrap();
		index += 1;
		if index == opts.datasource.cities.len() - 1 {
			index = 0;
		}

		debug!("📍 New location: {}, {}", &city.name, &city.country);
		state.clear_guesses().await;

		io.to("PRIMARY")
			.emit("newTarget", city.clone().anonymize())
			.expect("Unable to broadcast new target");

		last_city = Some(city.to_owned());

		for socket in io.sockets().unwrap() {
			if let Some(player) = state.get_player(socket.id).await {
				if Utc::now().timestamp_millis() > player.last_packet + 3 * 60 * 1000 {
					socket
						.emit(
							"kick",
							packets::DisconnectPacket {
								message: "Automatically removed due to inactivity".to_string(),
							},
						)
						.unwrap();
					socket.disconnect().unwrap();
				}
			}
		}
	}
}
