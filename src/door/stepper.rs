#![allow(unused)]

const WARMUP_TIME: Duration = Duration::from_millis(10);
const COOLDOWN_TIME: Duration = Duration::from_secs(1);

const START_FREQUENCY: f64 = 200.0;

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Sender, channel},
    },
    thread::{self, sleep},
    time::Duration,
};

use rppal::gpio::{Error, Gpio, OutputPin};
use rppal::pwm::{Channel, Polarity, Pwm};

pub struct Stepper {
    ena: Arc<Mutex<OutputPin>>,
    dir: OutputPin,
    step: OutputPin,
    steps_per_rot: u32,
    tx: Sender<bool>,
    is_running: Arc<AtomicBool>,
    step_counter: i64,
}

impl Stepper {
    pub fn new(ena: u8, dir: u8, step: u8, steps_per_rot: u32) -> Result<Self, Error> {
        let a = Arc::new(Mutex::new(Gpio::new()?.get(ena)?.into_output_high()));

        let t = Self {
            ena: a.clone(),
            dir: Gpio::new()?.get(dir)?.into_output_low(),
            step: Gpio::new()?.get(step)?.into_output_low(),
            steps_per_rot: steps_per_rot,
            tx: Stepper::spawn_watchdof(a),
            is_running: Arc::new(AtomicBool::new(false)),
            step_counter: 0,
        };
        Ok(t)
    }
    fn spawn_watchdof(ena_pin: Arc<Mutex<OutputPin>>) -> Sender<bool> {
        let (tx, rx) = channel();

        thread::spawn(move || {
            let mut ena_active = false;

            loop {
                match rx.recv_timeout(COOLDOWN_TIME) {
                    Ok(true) => {
                        if !ena_active {
                            ena_pin.lock().unwrap().set_low();
                            ena_active = true;
                            sleep(WARMUP_TIME);
                        }
                    }
                    Ok(false) => {
                        ena_active = false;
                    }
                    Err(_) => {
                        if !ena_active {
                            ena_pin.lock().unwrap().set_high();
                        }
                    }
                }
            }
        });
        tx
    }
    pub fn get_running_clone(&self) -> Arc<AtomicBool> {
        self.is_running.clone()
    }

    pub fn turn_to(&mut self, to_step: i64) {
        let do_steps = to_step - self.step_counter;
        let do_steps_abs = do_steps.abs();
        if do_steps == 0 {
            return;
        }
        self.tx.send(true);
        if do_steps > 0 {
            self.dir.set_high();
            for _ in 0..do_steps_abs {
                self.step.set_high();
                self.step_counter += 1;
                self.step.set_low();
            }
        } else {
            self.dir.set_low();
            for _ in 0..do_steps_abs {
                self.step.set_high();
                self.step_counter -= 1;
                self.step.set_low();
            }
        }
        self.tx.send(false);
    }
    pub fn turn_left(&mut self) {}
    pub fn turn_right(&mut self) {}

    pub fn clear(&mut self) {
        self.step.set_low();
    }
    pub fn reset_step_count(&mut self) {
        self.step_counter = 0;
    }
}
