#![allow(unused)]

use rppal::gpio::{Error, Gpio, Level, OutputPin};
const STEPS_PER_ROTATION: u16 = 2048;
pub struct Stepper {
    int1: OutputPin,
    int2: OutputPin,
    int3: OutputPin,
    int4: OutputPin,
    phases: [[Level; 4]; 8],
}

impl Stepper {
    pub fn new(int1: u8, int2: u8, int3: u8, int4: u8) -> Result<Self, Error> {
        Ok(Self {
            int1: Gpio::new()?.get(int1)?.into_output_low(),
            int2: Gpio::new()?.get(int2)?.into_output_low(),
            int3: Gpio::new()?.get(int3)?.into_output_low(),
            int4: Gpio::new()?.get(int4)?.into_output_low(),
            phases: [
                [Level::High, Level::Low, Level::Low, Level::High],
                [Level::High, Level::Low, Level::Low, Level::Low],
                [Level::High, Level::High, Level::Low, Level::Low],
                [Level::Low, Level::High, Level::Low, Level::Low],
                [Level::Low, Level::High, Level::High, Level::Low],
                [Level::Low, Level::Low, Level::High, Level::Low],
                [Level::Low, Level::Low, Level::High, Level::High],
                [Level::Low, Level::Low, Level::Low, Level::High],
            ],
        })
    }

    pub fn step(&mut self) {
        for s in self.phases {
            self.int1.write(s[0]);
            self.int2.write(s[1]);
            self.int3.write(s[2]);
            self.int4.write(s[3]);
        }
    }
}
