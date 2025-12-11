use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// The simple-signal crate is used to handle incoming signals.
use simple_signal::{self, Signal};

use rppal::gpio::{Event, Gpio, Trigger};

use crate::distance::Hcsr04;
use crate::pins::*;
use crate::rgb_swappper::RBGSwapper;
use crate::rgbled::RGBLed;
use crate::servo::Servo;
mod distance;
mod keypad;
mod pins;
mod rgb_swappper;
mod rgbled;
mod servo;
// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const STOP_AFTER_N_CHANGES: u8 = 5;

fn input_callback(event: Event, my_data: Arc<Mutex<u8>>) {
    println!("Event: {:?}", event);
    *my_data.lock().unwrap() += 1;
}

fn main() -> Result<(), Box<dyn Error>> {
    let rgb_led = Arc::new(Mutex::new(RGBLed::new(
        RGB_LED_RED,
        RGB_LED_GREEN,
        RGB_LED_BLUE,
    )?));
    let mut red = Gpio::new()?.get(RED_LED)?.into_output_low();

    let running = Arc::new(AtomicBool::new(true));
    simple_signal::set_handler(&[Signal::Int, Signal::Term], {
        let running = running.clone();
        move |_| {
            running.store(false, Ordering::SeqCst);
        }
    });
    let shared_state = Arc::new(Mutex::new(0));

    let mut input_pin = Gpio::new()?.get(INPUT_BUTTON)?.into_input_pulldown();
    let shared_state_hold = shared_state.clone();
    input_pin.set_async_interrupt(
        Trigger::FallingEdge,
        Some(Duration::from_millis(50)),
        move |event| {
            // Note: you could add more parameters here!
            input_callback(event, shared_state_hold.clone());
        },
    )?;

    // let s = RBGSwapper::new(rgb_led.clone());
    // let sensor = Hcsr04::new(TRIG, ECHO)?;
    // let observer = Arc::new(s);
    // sensor.add_observer(observer);

    let servo = Arc::new(Mutex::new(Servo::new(SERVO_PULS)?));
    let servo_clone = servo.clone();
    let rgb_led_clone = rgb_led.clone();
    let shared_state_hold2 = shared_state.clone();
    thread::sleep(Duration::from_millis(1000));
    //---------------------------------------------------------------------------------------------

    red.set_high();
    rgb_led.lock().unwrap().green()?;

    thread::spawn(move || {
        loop {
            if *shared_state_hold2.lock().unwrap() == 2 {
                rgb_led_clone.lock().unwrap().green().unwrap();
                servo_clone.lock().unwrap().set_degree(0);
                servo_clone.lock().unwrap().set_degree(90);
                servo_clone.lock().unwrap().set_degree(180);
                servo_clone.lock().unwrap().set_degree(90);

                thread::sleep(Duration::from_millis(1000));

                servo_clone.lock().unwrap().set_degree(0);
                servo_clone.lock().unwrap().set_degree(180);
                servo_clone.lock().unwrap().set_degree(0);

                thread::sleep(Duration::from_millis(1000));

                servo_clone.lock().unwrap().set_degree(45);
                servo_clone.lock().unwrap().set_degree(0);

                thread::sleep(Duration::from_millis(1000));

                servo_clone.lock().unwrap().set_degree(135);
                servo_clone.lock().unwrap().set_degree(0);
            }
            rgb_led_clone.lock().unwrap().red().unwrap();
            thread::sleep(Duration::from_secs(3));
        }
    });
    // -----------------------------------------------------
    while running.load(Ordering::SeqCst) {
        if *shared_state.lock().unwrap() >= STOP_AFTER_N_CHANGES {
            println!("Reached {STOP_AFTER_N_CHANGES} events, exiting...");
            *shared_state.lock().unwrap() = 0;
        }
        red.toggle();
        // println!("{:.2}", sensor.get_distance());
        thread::sleep(Duration::from_millis(200));
    }
    red.set_low();
    // rgb_led.clear()?;
    Ok(())
}
#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(true, true);
    }
}
