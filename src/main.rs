mod datasource;
mod game;
mod utils;
use axum::handler::Handler;
use axum::http::{HeaderMap, Method};
use axum::response::IntoResponse;
use axum::routing::get;
use dotenvy::dotenv;
use socketioxide::SocketIoBuilder;
use std::fs;
use std::sync::{Arc, Mutex};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

fn read_file(path: &str) -> String {
    fs::read_to_string(path).expect("Should be able to read landing page into memory")
}

#[derive(Clone)]
struct AppState {
    landing_page_content: String,
    game_page_content: String,
    not_found_page_content: String,
}

impl AppState {
    fn get() -> AppState {
        AppState {
            landing_page_content: read_file("./frontend/landing.html"),
            game_page_content: read_file("./frontend/game.html"),
            not_found_page_content: read_file("./frontend/404.html"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;
    info!("👋 Sveio says hi!");
    info!("⚙️  Loading environment variables!");
    let _ = dotenv();

    info!("⏳ Loading cities!");
    let cities = datasource::get_cities();
    info!("✨ Loaded {} cities", cities.len());

    let socketio_state = Arc::new(Mutex::new(game::GameState {
        guesses: game::Guesses::new(),
        leaderboard: game::Leaderboard::new(),
    }));

    let (socketio_layer, io) = SocketIoBuilder::new()
        .with_state(Arc::clone(&socketio_state))
        .build_layer();

    io.ns("/", game::on_connect);

    let state = AppState::get();
    let app = axum::Router::new()
        .route("/", get(landing_page))
        .route("/game", get(game_page))
        .layer(socketio_layer)
        .nest_service(
            "/static/",
            ServeDir::new("frontend")
                .not_found_service(Handler::with_state(get(not_found_page), state.clone())),
        )
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(Any),
        )
        .with_state(state);

    info!("🎮 Starting game loop");

    tokio::spawn(async move {
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

async fn not_found_page(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/html; charset=utf-8".parse().unwrap());
    (headers, state.not_found_page_content)
}

async fn landing_page(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/html; charset=utf-8".parse().unwrap());
    (headers, state.landing_page_content)
}

async fn game_page(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "text/html; charset=utf-8".parse().unwrap());
    (headers, state.game_page_content)
}
