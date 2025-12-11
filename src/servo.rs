#![allow(unused)]
use std::{thread, time::Duration};

use rppal::gpio::{Error, Gpio, OutputPin};

pub struct Servo {
    pulse_pin: OutputPin,
    last_degree: u8,
}

impl Servo {
    pub fn new(pulse_pin: u8) -> Result<Self, Error> {
        let mut s = Self {
            pulse_pin: Gpio::new()?.get(pulse_pin)?.into_output_low(),
            last_degree: 0,
        };
        Ok(s)
    }

    pub fn set_degree(&mut self, degree: u8) {
        if degree > 180 {
            return;
        }
        // always a 50Hz base
        self.pulse_pin
            .set_pwm_frequency(50.0, 0.05 + (degree as f64 / 180.0) * 0.05 * 1.6);
        let diff = self.last_degree.abs_diff(degree);
        thread::sleep(Duration::from_millis(2 * diff as u64));
        self.last_degree = degree;
    }
}
