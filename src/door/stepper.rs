const COOLDOWN_TIME: Duration = Duration::from_secs(1);

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Sender, channel},
    },
    thread::{self},
    time::{Duration, Instant},
};

use rppal::gpio::{Error, Gpio, OutputPin};

pub enum PulsePerRotation {
    PPR200,
    PPR800,
    PPR1600,
    PPR3200,
    PPR6400,
}

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
            start_freq: Stepper::rot_ref_base(300, 1600, steps_per_rot as i64) as f32,
            max_freq: Stepper::rot_ref_base(35000, 1600, steps_per_rot as i64) as f32,
            startup_steps: Stepper::rot_ref_base(2900, 1600, steps_per_rot as i64),
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
        Stepper::rot_ref_base(steps, base, self.steps_per_rot as i64)
    }
    pub fn rot_ref_base(steps: i64, base: i64, referenz: i64) -> i64 {
        steps * referenz / base
    }
    pub fn turn_while<F>(&mut self, condition: F, steps: i64)
    where
        F: Fn() -> bool,
    {
        let dir_positive = steps > 0;
        let step_delta: i64 = steps.signum();
        if dir_positive {
            self.dir.set_high();
        } else {
            self.dir.set_low();
        }

        let sleeper = spin_sleep::SpinSleeper::new(0);
        // let high = Duration::from_secs_f32(1.0 / self.start_freq);
        // let low = Duration::from_secs()
        let dur = Duration::from_secs_f32(1.0 / (self.start_freq * 2.0));
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

    pub fn turn_to(&mut self, step: i64) {
        let start = Instant::now();
        let do_steps = step - self.step_counter;
        if do_steps == 0 {
            return;
        }

        let step_delta: i64 = do_steps.signum();

        if do_steps > 0 {
            self.dir.set_high();
        } else {
            self.dir.set_low();
        }

        let sleeper = spin_sleep::SpinSleeper::new(0);
        let do_steps_abs = do_steps.abs();
        let mut c = 0;
        let mut istep = 0;
        let _ = self.tx.send(true);
        while self.step_counter != step && !self.canceler.load(Ordering::SeqCst) {
            istep = c.min(do_steps_abs - c);

            let freq = if istep > self.startup_steps {
                self.max_freq
            } else {
                linear_growth(istep, self.start_freq, self.max_freq, self.startup_steps)
                // logistic_growth(step, self.start_freq, self.max_freq, self.startup_steps)
            };

            let dur = Duration::from_secs_f32(1.0 / (freq * 2.0));

            self.step.set_high();
            sleeper.sleep(dur);

            self.step_counter += step_delta;

            self.step.set_low();
            sleeper.sleep(dur);

            c += 1;
        }

        for i in (0..(istep - 1).min(self.startup_steps)).rev() {
            let freq =
                linear_growth(i, self.start_freq, self.max_freq, self.startup_steps)
                // logistic_growth(step, self.start_freq, self.max_freq, self.startup_steps)
            ;

            let dur = Duration::from_secs_f32(1.0 / (freq * 2.0));

            self.step.set_high();
            sleeper.sleep(dur);

            self.step_counter += step_delta;

            self.step.set_low();
            sleeper.sleep(dur);
        }

        let _ = self.tx.send(false);
        self.canceler.store(false, Ordering::SeqCst);
        println!("time {}ms", start.elapsed().as_millis());
        sleeper.sleep(Duration::from_millis(50));
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
