mod datasource;
mod game;
mod packets;
mod state;
mod utils;
use axum::http::Method;
use dotenvy::dotenv;
use memory_serve::{load_assets, MemoryServe};
use socketioxide::SocketIoBuilder;
use state::GameState;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
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

	let socketio_state = GameState::new();

	let (socketio_layer, io) = SocketIoBuilder::new()
		.with_state(socketio_state.clone())
		.build_layer();

	io.ns("/", game::on_connect);

	let app = axum::Router::new()
		.nest_service(
			"/",
			MemoryServe::new(load_assets!("frontend"))
				.index_file(Some("/landing.html"))
				.add_alias("/game", "/game.html")
				.add_alias("/404", "/404.html")
				.fallback(Some("/404.html"))
				.html_cache_control(memory_serve::CacheControl::Medium)
				.into_router(),
		)
		.layer(
			ServiceBuilder::new()
				.layer(
					CorsLayer::new()
						.allow_methods([Method::GET])
						.allow_origin(Any),
				)
				.layer(socketio_layer),
		);

	info!("🎮 Starting game loop");

	tokio::spawn(async {
		game::game_loop(cities, io, socketio_state).await;
	});

	info!("⏳ Starting HTTP server");

	let listener = tokio::net::TcpListener::bind(format!(
		"0.0.0.0:{}",
		std::env::var("SVEIO_PORT").unwrap_or("8085".to_string())
	))
	.await
	.unwrap();

	info!("✅ Listening on http://{}", listener.local_addr().unwrap());
	axum::serve(listener, app).await.unwrap();
	Ok(())
}
