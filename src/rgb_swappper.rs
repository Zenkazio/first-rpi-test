use crate::{distance::Hcsr04Observer, rgbled::RGBLed};

pub struct RBGSwapper {
    rgb_led: RGBLed,
}
impl RBGSwapper {
    pub fn new(rgb_led: RGBLed) -> Self {
        Self { rgb_led }
    }
}
impl Hcsr04Observer for RBGSwapper {
    fn update(&mut self, value: f64) {
        match value {
            0.0..10.0 => self.rgb_led.green().unwrap(),
            10.0..50.0 => self.rgb_led.set_rgb(255, 255, 0).unwrap(),
            50.0..100.0 => self.rgb_led.red().unwrap(),
            _ => self.rgb_led.clear().unwrap(),
        }
    }
}
