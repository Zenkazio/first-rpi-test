#![allow(dead_code)]

use rppal::gpio::{Error, Gpio, InputPin, OutputPin};

pub struct Keypad {
    in1: InputPin,
    in2: InputPin,
    in3: InputPin,
    in4: InputPin,
    out1: OutputPin,
    out2: OutputPin,
    out3: OutputPin,
    out4: OutputPin,
    pub state: [[bool; 4]; 4],
}

impl Keypad {
    pub fn new(
        i1: u8,
        i2: u8,
        i3: u8,
        i4: u8,
        o1: u8,
        o2: u8,
        o3: u8,
        o4: u8,
    ) -> Result<Self, Error> {
        Ok(Keypad {
            in1: Gpio::new()?.get(i1)?.into_input_pulldown(),
            in2: Gpio::new()?.get(i2)?.into_input_pulldown(),
            in3: Gpio::new()?.get(i3)?.into_input_pulldown(),
            in4: Gpio::new()?.get(i4)?.into_input_pulldown(),
            out1: Gpio::new()?.get(o1)?.into_output_low(),
            out2: Gpio::new()?.get(o2)?.into_output_low(),
            out3: Gpio::new()?.get(o3)?.into_output_low(),
            out4: Gpio::new()?.get(o4)?.into_output_low(),
            state: [
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
                [false, false, false, false],
            ],
        })
    }
    pub fn cycle(&mut self) -> Result<(), Error> {
        self.out1.set_high();
        self.state[0][0] = self.in1.is_high();
        self.state[0][1] = self.in2.is_high();
        self.state[0][2] = self.in3.is_high();
        self.state[0][3] = self.in4.is_high();
        self.out1.set_low();
        self.out2.set_high();
        self.state[1][0] = self.in1.is_high();
        self.state[1][1] = self.in2.is_high();
        self.state[1][2] = self.in3.is_high();
        self.state[1][3] = self.in4.is_high();
        self.out2.set_low();
        self.out3.set_high();
        self.state[2][0] = self.in1.is_high();
        self.state[2][1] = self.in2.is_high();
        self.state[2][2] = self.in3.is_high();
        self.state[2][3] = self.in4.is_high();
        self.out3.set_low();
        self.out4.set_high();
        self.state[3][0] = self.in1.is_high();
        self.state[3][1] = self.in2.is_high();
        self.state[3][2] = self.in3.is_high();
        self.state[3][3] = self.in4.is_high();
        self.out4.set_low();
        Ok(())
    }
    pub fn display_state(&self) {
        println!(
            "{} {} {} {}\n{} {} {} {}\n{} {} {} {}\n{} {} {} {}\n=======",
            self.state[0][0] as u8,
            self.state[0][1] as u8,
            self.state[0][2] as u8,
            self.state[0][3] as u8,
            self.state[1][0] as u8,
            self.state[1][1] as u8,
            self.state[1][2] as u8,
            self.state[1][3] as u8,
            self.state[2][0] as u8,
            self.state[2][1] as u8,
            self.state[2][2] as u8,
            self.state[2][3] as u8,
            self.state[3][0] as u8,
            self.state[3][1] as u8,
            self.state[3][2] as u8,
            self.state[3][3] as u8
        )
    }
}
