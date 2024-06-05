use crate::datasource;
use crate::state::{Guesses, PlayerMap};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GuessPacket {
	pub lat: f32,
	pub long: f32,
}

#[derive(Serialize)]
pub struct SolutionPacket {
	pub location: datasource::City,
	pub guesses: Guesses,
	pub leaderboard: PlayerMap,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JoinMessage {
	pub game: String,
	pub username: String,
}

#[derive(Serialize, Debug)]
pub struct DisconnectPacket {
	pub message: String,
}
