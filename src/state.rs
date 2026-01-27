use std::sync::{Arc, Mutex, atomic::AtomicBool};

use tokio::sync::broadcast;

use crate::{led::stripe::Stripe, stepper::Stepper, ws::messages::ServerMsg};

pub struct AppState {
    pub led_stripe: Arc<Mutex<Stripe>>,
    pub left_running: Arc<AtomicBool>,
    pub right_running: Arc<AtomicBool>,
    pub led_repeat: Arc<AtomicBool>,
    pub led_thread_mutex: Arc<Mutex<()>>,

    pub stepper: Arc<Mutex<Stepper>>,

    pub tx: broadcast::Sender<ServerMsg>,
}
