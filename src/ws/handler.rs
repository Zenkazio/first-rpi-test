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
    led::{frame::Frame, led::LED},
    state::AppState,
    ws::messages::{ClientMsg, PlayerColors, ServerMsg, WorkMode},
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
            println!("ClientMsg: {:?}", &cmd);
            match cmd {
                ClientMsg::UpdateSettings {
                    r,
                    g,
                    b,
                    mode,
                    speed,
                    repeat,
                } => {
                    set_led_settings(r, g, b, mode, speed, repeat, state.clone());
                }
                ClientMsg::RedAlert => {
                    red_alert(state.clone());
                }
                ClientMsg::LEDReset => {
                    led_reset(state.clone());
                }
                ClientMsg::LeftStart => start_stepper(state.clone(), true),
                ClientMsg::RightStart => start_stepper(state.clone(), false),
                ClientMsg::StepperStop => stop_stepper(state.clone()),
                ClientMsg::PlayerTable { p1, p2, p3 } => {
                    playertable(p1, p2, p3, state.clone());
                }
            }
        }
    }

    send_task.abort();
}
fn set_led_settings(
    r: u8,
    g: u8,
    b: u8,
    mode: WorkMode,
    speed: f32,
    repeat: bool,
    state: Arc<AppState>,
) {
    let led_repeat_copy = state.led_repeat.clone();
    led_repeat_copy.store(false, Ordering::SeqCst);
    let led_stipe_copy = state.led_stripe.clone();
    spawn_blocking(move || {
        let mut stripe = led_stipe_copy.lock().unwrap();
        led_repeat_copy.store(repeat, Ordering::SeqCst);
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
    });
}
fn red_alert(state: Arc<AppState>) {
    let _ = state.tx.send(ServerMsg::PlaySound {
        name: "reset.mp3".into(),
    });
    let led_repeat_copy = state.led_repeat.clone();
    let led_stipe_copy = state.led_stripe.clone();
    led_repeat_copy.store(false, Ordering::SeqCst);
    spawn_blocking(move || {
        let mut t = led_stipe_copy.lock().unwrap();
        led_repeat_copy.store(true, Ordering::SeqCst);
        let s = t.red_alert();
        t.activate_sequenz(s)
    });
}
fn start_stepper(state: Arc<AppState>, left: bool) {
    if state.stepper_running.load(Ordering::SeqCst) {
        return;
    }
    state.stepper_running.store(true, Ordering::SeqCst);
    let stepper_copy = state.stepper.clone();
    if left {
        spawn_blocking(move || {
            stepper_copy.lock().unwrap().turn_left();
        });
    } else {
        spawn_blocking(move || {
            stepper_copy.lock().unwrap().turn_right();
        });
    }
}
fn stop_stepper(state: Arc<AppState>) {
    state.stepper_running.store(false, Ordering::SeqCst);
    state.stepper.lock().unwrap().clear();
}
fn led_reset(state: Arc<AppState>) {
    state.led_repeat.store(false, Ordering::SeqCst); // darf nicht anders gemacht werden!! der stripe lock greift sonst nicht
    state.led_stripe.lock().unwrap().reset();
}
fn playertable(p1: PlayerColors, p2: PlayerColors, p3: PlayerColors, state: Arc<AppState>) {
    led_reset(state.clone());
    let mut stripe = state.led_stripe.lock().unwrap();
    let mut v = Vec::new();

    for i in 0..stripe.get_number_of_leds() {
        v.push(match i {
            30..50 => LED::from_color(p1.get_color()),
            60..80 => LED::from_color(p2.get_color()),
            120..130 => LED::from_color(p3.get_color()),
            _ => LED(0, 0, 0),
        })
    }
    stripe.activate_frame(&Frame(v));
}
