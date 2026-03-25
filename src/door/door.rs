const DOOR_COOLDOWN: Duration = Duration::from_secs(5);
use std::{
    sync::{
        Arc,
        atomic::AtomicBool,
        mpsc::{Sender, channel},
    },
    thread::{sleep, spawn},
    time::Duration,
};

use rppal::gpio::{Gpio, InputPin};

use crate::door::stepper::Stepper;

#[derive(Debug)]
enum State {
    Opened,
    Closed,
    Opening,
    Closing,
    Holding,
}
#[derive(Debug)]
pub enum Event {
    Open,
    Close,
    Hold,
    Unhold,
    IsOpen,
    IsClose,
}
pub struct Door {
    state: State,
    stepper: Stepper,
    stepper_cancler: Arc<AtomicBool>,
    close: InputPin,
    door_dog: Option<Sender<()>>,
}
impl Door {
    pub fn new() -> Self {
        let lop = Stepper::new(17, 27, 22, 1600).unwrap();
        let mut t = Door {
            state: State::Closed,
            stepper_cancler: lop.get_cancler_clone(),
            stepper: lop,
            close: Gpio::new().unwrap().get(23).unwrap().into_input_pullup(),
            door_dog: None,
        };
        t.calibrate();
        t
    }
    fn calibrate(&mut self) {
        println!("Start door calibration");

        self.stepper.turn_while(|| self.close.is_low(), -1);
        self.stepper.turn_while(|| self.close.is_high(), 1);

        sleep(Duration::from_millis(500));
        self.stepper
            .set_step_count(self.stepper.rot_ref(4697, 1600));
        self.close_door();
        println!("Finished door calibration");
    }
    pub fn get_cancler(&self) -> Arc<AtomicBool> {
        self.stepper_cancler.clone()
    }
    pub fn process_event(&mut self, event: Event) {
        use Event::*;
        use State::*;

        match (&self.state, &event) {
            (Opened, Close) => self.close_door(),
            (Opened, Hold) => self.state = Holding,
            (Opened, IsClose) => todo!(),
            (Closed, Open) => {
                self.send_open_signal();
                self.open_door();
            }
            (Closed, IsOpen) => todo!(),
            // (Opening, Close) => self.close_door(),
            (Opening, IsOpen) => {
                self.state = Opened;
                self.send_open_signal();
            }
            // (Closing, Open) => self.open_door(),
            (Closing, IsOpen) => todo!(),
            (Closing, IsClose) => self.state = Closed,
            (Holding, Unhold) => self.state = Opened,
            (_, Open) => {
                self.send_open_signal();
            }
            (_, _) => {}
        }
    }
    fn open_door(&mut self) {
        self.state = State::Opening;
        let open = self.stepper.rot_ref(4300, 800);
        self.stepper.turn_to(open);

        if self.stepper.get_step_count() == open {
            self.process_event(Event::IsOpen);
        }
    }
    fn close_door(&mut self) {
        self.state = State::Closing;

        self.stepper.turn_to(0);

        if self.stepper.get_step_count() == 0 {
            self.process_event(Event::IsClose);
        }
    }
    fn send_open_signal(&self) {
        if let Some(dog) = &self.door_dog {
            let _ = dog.send(());
        }
    }

    pub fn set_watch_dog(&mut self, dog: Sender<()>) {
        self.door_dog = Some(dog)
    }
}
pub fn start_door_controller(mut door: Door) -> Sender<Event> {
    let (tx, rx) = channel::<Event>();

    let (btx, brx) = channel::<()>();
    let tx_clone = tx.clone();
    door.set_watch_dog(btx);

    spawn(move || {
        while brx.recv().is_ok() {
            loop {
                match brx.recv_timeout(DOOR_COOLDOWN) {
                    Ok(_) => continue,
                    Err(_) => {
                        let _ = tx_clone.send(Event::Close);
                        // Code ausführen
                        break;
                    }
                }
            }
        }
    });

    spawn(move || {
        for event in rx {
            println!("Doorevent: {:?}", &event);
            door.process_event(event);
        }
    });

    tx
}
