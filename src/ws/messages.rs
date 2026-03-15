use serde::{Deserialize, Serialize};

use crate::led::stripe::PlayerColors;

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMsg {
    StatusUpdate {
        value: String,
    },
    PlaySound {
        name: String,
    },
    TargetPositions {
        pos1: (f32, f32),
        vec1: (f32, f32),
        done1: bool,
        pos2: (f32, f32),
        vec2: (f32, f32),
        done2: bool,
        pos3: (f32, f32),
        vec3: (f32, f32),
        done3: bool,
    },
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
