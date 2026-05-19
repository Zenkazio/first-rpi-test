const COOLDOWN_TIME: Duration = Duration::from_millis(500);

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
    wheel_size: f32, // circumfrence in centimeter
    min_freq: f32,
    max_freq: f32,
    startup_steps: i64,
    lookuptable: Vec<(Duration, Duration)>,
}

impl Stepper {
    pub fn new(
        ena: u8,
        dir: u8,
        step: u8,
        steps_per_rot: u16,
        wheel_size: f32,
    ) -> Result<Self, Error> {
        let t = Self {
            // ena: a.clone(),
            dir: Gpio::new()?.get(dir)?.into_output_low(),
            step: Gpio::new()?.get(step)?.into_output_low(),
            tx: Stepper::spawn_watchdog(Gpio::new()?.get(ena)?.into_output_high()),
            step_counter: 0,
            canceler: Arc::new(AtomicBool::new(false)),
            steps_per_rot: steps_per_rot,
            wheel_size: wheel_size,
            min_freq: 300.0,
            max_freq: 20_000.0, //this is the cap of the driver TB6600
            startup_steps: 4000,
            lookuptable: Stepper::generate_lookuptable(8000),
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
    pub fn cm_to_steps(&self, cm: f32) -> i64 {
        (cm / self.wheel_size * self.steps_per_rot as f32) as i64
    }
    pub fn get_fmax(&self, distance_in_cm: f32, time: f32) -> f32 {
        (self.cm_to_steps(distance_in_cm) as f32 / time) * 2.0
    }
    fn frequency_to_highlow(freq: f32) -> (Duration, Duration) {
        let time = Duration::from_secs_f32(1.0 / freq);
        let high = time / 4;
        let low = high * 3;
        (high, low)
    }
    fn step_to_frequency(step: i64) -> f32 {
        //this expects a two second movement
        (step * 2) as f32
    }
    fn generate_lookuptable(steps: i64) -> Vec<(Duration, Duration)> {
        let mut vec = Vec::new();
        //table of max 8000 steps
        for i in 0..steps {
            vec.push(Stepper::frequency_to_highlow(
                Stepper::step_to_frequency(i) + 300.0,
            ));
        }
        vec
    }
    pub fn turn_while<F>(&mut self, condition: F, steps: i64, freq: f32)
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
        let dur = Duration::from_secs_f32(1.0 / (freq * 2.0));
        self.tx.send(true).expect("send failed true");
        while condition() {
            self.step.set_high();
            sleeper.sleep(dur);

            self.step_counter += step_delta;

            self.step.set_low();
            sleeper.sleep(dur);
        }
        self.tx.send(false).expect("send failed false");
        // sleeper.sleep(Duration::from_millis(50));
    }
    pub fn turn_to_cm(&mut self, cm: f32) {
        self.turn_to_step(self.cm_to_steps(cm));
    }

    pub fn turn_to_step(&mut self, step: i64) {
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
        let do_steps_abs = do_steps.abs() as usize;

        let mut c = 0;
        let mut istep;
        let _ = self.tx.send(true);

        while self.step_counter != step {
            let m1 = Instant::now();
            istep = c.min(do_steps_abs - c);

            let (high, low) = self.lookuptable[istep]; // what if istep bigger than len??

            self.step.set_high();
            sleeper.sleep(high - m1.elapsed());

            let m2 = Instant::now();
            self.step_counter += step_delta;

            self.step.set_low();
            sleeper.sleep(low - m2.elapsed());

            c += 1;
        }

        let _ = self.tx.send(false);
        self.canceler.store(false, Ordering::SeqCst);
        println!("time {}ms", start.elapsed().as_millis());
        sleeper.sleep(Duration::from_millis(50));
    }

    pub fn set_step_count(&mut self, steps: i64) {
        self.step_counter = steps;
    }
}
