#![allow(unused)]
use rppal::gpio::{Error, Gpio, OutputPin};

pub struct Servo {
    pulse_pin: OutputPin,
}

impl Servo {
    pub fn new(pulse_pin: u8) -> Result<Self, Error> {
        Ok(Self {
            pulse_pin: Gpio::new()?.get(pulse_pin)?.into_output_low(),
        })
    }

    pub fn set_degree(&mut self, degree: u8) {
        if degree > 180 {
            return;
        }
        // always a 50Hz base
        self.pulse_pin.set_pwm_frequency(50.0, 0.05);
    }
}
