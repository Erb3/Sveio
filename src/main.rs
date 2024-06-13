mod cli;
mod datasource;
mod game;
mod packets;
mod server;
mod state;
mod utils;
use dotenvy::dotenv;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let _ = dotenv();
	let settings = cli::get_settings();

	tracing_subscriber::fmt()
		.with_max_level(settings.logging.unwrap_or(cli::LoggingLevel::Info))
		.init();

	info!("👋 Sveio says hi!");
	info!("⏳ Loading cities!");
	let cities = datasource::get_cities().await;
	info!("✨ Loaded {} cities", cities.len());

	server::start_server(server::ServerOptions {
		game: game::GameOptions { cities },
		port: settings.port.unwrap_or(8085),
	})
	.await;

	Ok(())
}
