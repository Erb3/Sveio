use crate::{game, packets, state};
use axum::http::Method;
use memory_serve::{load_assets, MemoryServe};
use socketioxide::{SocketIo, SocketIoBuilder};
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tracing::info;

pub struct ServerOptions {
	pub game: game::GameOptions,
	pub port: Option<u32>,
}

pub async fn create_server(opts: ServerOptions) -> Option<axum::Router> {
	let socketio_state = state::GameState::new(opts.game.clone());

	let (socketio_layer, io) = SocketIoBuilder::new()
		.with_state(socketio_state.clone())
		.build_layer();

	io.ns("/", game::on_connect);

	let app = axum::Router::new()
		.nest_service(
			"/",
			MemoryServe::new(load_assets!("frontend"))
				.add_alias("/favicon.ico", "/icons/favicon.ico")
				.add_alias("/site.webmanifest", "/icons/site.webmanifest")
				.index_file(Some("/landing.html"))
				.fallback(Some("/404.html"))
				.html_cache_control(memory_serve::CacheControl::Medium)
				.enable_clean_url(true)
				.enable_brotli(true)
				.enable_gzip(true)
				.into_router(),
		)
		.layer(
			ServiceBuilder::new()
				.layer(
					CorsLayer::new()
						.allow_methods([Method::GET])
						.allow_origin(tower_http::cors::Any),
				)
				.layer(socketio_layer),
		)
		.layer(TimeoutLayer::new(Duration::from_secs(2)));

	info!("🎮 Starting game loop");

	let io_arc = Arc::new(io);
	let game_io = Arc::clone(&io_arc);
	let shutdown_io = Arc::clone(&io_arc);

	tokio::spawn(async move {
		game::game_loop(opts.game, game_io, socketio_state).await;
	});

	if let Some(port) = opts.port {
		info!("⏳ Starting HTTP server");
		let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
			.await
			.unwrap();

		info!("✅ Listening on http://{}", listener.local_addr().unwrap());
		axum::serve(listener, app)
			.with_graceful_shutdown(shutdown_signal(shutdown_io))
			.await
			.unwrap();

		return None;
	}

	Some(app)
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
