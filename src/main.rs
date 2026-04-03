mod door;
mod led;
mod state;
mod tasks;
mod ws;

use axum::{Router, routing::get};

use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

use crate::{
    door::{
        detector::{Detector, Target},
        door::{Door, start_door_controller},
        routes::door_routes,
    },
    led::stripe::Stripe,
    state::AppState,
    tasks::updater::status_update,
    ws::{handler::ws_handler, static_files::static_handler},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (ws_tx, _) = broadcast::channel(32);

    let led_stripe = Arc::new(Mutex::new(Stripe::new(150)));
    let t_bool = led_stripe.lock().unwrap().get_running_clone();

    let d = Door::new();
    let tx_door = start_door_controller(d);

    for uart in [3, 5] {
        #[allow(unused)]
        let tx_clone = tx_door.clone();
        let ws_tx_clone = ws_tx.clone();
        Detector::start(uart, move |arr: [Target; 3]| {
            for t in &arr {
                if t.is_alive() {
                    if t.is_door_open() {
                        let _ = tx_clone.send(door::door::Event::Open);
                    }
                }
            }
            let _ = ws_tx_clone.send(ws::messages::ServerMsg::Targets {
                id: uart,
                targets: arr,
            });
        });
    }

    let state = Arc::new(AppState {
        led_stripe: led_stripe,
        led_repeat: t_bool,

        door: tx_door,

        tx: ws_tx,
    });

    tokio::spawn(status_update(state.clone()));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .nest("/door", door_routes())
        .fallback(get(static_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:14444")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
