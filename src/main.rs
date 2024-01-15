use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use socketioxide::extract::{Data, SocketRef};
use socketioxide::SocketIo;
use std::time::Duration;
use axum::http::Method;
use tokio::time;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Capital {
    country: String,
    #[serde(rename = "Capital City")]
    capital: String,
    latitude: f32,
    longitude: f32,
    population: u64,
}

#[derive(Debug, Serialize)]
struct AnonymizedCapital<'a> {
    country: &'a str,
    capital: &'a str,
}

#[derive(Debug, Deserialize)]
struct GuessMessage {
    lat: f32,
    long: f32,
}

fn on_connect(socket: SocketRef) {
    info!("🆕 Client connected with ID {}", socket.id);

    socket.on(
        "message",
        |_socket: SocketRef, Data::<GuessMessage>(data)| {
            info!("Received message: {:?}", data);
        },
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(FmtSubscriber::default())?;
    info!("👋 Sveio says hi!");
    info!("⏳ Loading capitals!");

    let mut capitals_csv =
        csv::Reader::from_path("./capitals.csv").expect("Unable to read and parse capitals");
    let mut capitals: Vec<Capital> = capitals_csv
        .deserialize()
        .into_iter()
        .map(|field| field.unwrap())
        .collect();

    info!("✨ Loaded {} capitals", capitals.len());

    let (socketio_layer, io) = SocketIo::new_layer();

    io.ns("/", on_connect);

    let app = axum::Router::new()
        .nest_service("/", ServeDir::new("frontend"))
        .layer(ServiceBuilder::new()
            .layer(CorsLayer::new().allow_methods([Method::GET]).allow_origin(Any))
            .layer(socketio_layer)
        );

    info!("Starting game loop");

    tokio::spawn(async move {
        game_loop(capitals, io).await;
    });

    info!("⏳ Starting HTTP server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn game_loop(capitals: Vec<Capital>, io: SocketIo) {
    let mut interval = time::interval(Duration::from_secs(5));
    let mut last_capital: Option<&Capital> = None;

    loop {
        info!("Letting players put their markers");
        interval.tick().await;
        info!("Time is up!");

        if last_capital.is_some() {
            io.emit("solution", last_capital.unwrap())
                .expect("Unable to broadcast solution");
        }

        info!("Announced old solution, taking a break...");
        interval.tick().await;

        info!("Announcing new target!");

        let mut capital: &Capital = capitals
            .get(thread_rng().gen_range(0..capitals.len()))
            .unwrap();
        let anonymized_target = AnonymizedCapital {
            capital: &capital.capital,
            country: &capital.country,
        };

        info!("New location: {}", capital.clone().country);
        io.emit("newTarget", anonymized_target)
            .expect("Unable to broadcast new target");

        last_capital = Some(capital);
    }
}
