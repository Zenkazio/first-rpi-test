use std::{
    os::linux::raw::stat,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use rppal::gpio::{Error, Event, Gpio, InputPin, OutputPin, Trigger};

struct Distance {
    trigger: OutputPin,
    echo: InputPin,
    distance: Arc<Mutex<f64>>,
    start_time: Arc<Mutex<Instant>>,
}

impl Distance {
    fn new(trigger_pin: u8, echo_pin: u8) -> Result<Self, Error> {
        let mut echo = Gpio::new()?.get(echo_pin)?.into_input_pulldown();
        let mut trigger = Gpio::new()?.get(trigger_pin)?.into_output();
        let distance = Arc::new(Mutex::new(-1.0));
        let start_time = Arc::new(Mutex::new(Instant::now()));
        let dis = Distance {
            trigger: trigger,
            echo: echo,
            distance: distance,
            start_time: start_time,
        };
        let start_time_hold1 = start_time.clone();
        echo.set_async_interrupt(
            Trigger::RisingEdge,
            Some(Duration::from_millis(20)),
            move |_| Distance::write_start_time(start_time_hold1.clone()),
        );
        let start_time_hold2 = start_time.clone();
        let distance_hold = distance.clone();
        echo.set_async_interrupt(Trigger::FallingEdge, Duration::from_millis(20), move |_| {
            Distance::write_distance(start_time_hold2.clone(), distance_hold.clone())
        });
        Ok(dis)
    }
    fn time_to_distance() -> f64 {
        todo!();
    }
    fn write_start_time(start_time: Arc<Mutex<Instant>>) {
        *start_time.lock().unwrap() = Instant::now();
    }
    fn write_distance(start_time: Arc<Mutex<Instant>>, distance: Arc<Mutex<f64>>) {
        todo!()
    }
}
