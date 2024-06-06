mod datasource;
mod game;
mod packets;
mod state;
mod utils;
use axum::http::Method;
use dotenvy::dotenv;
use memory_serve::{load_assets, MemoryServe};
use socketioxide::{SocketIo, SocketIoBuilder};
use state::GameState;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
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
		)
		.layer(TimeoutLayer::new(Duration::from_secs(2)));

	info!("🎮 Starting game loop");

	let io_arc = Arc::new(io);
	let game_io = Arc::clone(&io_arc);
	let shutdown_io = Arc::clone(&io_arc);

	tokio::spawn(async move {
		game::game_loop(cities, game_io, socketio_state).await;
	});

	info!("⏳ Starting HTTP server");

	let listener = tokio::net::TcpListener::bind(format!(
		"0.0.0.0:{}",
		std::env::var("SVEIO_PORT").unwrap_or("8085".to_string())
	))
	.await
	.unwrap();

	info!("✅ Listening on http://{}", listener.local_addr().unwrap());
	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal(shutdown_io))
		.await
		.unwrap();
	Ok(())
}

async fn shutdown_signal(io: Arc<SocketIo>) {
	let ctrl_c = async {
		signal::ctrl_c()
			.await
			.expect("failed to install Ctrl+C handler");
	};

	#[cfg(unix)]
	let terminate = async {
		signal::unix::signal(signal::unix::SignalKind::terminate())
			.expect("failed to install signal handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
	let terminate = std::future::pending::<()>();

	tokio::select! {
		_ = ctrl_c => {},
		_ = terminate => {},
	}

	info!("Termination signal received, starting graceful shutdown. Exiting soon");
	for socket in io.sockets().unwrap() {
		socket
			.emit(
				"kick",
				packets::DisconnectPacket {
					message: "Server going down".to_string(),
				},
			)
			.unwrap();
		socket.disconnect().unwrap();
	}

	tokio::time::sleep(Duration::from_secs(if cfg!(debug_assertions) {
		0
	} else {
		5
	}))
	.await;
	info!("Exit imminent")
}
