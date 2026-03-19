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
    #[allow(unused)]
    let tx_clone1 = tx_door.clone();
    let tx_clone2 = tx_door.clone();
    let ws_tx_clone1 = ws_tx.clone();
    let ws_tx_clone2 = ws_tx.clone();
    Detector::start(3, move |arr: &[Target]| {
        for t in arr {
            if t.is_alive() {
                if t.is_door_open() {
                    let _ = tx_clone1.send(door::door::Event::Open);
                }
            }
        }
        // dbg!(arr);
        let _ = ws_tx_clone1.send(ws::messages::ServerMsg::TargetPositions {
            id: 0,
            pos1: arr[0].get_point(),
            vec1: arr[0].get_vec(),
            done1: arr[0].is_door_open(),
            pos2: arr[1].get_point(),
            vec2: arr[1].get_vec(),
            done2: arr[1].is_door_open(),
            pos3: arr[2].get_point(),
            vec3: arr[2].get_vec(),
            done3: arr[2].is_door_open(),
        });
    });
    Detector::start(5, move |arr: &[Target]| {
        for t in arr {
            if t.is_alive() {
                if t.is_door_open() {
                    let _ = tx_clone2.send(door::door::Event::Open);
                }
            }
        }
        // dbg!(arr);
        let _ = ws_tx_clone2.send(ws::messages::ServerMsg::TargetPositions {
            id: 1,
            pos1: arr[0].get_point(),
            vec1: arr[0].get_vec(),
            done1: arr[0].is_door_open(),
            pos2: arr[1].get_point(),
            vec2: arr[1].get_vec(),
            done2: arr[1].is_door_open(),
            pos3: arr[2].get_point(),
            vec3: arr[2].get_vec(),
            done3: arr[2].is_door_open(),
        });
    });

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
