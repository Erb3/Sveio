#[cfg(not(feature = "shuttle"))]
mod cli;
mod datasource;
mod game;
mod packets;
mod server;
mod state;
mod utils;
use tracing::info;

#[cfg(not(feature = "shuttle"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let settings = cli::get_settings();

	tracing_subscriber::fmt()
		.with_max_level(settings.logging.unwrap_or(cli::LoggingLevel::Info))
		.init();

	info!("👋 Sveio says hi!");

	server::create_server(server::ServerOptions {
		game: game::GameOptions {
			guess_time: settings.guess_time.unwrap_or(7),
			showcase_time: settings.showcase_time.unwrap_or(3),
		},
		port: Some(settings.port.unwrap_or(8085)),
		server_termination_kick: settings.termination_kick.unwrap_or(!cfg!(debug_assertions)),
	})
	.await;

	Ok(())
}

#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
	info!("👋 Sveio says hi to Shuttle.rs!");

	Ok(server::create_server(server::ServerOptions {
		game: game::GameOptions {
			guess_time: 7,
			showcase_time: 3,
		},
		port: None,
		server_termination_kick: true,
	})
	.await
	.unwrap()
	.into())
}
