use serde::{Deserialize, Serialize};

use crate::{door::detector::Target, led::stripe::PlayerColors};

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMsg {
    StatusUpdate { value: String },
    PlaySound { name: String },
    Targets { id: u8, targets: [Target; 3] },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMsg {
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
    PlayerTable {
        p1: PlayerColors,
        p2: PlayerColors,
        p3: PlayerColors,
    },
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WorkMode {
    Static,
    Blink,
    Dot,
    Custom,
}
