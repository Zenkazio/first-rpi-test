mod distance;
mod door_statemachine;
mod i2c;
mod keypad;
mod led_stripe;
mod pins;
mod rgb_swappper;
mod rgbled;
mod servo;
mod stepper;

use axum::{
    Json, Router,
    extract::State,
    response::Html,
    routing::{get, post},
};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

#[derive(Deserialize, Debug, Clone)]
struct Settings {
    r: u8,
    g: u8,
    b: u8,
    mode: String,
    speed: f32,
    repeat: bool,
}

struct AppState {
    current_settings: Mutex<Settings>,
}

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(AppState {
        current_settings: Mutex::new(Settings {
            r: 255,
            g: 0,
            b: 0,
            mode: "static".to_string(),
            speed: 1.0,
            repeat: false,
        }),
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/settings", post(settings_handler))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 14444));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../index.html"))
}

async fn settings_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Settings>,
) -> &'static str {
    let mut settings = state.current_settings.lock().unwrap();
    *settings = payload;

    println!("State aktualisiert: {:?}", *settings);
    "Einstellungen gespeichert!"
}
