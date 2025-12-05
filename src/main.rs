use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

// The simple-signal crate is used to handle incoming signals.
use simple_signal::{self, Signal};

use rppal::gpio::Gpio;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 23;

fn main() -> Result<(), Box<dyn Error>> {
    // Retrieve the GPIO pin and configure it as an output.
    let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();
    let mut r_pin = Gpio::new()?.get(17)?.into_output();
    let mut g_pin = Gpio::new()?.get(27)?.into_output();
    let mut b_pin = Gpio::new()?.get(22)?.into_output();
    // r_pin.set_pwm_frequency(1000.0, 0 as f64 / 255 as f64)?;
    // g_pin.set_pwm_frequency(1000.0, 255 as f64 / 255 as f64)?;
    // b_pin.set_pwm_frequency(1000.0, 0 as f64 / 255 as f64)?;
    r_pin.set_low();
    g_pin.set_high();
    b_pin.set_low();

    let running = Arc::new(AtomicBool::new(true));

    // When a SIGINT (Ctrl-C) or SIGTERM signal is caught, atomically set running to false.
    simple_signal::set_handler(&[Signal::Int, Signal::Term], {
        let running = running.clone();
        move |_| {
            running.store(false, Ordering::SeqCst);
        }
    });

    // Blink the LED until running is set to false.
    while running.load(Ordering::SeqCst) {
        pin.toggle();
        thread::sleep(Duration::from_millis(500));
    }

    // After we're done blinking, turn the LED off.
    pin.set_low();

    Ok(())

    // When the pin variable goes out of scope, the GPIO pin mode is automatically reset
    // to its original value, provided reset_on_drop is set to true (default).
}
#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(true, true);
    }
}
