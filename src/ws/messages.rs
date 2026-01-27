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
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WorkMode {
    Static,
    Blink,
    Dot,
    Custom,
}
