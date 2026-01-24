#![allow(unused)]

const TIME_SLEEP: Duration = Duration::from_micros(20);
const WARMUP_TIME: Duration = Duration::from_millis(1);
const COOLDOWN_TIME: Duration = Duration::from_secs(2);

use std::{
    sync::{Arc, atomic::AtomicBool},
    thread::sleep,
    time::Duration,
};

use rppal::gpio::{Error, Gpio, OutputPin};

pub struct Stepper {
    ena: OutputPin,
    dir: OutputPin,
    step: OutputPin,
    steps_per_rot: u32,
    delay_time: Duration,
}

impl Stepper {
    pub fn new(ena: u8, dir: u8, step: u8, steps_per_rot: u32) -> Result<Self, Error> {
        Ok(Self {
            ena: Gpio::new()?.get(ena)?.into_output_high(),
            dir: Gpio::new()?.get(dir)?.into_output_low(),
            step: Gpio::new()?.get(step)?.into_output_low(),
            steps_per_rot: steps_per_rot,
            delay_time: TIME_SLEEP,
        })
    }
    pub fn turn_left(&mut self, left_running: Arc<AtomicBool>) {
        self.turn(left_running);
    }
    pub fn turn_right(&mut self, right_running: Arc<AtomicBool>) {
        self.dir.set_high();
        self.turn(right_running);
        self.dir.set_low();
    }
    fn turn(&mut self, running: Arc<AtomicBool>) {
        self.warmup();

        // for _ in 0..1600 {
        //     self.one_step();
        // }
        while running.load(std::sync::atomic::Ordering::SeqCst) {
            self.one_step();
        }
        // self.cooldown();
    }

    #[inline]
    fn one_step(&mut self) {
        self.step.set_high();
        sleep(self.delay_time);
        self.step.set_low();
        sleep(self.delay_time);
    }
    fn warmup(&mut self) {
        self.ena.set_low();
        sleep(WARMUP_TIME);
    }
    fn cooldown(&mut self) {
        sleep(COOLDOWN_TIME);
        self.ena.set_high();
    }
    pub fn set_rpm(&mut self, rpm: u32) {
        let hz = rpm * self.steps_per_rot / 60;
        self.delay_time = Duration::from_secs_f32(1.0 / (2.0 * hz as f32))
    }
    pub fn clear(&mut self) {
        self.dir.set_low();
        self.step.set_low();
    }
}
