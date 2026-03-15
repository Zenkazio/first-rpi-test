use std::sync::{Arc, Mutex, atomic::AtomicBool, mpsc::Sender};

use tokio::sync::broadcast;

use crate::{door, led::stripe::Stripe, ws::messages::ServerMsg};

pub struct AppState {
    pub led_repeat: Arc<AtomicBool>,
    pub led_stripe: Arc<Mutex<Stripe>>,
    // pub led_tx: Sender<led::stripe::Event>,
    pub door: Sender<door::door::Event>,

    pub tx: broadcast::Sender<ServerMsg>,
}
