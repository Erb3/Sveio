use crate::game::GameOptions;
use crate::packets;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use socketioxide::socket::Sid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(transparent)]
pub(crate) struct Username(String);

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Player {
	pub(crate) username: Username,
	pub(crate) score: u64,

	#[serde(skip_serializing)]
	pub(crate) last_packet: i64,
}

impl Player {
	pub(crate) fn new(username: String) -> Player {
		Player {
			username: Username(username),
			score: 0,
			last_packet: Utc::now().timestamp_millis(),
		}
	}
}

pub(crate) type GuessMap = HashMap<Sid, packets::GuessPacket>;
pub(crate) type PlayerMap = HashMap<Sid, Player>;

#[derive(Clone)]
pub(crate) struct GameState {
	guesses: Arc<RwLock<GuessMap>>,
	players: Arc<RwLock<PlayerMap>>,
	pub(crate) options: GameOptions,
}

impl GameState {
	pub(crate) fn new(options: GameOptions) -> GameState {
		GameState {
			guesses: Arc::new(RwLock::new(GuessMap::new())),
			players: Arc::new(RwLock::new(PlayerMap::new())),
			options,
		}
	}

	// Guesses

	pub(crate) async fn get_guesses(&self) -> GuessMap {
		self.guesses.read().await.clone()
	}

	pub(crate) async fn clear_guesses(&self) {
		self.guesses.write().await.clear()
	}

	pub(crate) async fn insert_guess(&self, sid: Sid, guess: packets::GuessPacket) {
		self.guesses.write().await.insert(sid, guess);
	}

	// Players

	pub(crate) async fn get_players(&self) -> PlayerMap {
		self.players.read().await.clone()
	}

	pub(crate) async fn insert_player(&self, sid: Sid, player: Player) {
		self.players.write().await.insert(sid, player);
	}

	pub(crate) async fn get_player(&self, sid: Sid) -> Option<Player> {
		self.players
			.read()
			.await
			.get(&sid)
			.map(|player| player.to_owned())
	}

	pub(crate) async fn remove_player(&self, sid: Sid) {
		self.players.write().await.remove(&sid);
	}

	pub(crate) async fn is_username_taken(&self, wanted: String) -> bool {
		self.get_players()
			.await
			.into_iter()
			.any(|v| v.1.username.0 == wanted)
	}

	pub(crate) async fn update_last_packet(&self, sid: Sid) {
		let player = self.get_player(sid).await;

		if let Some(mut p) = player {
			p.last_packet = Utc::now().timestamp_millis();
			self.insert_player(sid, p).await;
		}
	}
}
