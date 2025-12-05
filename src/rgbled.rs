#![allow(dead_code)]
use rppal::gpio::{Error, Gpio, OutputPin};

pub struct RGBLed {
    r_pin: OutputPin,
    g_pin: OutputPin,
    b_pin: OutputPin,
    freq: f64,
}

impl RGBLed {
    pub fn new(r_pin: u8, g_pin: u8, b_pin: u8) -> Result<Self, Error> {
        let r = Gpio::new()?.get(r_pin)?.into_output_low();
        let g = Gpio::new()?.get(g_pin)?.into_output_low();
        let b = Gpio::new()?.get(b_pin)?.into_output_low();
        Ok(RGBLed {
            r_pin: r,
            g_pin: g,
            b_pin: b,
            freq: 2000.0,
        })
    }
    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) -> Result<(), Error> {
        self.clear()?;
        self.r_pin
            .set_pwm_frequency(self.freq, r as f64 / 255 as f64)?;
        self.g_pin
            .set_pwm_frequency(self.freq, g as f64 / 255 as f64)?;
        self.b_pin
            .set_pwm_frequency(self.freq, b as f64 / 255 as f64)?;
        Ok(())
    }
    pub fn clear(&mut self) -> Result<(), Error> {
        self.r_pin.clear_pwm()?;
        self.g_pin.clear_pwm()?;
        self.b_pin.clear_pwm()?;
        self.r_pin.set_low();
        self.g_pin.set_low();
        self.b_pin.set_low();
        Ok(())
    }
    pub fn red(&mut self) -> Result<(), Error> {
        self.clear()?;
        self.r_pin.set_high();
        Ok(())
    }
    pub fn green(&mut self) -> Result<(), Error> {
        self.clear()?;
        self.g_pin.set_high();
        Ok(())
    }
    pub fn blue(&mut self) -> Result<(), Error> {
        self.clear()?;
        self.b_pin.set_high();
        Ok(())
    }
}
