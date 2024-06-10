mod cli;
mod datasource;
mod game;
mod packets;
mod server;
mod state;
mod utils;
use dotenvy::dotenv;
use server::ServerOptions;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing::subscriber::set_global_default(FmtSubscriber::default())?;
	info!("👋 Sveio says hi!");
	info!("⚙️  Loading environment variables!");
	let _ = dotenv();

	info!("⏳ Loading cities!");
	let cities = datasource::get_cities().await;
	info!("✨ Loaded {} cities", cities.len());
	let settings = cli::get_settings();

	server::start_server(ServerOptions {
		cities,
		port: settings.port.unwrap_or(8085),
	})
	.await;

	Ok(())
}
