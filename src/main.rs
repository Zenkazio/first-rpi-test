mod door_statemachine;
mod led;
mod state;
mod stepper;
mod tasks;
mod ws;

use axum::{Router, extract::State, routing::get};
use rand::Rng;

use std::{
    error::Error,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{sync::broadcast, task::spawn_blocking};

use crate::{
    led::stripe::Stripe,
    state::AppState,
    stepper::Stepper,
    tasks::updater::status_update,
    ws::{handler::ws_handler, static_files::static_handler},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, _) = broadcast::channel(32);

    let led_stripe = Arc::new(Mutex::new(Stripe::new(150)));
    let t_bool = led_stripe.lock().unwrap().get_running_clone();
    {
        let mut stripe = led_stripe.lock().unwrap();
        let f1 = stripe.strength(rand::rng().random(), (255, 0, 0));
        let f2 = stripe.strength(rand::rng().random(), (0, 255, 0));
        let f3 = stripe.strength(rand::rng().random(), (0, 0, 255));
        stripe.activate_frame(&f1.add(&f2).add(&f3));
    }
    let mut stepper = Stepper::new(17, 27, 22, 800)?;

    stepper.set_rpm(800);

    let state = Arc::new(AppState {
        led_stripe: led_stripe,
        left_running: Arc::new(AtomicBool::new(false)),
        right_running: Arc::new(AtomicBool::new(false)),
        led_repeat: t_bool,
        led_thread_mutex: Arc::new(Mutex::new(())),

        stepper: Arc::new(Mutex::new(stepper)),

        tx,
    });

    tokio::spawn(status_update(state.clone()));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/left/start", get(left_start_handler))
        .route("/left/stop", get(left_stop_handler))
        .route("/right/start", get(right_start_handler))
        .route("/right/stop", get(right_stop_handler))
        .fallback(get(static_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 14444));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
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
