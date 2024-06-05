use crate::packets::{self, GuessPacket};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use socketioxide::socket::Sid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct Username(String);

#[derive(Debug, Clone, Serialize)]
pub struct Player {
	pub username: Username,
	pub score: u64,

	#[serde(skip_serializing)]
	pub last_packet: i64,
}

impl Player {
	pub fn new(username: String) -> Player {
		return Player {
			username: Username(username),
			score: 0,
			last_packet: Utc::now().timestamp_millis(),
		};
	}
}

pub type Guesses = HashMap<Sid, packets::GuessPacket>;
pub type PlayerMap = HashMap<Sid, Player>;

#[derive(Clone)]
pub struct GameState {
	guesses: Arc<RwLock<Guesses>>,
	players: Arc<RwLock<PlayerMap>>,
}

impl GameState {
	pub fn new() -> GameState {
		return GameState {
			guesses: Arc::new(RwLock::new(Guesses::new())),
			players: Arc::new(RwLock::new(PlayerMap::new())),
		};
	}

	// Guesses

	pub async fn get_guesses(&self) -> Guesses {
		self.guesses.read().await.clone()
	}

	pub async fn clear_guesses(&self) {
		self.guesses.write().await.clear()
	}

	pub async fn insert_guess(&self, sid: Sid, guess: GuessPacket) {
		self.guesses.write().await.insert(sid, guess);
	}

	// Players

	pub async fn get_players(&self) -> PlayerMap {
		self.players.read().await.clone()
	}

	pub async fn insert_player(&self, sid: Sid, player: Player) {
		self.players.write().await.insert(sid, player);
	}

	pub async fn get_player(&self, sid: Sid) -> Option<Player> {
		match self.players.read().await.get(&sid) {
			Some(player) => Some(player.to_owned()),
			None => None,
		}
	}

	pub async fn remove_player(&self, sid: Sid) {
		self.players.write().await.remove(&sid);
	}

	pub async fn is_username_taken(&self, wanted: String) -> bool {
		self.get_players()
			.await
			.into_iter()
			.any(|v| v.1.username.0 == wanted)
	}

	pub async fn update_last_packet(&self, sid: Sid) {
		let player = self.get_player(sid).await;

		if let Some(mut p) = player {
			p.last_packet = Utc::now().timestamp_millis();
			self.players.write().await.insert(sid, p);
		}
	}
}
