const COOLDOWN_TIME: Duration = Duration::from_secs(1);

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Sender, channel},
    },
    thread::{self},
    time::Duration,
};

use rppal::gpio::{Error, Gpio, OutputPin};

pub struct Stepper {
    // ena: Arc<Mutex<OutputPin>>,
    pub dir: OutputPin,
    pub step: OutputPin,
    // steps_per_rot: u32,
    tx: Sender<bool>,
    step_counter: i64,
    canceler: Arc<AtomicBool>,
    steps_per_rot: u16,
    start_freq: f32,
    max_freq: f32,
    startup_steps: i64,
}

impl Stepper {
    pub fn new(ena: u8, dir: u8, step: u8, steps_per_rot: u16) -> Result<Self, Error> {
        let t = Self {
            // ena: a.clone(),
            dir: Gpio::new()?.get(dir)?.into_output_low(),
            step: Gpio::new()?.get(step)?.into_output_low(),
            tx: Stepper::spawn_watchdog(Gpio::new()?.get(ena)?.into_output_high()),
            step_counter: 0,
            canceler: Arc::new(AtomicBool::new(false)),
            steps_per_rot: steps_per_rot,
            start_freq: 300.0 * 2.0, //* 2.0 to get 50% dutycycle in pwm
            max_freq: 25000.0 * 2.0,
            startup_steps: 3000,
        };
        Ok(t)
    }

    fn spawn_watchdog(mut ena_pin: OutputPin) -> Sender<bool> {
        let (tx, rx) = channel();

        thread::spawn(move || {
            while let Ok(signal) = rx.recv() {
                if signal {
                    ena_pin.set_low();
                    continue;
                }

                loop {
                    match rx.recv_timeout(COOLDOWN_TIME) {
                        Ok(true) => {
                            break;
                        }
                        Ok(false) => {
                            continue;
                        }
                        Err(_) => {
                            ena_pin.set_high();
                            break;
                        }
                    }
                }
            }
        });
        tx
    }
    pub fn get_cancler_clone(&self) -> Arc<AtomicBool> {
        self.canceler.clone()
    }
    pub fn get_step_count(&self) -> i64 {
        self.step_counter
    }
    pub fn rot_ref(&self, steps: i64, base: i64) -> i64 {
        steps * self.steps_per_rot as i64 / base
    }
    pub fn turn_while<F>(&mut self, mut condition: F, steps: i64)
    where
        F: FnMut() -> bool,
    {
        let dir_positive = steps > 0;
        let step_delta: i64 = steps.signum();
        if dir_positive {
            self.dir.set_high();
        } else {
            self.dir.set_low();
        }

        let sleeper = spin_sleep::SpinSleeper::new(0);
        let dur = Duration::from_secs_f32(1.0 / self.start_freq);
        self.tx.send(true).expect("send failed true");
        while condition() {
            self.step.set_high();
            sleeper.sleep(dur);

            self.step_counter += step_delta;

            self.step.set_low();
            sleeper.sleep(dur);
        }
        self.tx.send(false).expect("send failed false");
    }

    pub fn turn_to(&mut self, to_step: i64) {
        let start = std::time::Instant::now();
        self.canceler.store(false, Ordering::SeqCst);
        let do_steps = to_step - self.step_counter;
        let do_steps_abs = do_steps.abs();
        if do_steps == 0 {
            return;
        }

        let dir_positive = do_steps > 0;
        let step_delta: i64 = do_steps.signum();

        if dir_positive {
            self.dir.set_high();
        } else {
            self.dir.set_low();
        }
        let sleeper = spin_sleep::SpinSleeper::new(0);
        self.tx.send(true).expect("send failed true");
        for i in 0..do_steps_abs {
            let step = i.min(do_steps_abs - i);

            let freq = if step > self.startup_steps {
                self.max_freq
            } else {
                linear_growth(step, self.start_freq, self.max_freq, self.startup_steps)
                // logistic_growth(step, self.start_freq, self.max_freq, self.startup_steps)
            };

            let dur = Duration::from_secs_f32(1.0 / freq);

            self.step.set_high();
            sleeper.sleep(dur);

            self.step_counter += step_delta;

            self.step.set_low();
            sleeper.sleep(dur);

            // if self.canceler.load(Ordering::SeqCst) {
            //     break;
            // }
        }

        self.tx.send(false).expect("send failed false");
        // sleeper.sleep(Duration::from_millis(500));
        println!("time {}ms", start.elapsed().as_millis());
    }

    pub fn reset_step_count(&mut self) {
        self.step_counter = 0;
    }
    pub fn set_step_count(&mut self, steps: i64) {
        self.step_counter = steps;
    }
}
#[inline]
fn logistic_growth(step: i64, start_freq: f32, max_freq: f32, startup_steps: i64) -> f32 {
    let k = 0.1; // Wachstumsrate, anpassen für gewünschte Steilheit
    let x0 = startup_steps as f32 / 2.0; // Wendepunkt in der Mitte der Startup-Phase

    let x = step as f32;
    start_freq + (max_freq - start_freq) / (1.0 + (-k * (x - x0)).exp())
}
#[inline]
fn linear_growth(step: i64, start_freq: f32, max_freq: f32, startup_steps: i64) -> f32 {
    let m = (max_freq - start_freq) / startup_steps as f32;
    step as f32 * m + start_freq
}
