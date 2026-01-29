use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMsg {
    StatusUpdate { value: String },
    PlaySound { name: String },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ClientMsg {
    LeftStart,
    RightStart,
    StepperStop,
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

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PlayerColors {
    White,
    Green,
    Blue,
    Orange,
}
impl PlayerColors {
    pub fn get_color(&self) -> (u8, u8, u8) {
        match self {
            PlayerColors::White => (255, 255, 255),
            PlayerColors::Green => (0, 255, 0),
            PlayerColors::Blue => (0, 0, 255),
            PlayerColors::Orange => (255, 147, 15),
        }
    }
}
