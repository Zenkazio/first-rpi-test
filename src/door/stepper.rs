// #![allow(unused)]

const COOLDOWN_TIME: Duration = Duration::from_secs(1);

const START_FREQUENCY: f64 = 200.0 * 2.0; // double to get 0.5 dutycycle
const MAX_FREQUENCY: f64 = 12800.0 * 2.0;
const STARTUP_STEPS: i64 = 10000;

use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Sender, channel},
    },
    thread::{self, sleep},
    time::Duration,
};

use rppal::gpio::{Error, Gpio, OutputPin};

pub struct Stepper {
    // ena: Arc<Mutex<OutputPin>>,
    dir: OutputPin,
    step: OutputPin,
    // steps_per_rot: u32,
    tx: Sender<bool>,
    step_counter: i64,
}

impl Stepper {
    pub fn new(ena: u8, dir: u8, step: u8) -> Result<Self, Error> {
        let a = Arc::new(Mutex::new(Gpio::new()?.get(ena)?.into_output_high()));

        let t = Self {
            // ena: a.clone(),
            dir: Gpio::new()?.get(dir)?.into_output_low(),
            step: Gpio::new()?.get(step)?.into_output_low(),
            // steps_per_rot: steps_per_rot,
            tx: Stepper::spawn_watchdog(a),
            step_counter: 0,
        };
        Ok(t)
    }
    fn spawn_watchdog(ena_pin: Arc<Mutex<OutputPin>>) -> Sender<bool> {
        let (tx, rx) = channel();

        thread::spawn(move || {
            let mut ena_active = false;

            loop {
                match rx.recv_timeout(COOLDOWN_TIME) {
                    Ok(true) => {
                        if !ena_active {
                            ena_pin.lock().unwrap().set_low();
                            ena_active = true;
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

    pub fn turn_to(&mut self, to_step: i64) {
        let do_steps = to_step - self.step_counter;
        let do_steps_abs = do_steps.abs();
        if do_steps == 0 {
            return;
        }
        self.tx.send(true).expect("send failed true");
        let m = (MAX_FREQUENCY - START_FREQUENCY) / STARTUP_STEPS as f64;
        if do_steps > 0 {
            self.dir.set_high();
            for i in 0..do_steps_abs {
                let step = i.min(do_steps_abs - i);

                let freq = if step > STARTUP_STEPS {
                    MAX_FREQUENCY
                } else {
                    step as f64 * m + START_FREQUENCY
                };
                let dur = Duration::from_secs_f64(1.0 / freq);
                self.step.set_high();
                sleep(dur);
                self.step_counter += 1;
                self.step.set_low();
                sleep(dur);
            }
        } else {
            self.dir.set_low();
            for i in 0..do_steps_abs {
                let step = i.min(do_steps_abs - i);

                let freq = if step > STARTUP_STEPS {
                    MAX_FREQUENCY
                } else {
                    step as f64 * m + START_FREQUENCY
                };
                let dur = Duration::from_secs_f64(1.0 / freq);
                self.step.set_high();
                sleep(dur);
                self.step_counter -= 1;
                self.step.set_low();
                sleep(dur);
            }
        }
        self.tx.send(false).expect("send failed false");
        println!("step count{}", self.step_counter);
    }

    pub fn reset_step_count(&mut self) {
        self.step_counter = 0;
    }
}
