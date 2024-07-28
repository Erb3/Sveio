use crate::{datasource, state};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct GuessPacket {
	pub(crate) lat: f32,
	pub(crate) long: f32,
}

#[derive(Serialize)]
pub(crate) struct SolutionPacket {
	pub(crate) location: datasource::City,
	pub(crate) guesses: state::GuessMap,
	pub(crate) leaderboard: state::PlayerMap,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct JoinMessage {
	pub(crate) game: String,
	pub(crate) username: String,
}

#[derive(Serialize, Debug)]
pub(crate) struct GameMetadataMessage {
	pub(crate) guess_time: u64,
	pub(crate) showcase_time: u64,
}

#[derive(Serialize, Debug)]
pub(crate) struct DisconnectPacket {
	pub(crate) message: String,
}
