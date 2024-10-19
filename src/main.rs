mod cli;
mod datasource;
mod game;
mod packets;
mod server;
mod state;
mod utils;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let settings = cli::get_settings();

	tracing_subscriber::fmt()
		.with_max_level(settings.logging.unwrap_or(cli::LoggingLevel::Info))
		.init();

	info!("Sveio says hi!");

	server::start_server(server::ServerOptions {
		game: game::GameOptions {
			guess_time: settings.guess_time.unwrap_or(7),
			showcase_time: settings.showcase_time.unwrap_or(3),
		},
		port: settings.port.unwrap_or(8085),
		server_termination_kick: settings.termination_kick.unwrap_or(!cfg!(debug_assertions)),
	})
	.await;

	Ok(())
}
