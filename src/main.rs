mod datasource;
use axum::http::{HeaderMap, Method};
use axum::response::IntoResponse;
use axum::routing::get;
use dotenvy::dotenv;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use socketioxide::extract::{Data, SocketRef, State};
use socketioxide::socket::Sid;
use socketioxide::{SocketIo, SocketIoBuilder};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Deserialize, Serialize, Clone)]
struct GuessMessage {
    lat: f32,
    long: f32,
}

type Guesses = HashMap<Sid, GuessMessage>;
type EncapsulatedGuesses = Arc<Mutex<Guesses>>;

#[derive(Serialize)]
struct Solution {
    location: datasource::City,
    guesses: Guesses,
}

fn on_connect(socket: SocketRef) {
    info!("🆕 Client connected with ID {}", socket.id);

    socket.on(
        "guess",
        |_socket: SocketRef, Data::<GuessMessage>(data), guesses: State<EncapsulatedGuesses>| {
            info!("Received message: {:?}", data);
            guesses.lock().unwrap().insert(_socket.id, data);
        },
    );
}

fn read_file(path: &str) -> String {
    fs::read_to_string(path).expect("Should be able to read landing page into memory")
}

#[derive(Clone)]
struct AppState {
    landing_page_content: String,
    game_page_content: String,
}

impl AppState {
    fn get() -> AppState {
        AppState {
            landing_page_content: read_file("./frontend/landing.html"),
            game_page_content: read_file("./frontend/game.html"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;
    info!("👋 Sveio says hi!");
    info!("👋 Loading environment variables!");
    let _ = dotenv();

    info!("⏳ Loading cities!");
    let cities = datasource::get_cities();
    info!("✨ Loaded {} cities", cities.len());

    let socketio_state = Arc::new(Mutex::new(Guesses::new()));
    let (socketio_layer, io) = SocketIoBuilder::new()
        .with_state(Arc::clone(&socketio_state))
        .build_layer();

    io.ns("/", on_connect);

    let app = axum::Router::new()
        .route("/", get(landing_page))
        .route("/game", get(game_page))
        .with_state(AppState::get())
        .nest_service("/static/", ServeDir::new("frontend"))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET])
                .allow_origin(Any),
        )
        .layer(socketio_layer);

    info!("🎮 Starting game loop");

    tokio::spawn(async move {
        game_loop(cities, io, socketio_state).await;
    });

    info!("⏳ Starting HTTP server");

    let listener = tokio::net::TcpListener::bind(format!(
        "0.0.0.0:{}",
        std::env::var("SVEIO_PORT").unwrap_or("8085".to_string())
    ))
    .await
    .unwrap();

    info!("✅ Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    Ok(())
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

async fn game_loop(cities: Vec<datasource::City>, io: SocketIo, guesses: EncapsulatedGuesses) {
    let mut interval = time::interval(Duration::from_secs(5));
    let mut last_city: Option<&datasource::City> = None;

    loop {
        interval.tick().await;

        if let Some(city) = last_city.cloned() {
            let solution = Solution {
                location: city,
                guesses: guesses.lock().unwrap().clone(),
            };
            io.emit("solution", solution)
                .expect("Unable to broadcast solution");
        }

        interval.tick().await;

        let city: &datasource::City = cities.get(thread_rng().gen_range(0..cities.len())).unwrap();
        let anonymized_target = datasource::AnonymizedCity {
            name: &city.name,
            country: &city.country,
        };

        info!("New location: {}", city.clone().country);
        guesses.lock().unwrap().clear();
        io.emit("newTarget", anonymized_target)
            .expect("Unable to broadcast new target");

        last_city = Some(city);
    }
}
