#[derive(Debug, Clone)]
pub struct LED(pub u8, pub u8, pub u8);

impl LED {
    pub fn from_color(color: (u8, u8, u8)) -> LED {
        LED(color.0, color.1, color.2)
    }
    pub fn get_color(&self) -> (u8, u8, u8) {
        (self.0, self.1, self.2)
    }
    pub fn scale(&self, fac: f32) -> LED {
        LED(
            (self.0 as f32 * fac) as u8,
            (self.1 as f32 * fac) as u8,
            (self.2 as f32 * fac) as u8,
        )
    }
    pub fn lerp(&self, led2: &LED, t: f32) -> LED {
        let t = t.clamp(0.0, 1.0); // Sicherstellen, dass t im Bereich [0, 1] liegt
        let r = (self.0 as f32 * (1.0 - t) + led2.0 as f32 * t) as u8;
        let g = (self.1 as f32 * (1.0 - t) + led2.1 as f32 * t) as u8;
        let b = (self.2 as f32 * (1.0 - t) + led2.2 as f32 * t) as u8;
        LED(r, g, b)
    }
    pub fn add(&self, other: &Self) -> Self {
        LED(
            (self.0 as u16 + other.0 as u16).min(255) as u8,
            (self.1 as u16 + other.1 as u16).min(255) as u8,
            (self.2 as u16 + other.2 as u16).min(255) as u8,
        )
    }
}
impl Default for LED {
    fn default() -> Self {
        Self(Default::default(), Default::default(), Default::default())
    }
}
/// 0.0 = led1
/// 1.0 = led2
pub fn lerp_leds(led1: &LED, led2: &LED, t: f32) -> LED {
    let t = t.clamp(0.0, 1.0); // Sicherstellen, dass t im Bereich [0, 1] liegt
    let r = (led1.0 as f32 * (1.0 - t) + led2.0 as f32 * t) as u8;
    let g = (led1.1 as f32 * (1.0 - t) + led2.1 as f32 * t) as u8;
    let b = (led1.2 as f32 * (1.0 - t) + led2.2 as f32 * t) as u8;
    LED(r, g, b)
}
