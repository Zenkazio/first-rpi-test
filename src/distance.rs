#![allow(dead_code)]
use rppal::gpio::{Error, Gpio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const RATE: Duration = Duration::from_micros(200);
pub trait Hcsr04Observer: Send + Sync {
    fn update(&self, value: f64);
}
pub struct Hcsr04 {
    distance: Arc<Mutex<SensorAverager>>,
    observer: Arc<Mutex<Vec<Arc<dyn Hcsr04Observer>>>>,
}

impl Hcsr04 {
    pub fn new(trig_pin: u8, echo_pin: u8) -> Result<Self, Error> {
        let mut trig = Gpio::new()?.get(trig_pin)?.into_output_low();
        let echo = Gpio::new()?.get(echo_pin)?.into_input_pulldown();

        let distance = Arc::new(Mutex::new(SensorAverager::new(10)));
        let distance_clone = distance.clone();

        let observer: Arc<Mutex<Vec<Arc<dyn Hcsr04Observer>>>> = Arc::new(Mutex::new(Vec::new()));
        let observer_clone = observer.clone();

        thread::spawn(move || {
            trig.set_low();
            thread::sleep(Duration::from_secs(2));

            loop {
                trig.set_high();
                thread::sleep(Duration::from_micros(10));
                trig.set_low();

                while echo.is_low() {}
                let t0 = Instant::now();
                while echo.is_high() {}
                let dt = t0.elapsed().as_micros() as f64 / 58.0;

                let mut tmp = distance_clone.lock().unwrap();
                tmp.add(dt);
                for obs in observer_clone.lock().unwrap().iter() {
                    obs.update(tmp.average());
                }
                drop(tmp);
                thread::sleep(RATE);
            }
        });
        Ok(Hcsr04 { distance, observer })
    }

    pub fn get_distance(&self) -> f64 {
        self.distance.lock().unwrap().average()
    }
    pub fn add_observer(&self, observer: Arc<dyn Hcsr04Observer>) {
        self.observer.lock().unwrap().push(observer);
    }
}

use std::collections::VecDeque;

struct SensorAverager {
    values: VecDeque<f64>,
    capacity: usize,
}

impl SensorAverager {
    fn new(capacity: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn add(&mut self, value: f64) {
        if self.values.len() == self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    fn average(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            let sum: f64 = self.values.iter().sum();
            sum / self.values.len() as f64
        }
    }
}
