mod door_statemachine;
mod led;
mod stepper;
use axum::{
    Router,
    body::Body,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    http::{Response, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use futures::{SinkExt, StreamExt};
use include_dir::{Dir, include_dir};
use rand::Rng;

use std::{
    error::Error,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{sync::broadcast, task::spawn_blocking, time::sleep};

use crate::{led::stripe::Stripe, stepper::Stepper};

use serde::{Deserialize, Serialize};

static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/assets");

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
enum ServerMsg {
    CounterUpdate { value: i32 },
    PlaySound { name: String },
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMsg {
    Increment,
    Decrement,
    Reset,
    UpdateSettings {
        r: u8,
        g: u8,
        b: u8,
        mode: WorkMode,
        speed: f32,
        repeat: bool,
    },
    RedAlert,
    LEDReset,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
enum WorkMode {
    Static,
    Blink,
    Dot,
    Custom,
}

struct AppState {
    led_stripe: Arc<Mutex<Stripe>>,
    left_running: Arc<AtomicBool>,
    right_running: Arc<AtomicBool>,
    led_repeat: Arc<AtomicBool>,
    led_thread_mutex: Arc<Mutex<()>>,

    stepper: Arc<Mutex<Stepper>>,

    counter: Arc<Mutex<i32>>,
    tx: broadcast::Sender<ServerMsg>,
}

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

    // stepper.turn_left(Arc::new(AtomicBool::new(true)));

    // return Ok(());
    let shared_state = Arc::new(AppState {
        led_stripe: led_stripe,
        left_running: Arc::new(AtomicBool::new(false)),
        right_running: Arc::new(AtomicBool::new(false)),
        led_repeat: t_bool,
        led_thread_mutex: Arc::new(Mutex::new(())),

        stepper: Arc::new(Mutex::new(stepper)),

        counter: Arc::new(Mutex::new(0)),
        tx,
    });

    tokio::spawn(counter_task(shared_state.clone()));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/left/start", get(left_start_handler))
        .route("/left/stop", get(left_stop_handler))
        .route("/right/start", get(right_start_handler))
        .route("/right/stop", get(right_stop_handler))
        .fallback(get(static_handler))
        .with_state(shared_state);

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

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
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
            let mut counter = state.counter.lock().unwrap();

            match cmd {
                ClientMsg::Increment => {
                    *counter += 1;
                    let _ = state.tx.send(ServerMsg::CounterUpdate { value: *counter });
                }
                ClientMsg::Decrement => {
                    *counter -= 1;
                    let _ = state.tx.send(ServerMsg::CounterUpdate { value: *counter });
                }
                ClientMsg::Reset => {
                    *counter = 0;
                    let _ = state.tx.send(ServerMsg::CounterUpdate { value: *counter });
                }
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
                        println!("spawn thread");
                        led_repeat_copy.store(false, Ordering::SeqCst);
                        let _guard = led_thread_mutex_copy.lock().unwrap();
                        eprintln!("start work");
                        led_repeat_copy.store(repeat, Ordering::SeqCst);
                        let mut stripe = led_stipe_copy.lock().unwrap();
                        use WorkMode::*;
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
                        println!("end work");
                    });
                    println!("New LED Stripe Data: {:?}\n{:?}", mode, speed);
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
async fn counter_task(state: Arc<AppState>) {
    loop {
        sleep(Duration::from_secs(1)).await;

        let mut counter = state.counter.lock().unwrap();
        *counter += 1;

        let _ = state.tx.send(ServerMsg::CounterUpdate { value: *counter });
    }
}

async fn static_handler(uri: axum::http::Uri) -> impl IntoResponse {
    // z.B. "/sounds/reset.mp3" -> "sounds/reset.mp3"
    let path = uri.path().trim_start_matches('/');

    // "/" -> index.html
    let path = if path.is_empty() { "index.html" } else { path };

    match ASSETS.get_file(path) {
        Some(file) => {
            let body = Body::from(file.contents());
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(body)
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(format!("File not found: {}", path)))
            .unwrap(),
    }
}
