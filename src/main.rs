mod distance;
mod door_statemachine;
mod i2c;
mod keypad;
mod led;
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
    error::Error,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::task::spawn_blocking;

use crate::{led::stripe::Stripe, stepper::Stepper};

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
enum WorkMode {
    Static,
    Blink,
    Dot,
    Custom,
}

#[derive(Deserialize, Debug, Clone)]
struct Settings {
    r: u8,
    g: u8,
    b: u8,
    mode: WorkMode,
    speed: f32,
    repeat: bool,
}

struct AppState {
    led_stripe: Arc<Mutex<Stripe>>,
    left_running: Arc<AtomicBool>,
    right_running: Arc<AtomicBool>,
    led_repeat: Arc<AtomicBool>,
    led_thread_mutex: Arc<Mutex<()>>,

    stepper: Arc<Mutex<Stepper>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let t_bool = Arc::new(AtomicBool::new(true));
    let mut led_stripe = Stripe::new(150);

    led_stripe.activate_sequenz(led_stripe.red_alert(), t_bool.clone());

    let mut stepper = Stepper::new(17, 27, 22, 200)?;
    stepper.set_rpm(800);

    // stepper.turn_left(Arc::new(AtomicBool::new(true)));

    // return Ok(());

    let shared_state = Arc::new(AppState {
        led_stripe: Arc::new(Mutex::new(led_stripe)),
        left_running: Arc::new(AtomicBool::new(false)),
        right_running: Arc::new(AtomicBool::new(false)),
        led_repeat: t_bool,
        led_thread_mutex: Arc::new(Mutex::new(())),

        stepper: Arc::new(Mutex::new(stepper)),
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/led/reset", get(led_reset_handler))
        .route("/led/settings", post(led_settings_handler))
        .route("/left/start", get(left_start_handler))
        .route("/left/stop", get(left_stop_handler))
        .route("/right/start", get(right_start_handler))
        .route("/right/stop", get(right_stop_handler))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 14444));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../index.html"))
}

async fn led_settings_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Settings>,
) -> &'static str {
    println!("Enter led settings");
    let led_repeat_copy = state.led_repeat.clone();
    let led_stipe_copy = state.led_stripe.clone();
    let led_thread_mutex_copy = state.led_thread_mutex.clone();
    spawn_blocking(move || {
        println!("spawn thread");
        led_repeat_copy.store(false, Ordering::SeqCst);
        let _guard = led_thread_mutex_copy.lock().unwrap();
        eprintln!("start work");
        led_repeat_copy.store(payload.repeat, Ordering::SeqCst);
        let mut stripe = led_stipe_copy.lock().unwrap();
        use WorkMode::*;
        let seq = match payload.mode {
            Static => {
                led_repeat_copy.store(false, Ordering::SeqCst);
                stripe.create_static((payload.r, payload.g, payload.b))
            }
            Blink => stripe.create_blink((payload.r, payload.g, payload.b), payload.speed),
            Dot => stripe.create_dot((payload.r, payload.g, payload.b), payload.speed, 0, 0),
            Custom => stripe.custom(),
        };
        stripe.activate_sequenz(seq, led_repeat_copy);
        println!("end work");
    });
    println!("New LED Stripe Data: {:?}", payload);
    "Einstellungen gespeichert!"
}
async fn led_reset_handler(State(state): State<Arc<AppState>>) -> &'static str {
    state.led_repeat.store(false, Ordering::SeqCst);
    state.led_stripe.lock().unwrap().reset();
    println!("Reset LEDStripe");
    "Led Resetted!"
}

async fn left_start_handler(State(state): State<Arc<AppState>>) -> &'static str {
    if state.left_running.load(Ordering::SeqCst) || state.right_running.load(Ordering::SeqCst) {
        return "Motor läuft bereits";
    }
    state.left_running.store(true, Ordering::SeqCst);
    let stepper_copy = state.stepper.clone();
    let left_running_copy = state.left_running.clone();
    spawn_blocking(move || {
        stepper_copy.lock().unwrap().turn_left(left_running_copy);
    });
    println!("Left Start");
    "Left Start"
}
async fn right_start_handler(State(state): State<Arc<AppState>>) -> &'static str {
    if state.left_running.load(Ordering::SeqCst) || state.right_running.load(Ordering::SeqCst) {
        return "Motor läuft bereits";
    }
    let stepper_copy = state.stepper.clone();
    let right_running_copy = state.right_running.clone();
    spawn_blocking(move || {
        stepper_copy.lock().unwrap().turn_right(right_running_copy);
    });
    state.right_running.store(true, Ordering::SeqCst);
    println!("Right Start");
    "Right Start"
}
async fn left_stop_handler(State(state): State<Arc<AppState>>) -> &'static str {
    if !state.left_running.load(Ordering::SeqCst) {
        return "Left is stopped";
    }
    state.left_running.store(false, Ordering::SeqCst);
    state.stepper.lock().unwrap().clear();
    println!("Left Stop");
    "Left Stop"
}
async fn right_stop_handler(State(state): State<Arc<AppState>>) -> &'static str {
    if !state.right_running.load(Ordering::SeqCst) {
        return "Right is stopped";
    }
    state.right_running.store(false, Ordering::SeqCst);
    state.stepper.lock().unwrap().clear();
    println!("Right Stop");
    "Right Stop"
}
