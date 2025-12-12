#![allow(unused)]

use std::sync::{Arc, Mutex};

use rppal::i2c::{Error, I2c};

fn bcd2dec(bcd: u8) -> u8 {
    (((bcd & 0xF0) >> 4) * 10) + (bcd & 0x0F)
}

fn dec2bcd(dec: u8) -> u8 {
    ((dec / 10) << 4) | (dec % 10)
}

trait I2CDevice {
    fn get_address() -> u16;
}

pub struct I2CMaster {
    i2c: I2c,
}

impl I2CMaster {
    pub fn new() -> Result<Self, Error> {
        Ok(Self { i2c: I2c::new()? })
    }

    pub fn send(&mut self, slave_address: u16, reg: u8, block: &[u8]) -> Result<(), Error> {
        self.i2c.set_slave_address(slave_address)?;
        self.i2c.block_write(reg, block);
        Ok(())
    }
    pub fn read(&mut self, slave_address: u16, reg: u8, block: &mut [u8]) -> Result<(), Error> {
        self.i2c.set_slave_address(slave_address)?;
        self.i2c.block_read(reg, block);
        Ok(())
    }
}

pub struct MPU6050 {
    address: u16,
    i2c_master: Arc<Mutex<I2CMaster>>,
}

impl MPU6050 {
    pub fn new(i2c_master: Arc<Mutex<I2CMaster>>) -> Self {
        Self {
            address: 0x69,
            i2c_master,
        }
    }
    pub fn get_temperature(&mut self) -> f32 {
        let mut temp_raw = [0u8, 2];
        self.i2c_master
            .lock()
            .unwrap()
            .read(self.address, 0x41, &mut temp_raw);
        let raw = ((temp_raw[0] as i16) << 8) | (temp_raw[1] as i16);
        let temperature_c = (raw as f32 / 340.0) + 36.53;

        temperature_c
    }
}
