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
    error::Error,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{spawn, task::spawn_blocking};

use crate::{
    led_stripe::{Frame, LED, LEDStripe, Sequenz},
    pins::RED_LED,
};

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
enum WorkMode {
    Static,
    Blink,
    Dot,
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
    led_stripe: Arc<Mutex<LEDStripe>>,
    left_running: Arc<AtomicBool>,
    right_running: Arc<AtomicBool>,
    led_repeat: Arc<AtomicBool>,
    led_thread_mutex: Arc<Mutex<()>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut led_stripe = LEDStripe::new(RED_LED, 150)?;
    // let led1 = LED::new(255, 0, 0);
    // let led2 = LED::new(0, 255, 0);
    // let frame = Frame(vec![led1, led2]);
    // let led1 = LED::new(255, 0, 0);
    // let led2 = LED::new(0, 255, 0);
    // let led3 = LED::new(0, 255, 0);
    // let led4 = LED::new(0, 255, 0);
    // let frame2 = Frame(vec![led2, led1, led3, led4]);
    // let seq = Sequenz::new(vec![frame, frame2], 1.0);
    // led_stripe.activate_sequenz(seq, Arc::new(AtomicBool::new(false)));
    // let seq = led_stripe.create_static((255, 0, 0));
    // led_stripe.activate_sequenz(seq, Arc::new(AtomicBool::new(false)));
    // let seq = led_stripe.create_static((0, 255, 0));
    // led_stripe.activate_sequenz(seq, Arc::new(AtomicBool::new(false)));
    // let seq = led_stripe.create_dot((0, 255, 0), 8.0, 0, 0);
    // led_stripe.activate_sequenz(seq, Arc::new(AtomicBool::new(true)));
    // return Ok(());

    let shared_state = Arc::new(AppState {
        led_stripe: Arc::new(Mutex::new(led_stripe)),
        left_running: Arc::new(AtomicBool::new(false)),
        right_running: Arc::new(AtomicBool::new(false)),
        led_repeat: Arc::new(AtomicBool::new(false)),
        led_thread_mutex: Arc::new(Mutex::new(())),
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
    state.led_repeat.store(payload.repeat, Ordering::SeqCst);

    spawn(async move || {
        println!("spawn thread");
        let _guard = led_thread_mutex_copy.lock().unwrap();
        eprintln!("start work");
        let mut stripe = led_stipe_copy.lock().unwrap();
        use WorkMode::*;
        let seq = match payload.mode {
            Static => stripe.create_static((payload.r, payload.g, payload.b)),
            Blink => stripe.create_blink((payload.r, payload.g, payload.b), payload.speed),
            Dot => stripe.create_dot((payload.r, payload.g, payload.b), payload.speed, 0, 0),
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
    println!("Left Start");
    "Left Start"
}
async fn right_start_handler(State(state): State<Arc<AppState>>) -> &'static str {
    if state.left_running.load(Ordering::SeqCst) || state.right_running.load(Ordering::SeqCst) {
        return "Motor läuft bereits";
    }
    state.right_running.store(true, Ordering::SeqCst);
    println!("Right Start");
    "Right Start"
}
async fn left_stop_handler(State(state): State<Arc<AppState>>) -> &'static str {
    if !state.left_running.load(Ordering::SeqCst) {
        return "Left is stopped";
    }
    state.left_running.store(false, Ordering::SeqCst);
    println!("Left Stop");
    "Left Stop"
}
async fn right_stop_handler(State(state): State<Arc<AppState>>) -> &'static str {
    if !state.right_running.load(Ordering::SeqCst) {
        return "Right is stopped";
    }
    state.right_running.store(false, Ordering::SeqCst);
    println!("Right Stop");
    "Right Stop"
}
