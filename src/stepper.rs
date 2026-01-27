#![allow(unused)]

const TIME_SLEEP: Duration = Duration::from_micros(20);
const WARMUP_TIME: Duration = Duration::from_millis(10);
const COOLDOWN_TIME: Duration = Duration::from_secs(1);

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
    delay_time: Duration,
    tx: Sender<bool>,
    pwm_step: Pwm,
    is_running: Arc<AtomicBool>,
}

impl Stepper {
    pub fn new(ena: u8, dir: u8, step: u8, steps_per_rot: u32) -> Result<Self, Error> {
        let a = Arc::new(Mutex::new(Gpio::new()?.get(ena)?.into_output_high()));

        let t = Self {
            ena: a.clone(),
            dir: Gpio::new()?.get(dir)?.into_output_low(),
            step: Gpio::new()?.get(step)?.into_output_low(),
            steps_per_rot: steps_per_rot,
            delay_time: TIME_SLEEP,
            tx: Stepper::spawn_watchdof(a),
            pwm_step: Pwm::with_frequency(Channel::Pwm0, 200.0, 0.5, Polarity::Normal, false)
                .unwrap(),
            is_running: Arc::new(AtomicBool::new(false)),
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
    pub fn turn_left(&mut self) {
        self.dir.set_low();
        self.turn();
    }
    pub fn turn_right(&mut self) {
        self.dir.set_high();
        self.turn();
    }
    fn turn(&mut self) {
        self.tx.send(true);

        let target_freq: f64 = 12800.0;
        let start_freq = 200.0;
        let steps = 100;
        let ramp_duration = Duration::from_millis(500);
        let step_delay = ramp_duration / steps;
        let freq_increment = (target_freq - start_freq) / steps as f64;

        let mut current_freq = start_freq;

        for _ in 0..steps {
            if !self.is_running.load(Ordering::SeqCst) {
                break;
            }
            self.pwm_step.set_frequency(current_freq, 0.5);
            self.pwm_step.enable();
            sleep(step_delay);
            current_freq += freq_increment;
        }

        self.pwm_step.set_frequency(target_freq, 0.5);
        self.pwm_step.enable();
        while self.is_running.load(Ordering::SeqCst) {
            sleep(TIME_SLEEP);
        }
        let mut current_freq = target_freq;

        for _ in 0..steps {
            current_freq -= freq_increment;
            if current_freq < start_freq {
                break;
            }
            self.pwm_step.set_frequency(current_freq, 0.5);
            sleep(step_delay);
        }

        self.pwm_step.disable();
        self.tx.send(false);
    }

    #[inline]
    fn one_step(&mut self) {
        self.step.set_high();
        sleep(self.delay_time);
        self.step.set_low();
        sleep(self.delay_time);
    }

    pub fn set_rpm(&mut self, rpm: u32) {
        let hz = rpm * self.steps_per_rot / 60;
        self.delay_time = Duration::from_secs_f32(1.0 / (2.0 * hz as f32))
    }
    pub fn clear(&mut self) {
        self.step.set_low();
        self.pwm_step.disable();
    }
}
