use std::sync::{Arc, Mutex, atomic::AtomicBool};

use tokio::sync::broadcast;

use crate::{door::stepper::Stepper, led::stripe::Stripe, ws::messages::ServerMsg};

pub struct AppState {
    pub led_repeat: Arc<AtomicBool>,
    pub led_stripe: Arc<Mutex<Stripe>>,

    pub stepper_running: Arc<AtomicBool>,
    pub stepper: Arc<Mutex<Stepper>>,

    pub tx: broadcast::Sender<ServerMsg>,
}
