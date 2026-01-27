use std::sync::{Arc, atomic::Ordering};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    response::IntoResponse,
};
use futures::SinkExt;
use futures::StreamExt;
use tokio::task::spawn_blocking;

use crate::{
    state::AppState,
    ws::messages::{ClientMsg, ServerMsg},
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Task: Server → Client
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let text = serde_json::to_string(&msg).unwrap();
            if sender
                .send(Message::Text(Utf8Bytes::from(text)))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Client → Server
    while let Some(Ok(Message::Text(text))) = receiver.next().await {
        if let Ok(cmd) = serde_json::from_str::<ClientMsg>(&text) {
            match cmd {
                ClientMsg::UpdateSettings {
                    r,
                    g,
                    b,
                    mode,
                    speed,
                    repeat,
                } => {
                    println!("Enter led settings");
                    let led_repeat_copy = state.led_repeat.clone();
                    let led_stipe_copy = state.led_stripe.clone();
                    let led_thread_mutex_copy = state.led_thread_mutex.clone();
                    spawn_blocking(move || {
                        // println!("spawn thread");
                        led_repeat_copy.store(false, Ordering::SeqCst);
                        let _guard = led_thread_mutex_copy.lock().unwrap();
                        // eprintln!("start work");
                        led_repeat_copy.store(repeat, Ordering::SeqCst);
                        let mut stripe = led_stipe_copy.lock().unwrap();
                        use crate::ws::messages::WorkMode::*;
                        let seq = match mode {
                            Static => {
                                led_repeat_copy.store(false, Ordering::SeqCst);
                                stripe.create_static((r, g, b))
                            }
                            Blink => stripe.create_blink((r, g, b), speed),
                            Dot => stripe.create_dot((r, g, b), speed, 0, 0),
                            Custom => {
                                led_repeat_copy.store(true, Ordering::SeqCst);
                                stripe.custom()
                            }
                        };
                        stripe.activate_sequenz(seq);
                        // println!("end work");
                    });
                    // println!("New LED Stripe Data: {:?}\n{:?}", mode, speed);
                }
                ClientMsg::RedAlert => {
                    red_alert(state.clone());
                }
                ClientMsg::LEDReset => {
                    state.led_repeat.store(false, Ordering::SeqCst);
                    state.led_stripe.lock().unwrap().reset();
                    println!("Reset LEDStripe");
                }
            }
        }
    }

    send_task.abort();
}
fn red_alert(state: Arc<AppState>) {
    println!("red alert!");
    let _ = state.tx.send(ServerMsg::PlaySound {
        name: "reset.mp3".into(),
    });
    let led_repeat_copy = state.led_repeat.clone();
    let led_stipe_copy = state.led_stripe.clone();
    let led_thread_mutex_copy = state.led_thread_mutex.clone();
    spawn_blocking(move || {
        led_repeat_copy.store(false, Ordering::SeqCst);
        let _guard = led_thread_mutex_copy.lock().unwrap();
        led_repeat_copy.store(true, Ordering::SeqCst);
        let mut t = led_stipe_copy.lock().unwrap();
        let s = t.red_alert();
        t.activate_sequenz(s)
    });
}
