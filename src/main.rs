use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// The simple-signal crate is used to handle incoming signals.
use simple_signal::{self, Signal};

use rppal::gpio::{Event, Gpio, Trigger};

use crate::i2c::{I2CMaster, MPU6050};
use crate::pins::*;
use crate::rgbled::RGBLed;
use crate::servo::Servo;
use crate::stepper::Stepper;
mod distance;
mod door_statemachine;
mod i2c;
mod keypad;
mod led_stripe;
mod pins;
mod rgb_swappper;
mod rgbled;
mod servo;
mod stepper;
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

    let mut stepper = Stepper::new(KEYPAD_OUT1, KEYPAD_OUT2, KEYPAD_OUT3, KEYPAD_OUT4)?;

    let servo = Arc::new(Mutex::new(Servo::new(SERVO_PULS)?));
    let servo_clone = servo.clone();
    let rgb_led_clone = rgb_led.clone();
    let shared_state_hold2 = shared_state.clone();
    // thread::sleep(Duration::from_millis(1000));

    let i2c_master = Arc::new(Mutex::new(I2CMaster::new()?));
    let mut mpu6050 = MPU6050::new(i2c_master);
    mpu6050.wake_sensor();
    //---------------------------------------------------------------------------------------------

    red.set_high();
    rgb_led.lock().unwrap().green()?;
    thread::spawn(move || {
        stepper.one_rotation();
        stepper.clear();
    });

    thread::spawn(move || {
        loop {
            if *shared_state_hold2.lock().unwrap() == 2 {
                rgb_led_clone.lock().unwrap().green().unwrap();
                servo_clone.lock().unwrap().set_degree(0);
                servo_clone.lock().unwrap().set_degree(90);
                servo_clone.lock().unwrap().set_degree(100);
                servo_clone.lock().unwrap().set_degree(90);

                thread::sleep(Duration::from_millis(1000));

                servo_clone.lock().unwrap().set_degree(0);
                servo_clone.lock().unwrap().set_degree(100);
                servo_clone.lock().unwrap().set_degree(0);

                thread::sleep(Duration::from_millis(1000));

                servo_clone.lock().unwrap().set_degree(45);
                servo_clone.lock().unwrap().set_degree(0);

                thread::sleep(Duration::from_millis(1000));

                servo_clone.lock().unwrap().set_degree(100);
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
        // println!("{:.2}", mpu6050.get_temperature());
        // println!("{:.2}", sensor.get_distance());
        thread::sleep(Duration::from_millis(1000));
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
