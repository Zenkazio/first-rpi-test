use std::sync::{Arc, Mutex, atomic::AtomicBool};

use tokio::sync::broadcast;

use crate::{door::statemachine::Door, led::stripe::Stripe, ws::messages::ServerMsg};

pub struct AppState {
    pub led_repeat: Arc<AtomicBool>,
    pub led_stripe: Arc<Mutex<Stripe>>,

    pub door: Arc<Mutex<Door>>,
    pub door_cancler: Arc<AtomicBool>,

    pub tx: broadcast::Sender<ServerMsg>,
}
