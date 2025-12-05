use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

// The simple-signal crate is used to handle incoming signals.
use simple_signal::{self, Signal};

use rppal::gpio::Gpio;

use crate::keypad::Keypad;
use crate::rgbled::RGBLed;
mod keypad;
mod rgbled;
// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_LED: u8 = 23;

fn main() -> Result<(), Box<dyn Error>> {
    // Retrieve the GPIO pin and configure it as an output.
    let mut rgb_led = RGBLed::new(17, 27, 22)?;
    let mut pin = Gpio::new()?.get(GPIO_LED)?.into_output();
    let mut keypad = Keypad::new(10, 9, 11, 0, 25, 8, 7, 1)?;

    // rgb_led.set_rgb(226, 34, 120)?;
    rgb_led.green()?;

    let running = Arc::new(AtomicBool::new(true));

    // When a SIGINT (Ctrl-C) or SIGTERM signal is caught, atomically set running to false.
    simple_signal::set_handler(&[Signal::Int, Signal::Term], {
        let running = running.clone();
        move |_| {
            running.store(false, Ordering::SeqCst);
        }
    });
    let mut r;
    let mut g;
    let mut b;
    // Blink the LED until running is set to false.
    while running.load(Ordering::SeqCst) {
        pin.toggle();
        keypad.cycle()?;
        r = 0;
        g = 0;
        b = 0;
        if keypad.state[0][0] {
            r += 64;
        }
        if keypad.state[0][1] {
            r += 64;
        }
        if keypad.state[0][2] {
            r += 64;
        }
        if keypad.state[0][3] {
            r += 63;
        }

        if keypad.state[1][0] {
            g += 64;
        }
        if keypad.state[1][1] {
            g += 64;
        }
        if keypad.state[1][2] {
            g += 64;
        }
        if keypad.state[1][3] {
            g += 63;
        }

        if keypad.state[2][0] {
            b += 64;
        }
        if keypad.state[2][1] {
            b += 64;
        }
        if keypad.state[2][2] {
            b += 64;
        }
        if keypad.state[3][3] {
            b += 63;
        }

        rgb_led.set_rgb(r, g, b)?;
        thread::sleep(Duration::from_millis(10));
        // keypad.display_state();
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
