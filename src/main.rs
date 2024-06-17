#[cfg(not(feature = "shuttle"))]
mod cli;
mod datasource;
mod game;
mod packets;
mod server;
mod state;
mod utils;
use dotenvy::dotenv;
use tracing::info;

#[cfg(not(feature = "shuttle"))]
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

	server::create_server(server::ServerOptions {
		game: game::GameOptions { cities },
		port: Some(settings.port.unwrap_or(8085)),
	})
	.await;

	Ok(())
}

#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
	let _ = dotenv();

	info!("👋 Sveio says hi to Shuttle.rs!");
	info!("⏳ Loading cities!");
	let cities = datasource::get_cities().await;
	info!("✨ Loaded {} cities", cities.len());

	Ok(server::create_server(server::ServerOptions {
		game: game::GameOptions { cities },
		port: None,
	})
	.await
	.unwrap()
	.into())
}
