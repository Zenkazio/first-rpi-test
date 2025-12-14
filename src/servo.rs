#![allow(unused)]
use std::{thread, time::Duration};

use rppal::gpio::{Error, Gpio, OutputPin};

pub struct Servo {
    pulse_pin: OutputPin,
    last_degree: i8,
}

impl Servo {
    pub fn new(pulse_pin: u8) -> Result<Self, Error> {
        let mut s = Self {
            pulse_pin: Gpio::new()?.get(pulse_pin)?.into_output_low(),
            last_degree: 0,
        };
        Ok(s)
    }

    pub fn set_degree(&mut self, degree: i8) {
        if degree > 90 || degree < -90 {
            return;
        }
        // always a 50Hz base
        self.pulse_pin
            .set_pwm_frequency(50.0, (degree as f64 * (0.025 / 90.0)) + 0.075);
        let diff = self.last_degree.abs_diff(degree);
        thread::sleep(Duration::from_millis(2 * diff as u64));
        self.last_degree = degree;
    }
}

// 0 -> 1.5ms -> 0.075
// -90 -> 1ms -> 0.05
// 90 -> 2ms -> 0.1
