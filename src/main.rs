mod door_statemachine;
mod led;
mod state;
mod stepper;
mod tasks;
mod ws;

use axum::{Router, routing::get};
use rand::Rng;

use std::{
    error::Error,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

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
    let u_bool = stepper.get_running_clone();

    stepper.set_rpm(800);

    let state = Arc::new(AppState {
        led_stripe: led_stripe,
        led_repeat: t_bool,

        stepper_running: u_bool,
        stepper: Arc::new(Mutex::new(stepper)),

        tx,
    });

    tokio::spawn(status_update(state.clone()));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback(get(static_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 14444));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
