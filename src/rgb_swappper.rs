use std::sync::{Arc, Mutex};

use crate::{distance::Hcsr04Observer, rgbled::RGBLed};

pub struct RBGSwapper {
    rgb_led: Arc<Mutex<RGBLed>>,
}
impl RBGSwapper {
    pub fn new(rgb_led: Arc<Mutex<RGBLed>>) -> Self {
        Self { rgb_led: rgb_led }
    }
}
impl Hcsr04Observer for RBGSwapper {
    fn update(&self, value: f64) {
        // dbg!(value);
        match value {
            0.0..25.0 => self.rgb_led.lock().unwrap().green().unwrap(),
            25.0..50.0 => self.rgb_led.lock().unwrap().set_rgb(255, 255, 0).unwrap(),
            50.0..1000.0 => self.rgb_led.lock().unwrap().red().unwrap(),
            _ => self.rgb_led.lock().unwrap().clear().unwrap(),
        }
    }
}
