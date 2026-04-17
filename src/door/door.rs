const DOOR_COOLDOWN: Duration = Duration::from_secs(5);
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Sender, channel},
    },
    thread::{JoinHandle, spawn},
    time::Duration,
};

use rppal::gpio::Gpio;

use crate::door::stepper::Stepper;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Opened,
    Closed,
    Opening,
    Closing,
    Held,
    Locked,
    Undefined,
}
#[derive(Debug, PartialEq)]
pub enum Event {
    Open,
    Close,
    Hold,
    Release,
    IsOpen,
    IsClose,
    Lock,
    Unlock,
    Calibrate,
}
pub struct Door {
    state: Arc<Mutex<State>>,
    stepper: Stepper,
    stepper_cancler: Arc<AtomicBool>,
    door_dog: Option<Sender<()>>,
}
impl Door {
    pub fn new() -> Arc<Mutex<Self>> {
        let lop = Stepper::new(17, 27, 22, 1600, 8.0).unwrap();
        let t = Door {
            state: Arc::new(Mutex::new(State::Undefined)),
            stepper_cancler: lop.get_cancler_clone(),
            stepper: lop,
            door_dog: None,
        };
        let dooro = Arc::new(Mutex::new(t));
        dooro
    }
    fn calibrate(door_arc: Arc<Mutex<Door>>) {
        println!("Start door calibration");
        {
            let mut door = door_arc.lock().unwrap();
            let Door {
                ref mut stepper, ..
            } = *door;
            let close = Gpio::new().unwrap().get(25).unwrap().into_input_pullup(); //0
            let middle = Gpio::new().unwrap().get(23).unwrap().into_input_pullup(); //3209
            let furtherest = Gpio::new().unwrap().get(24).unwrap().into_input_pullup(); //6960

            let first = 157;
            let second = 3409;
            let third = 7722;

            //place door in closed position before running
            if false {
                for _ in 0..2 {
                    stepper.turn_while(|| close.is_low(), 1, 150.0);
                    println!("First: {}", stepper.get_step_count());
                    stepper.turn_while(|| middle.is_high(), 1, 150.0);
                    stepper.turn_while(|| middle.is_low(), 1, 150.0);
                    println!("Second: {}", stepper.get_step_count());
                    stepper.turn_while(|| furtherest.is_high(), 1, 150.0);
                    stepper.turn_while(|| furtherest.is_low(), 1, 150.0);
                    println!("Third: {}", stepper.get_step_count());
                    stepper.turn_to(0);
                }
                return;
            }
            stepper.turn_while(
                || close.is_high() && middle.is_high() && furtherest.is_high(),
                -1,
                150.0,
            );
            if close.is_low() {
                stepper.turn_while(|| close.is_low(), 1, 150.0);
                stepper.set_step_count(first);
            } else if middle.is_low() {
                stepper.turn_while(|| middle.is_low(), 1, 150.0);
                stepper.set_step_count(second);
            } else if furtherest.is_low() {
                stepper.turn_while(|| furtherest.is_low(), 1, 150.0);
                stepper.set_step_count(third);
            }
            stepper.turn_to(second - ((second - first) / 2));
            stepper.turn_while(|| middle.is_high(), 1, 150.0);
            stepper.turn_while(|| middle.is_low(), 1, 150.0);
            stepper.set_step_count(second);
        }
        println!("Finished door calibration");
        Door::close_door(door_arc.clone());
    }
    pub fn get_cancler(&self) -> Arc<AtomicBool> {
        self.stepper_cancler.clone()
    }
    pub fn get_state_arc(&self) -> Arc<Mutex<State>> {
        self.state.clone()
    }
    fn process_event(door_arc: Arc<Mutex<Door>>, event: Event) {
        use Event::*;
        use State::*;
        let state_clone = door_arc.lock().unwrap().get_state_arc();
        let current_state = state_clone.lock().unwrap().clone();
        let dl = || door_arc.lock().unwrap();
        let sl = || state_clone.lock().unwrap();

        match (current_state, event) {
            (Opened | Opening | Closing, Close) => Door::close_door(door_arc),
            (Closed | Closing | Opening, Open) => {
                dl().send_open_signal();
                Door::open_door(door_arc)
            }

            //End postions reached
            (Opening, IsOpen) => {
                *sl() = Opened;
                dl().send_open_signal()
            }
            (Closing, IsClose) => *sl() = Closed,

            (_, Open) => dl().send_open_signal(),

            (Opened, Hold) => *sl() = Held, // Hold Transitions
            (Held, Release) => *sl() = Opened,

            (Closed, Lock) => *sl() = Locked, // Lock Transitions
            (Locked, Unlock) => *sl() = Closed,

            (_, Calibrate) => {
                *sl() = Undefined;
                Door::calibrate(door_arc)
            }

            (_, _) => {} // Total function complete most Events do nothing
        }
    }
    fn open_door(door_arc: Arc<Mutex<Door>>) {
        let condi;
        {
            let mut door = door_arc.lock().unwrap();
            let state_clone = door.get_state_arc();

            *state_clone.lock().unwrap() = State::Opening;
            let open = door.stepper.get_steps(40.0);
            door.stepper.turn_to(open);
            condi = door.stepper.get_step_count() == open;
        }
        if condi {
            Door::process_event(door_arc, Event::IsOpen);
        }
    }
    fn close_door(door_arc: Arc<Mutex<Door>>) {
        let condi;
        {
            let mut door = door_arc.lock().unwrap();
            let state_clone = door.get_state_arc();

            *state_clone.lock().unwrap() = State::Closing;

            door.stepper.turn_to(0);

            condi = door.stepper.get_step_count() == 0;
        }
        if condi {
            Door::process_event(door_arc, Event::IsClose);
        }
    }
    fn send_open_signal(&self) {
        if let Some(dog) = &self.door_dog {
            let _ = dog.send(());
        }
    }

    fn set_watch_dog(&mut self, dog: Sender<()>) {
        self.door_dog = Some(dog)
    }
}
pub fn start_door_controller(door_arc: Arc<Mutex<Door>>) -> Sender<Event> {
    let (tx, rx) = channel::<Event>();

    let (btx, brx) = channel::<()>();
    let tx_clone = tx.clone();
    door_arc.lock().unwrap().set_watch_dog(btx);

    spawn(move || {
        while brx.recv().is_ok() {
            loop {
                match brx.recv_timeout(DOOR_COOLDOWN) {
                    Ok(_) => continue,
                    Err(_) => {
                        let _ = tx_clone.send(Event::Close);
                        break;
                    }
                }
            }
        }
    });

    spawn(move || {
        let mut queue = VecDeque::new();
        let mut thread: Option<JoinHandle<()>> = None;
        let state = door_arc.lock().unwrap().get_state_arc();
        let cancler = door_arc.lock().unwrap().get_cancler();
        let sl = || state.lock().unwrap();
        loop {
            if queue.is_empty() {
                queue.push_back(rx.recv().unwrap());
            }
            while let Ok(event) = rx.try_recv() {
                queue.push_back(event);
            }
            if *sl() == State::Opening {
                if queue.contains(&Event::Close) {
                    cancler.store(true, Ordering::SeqCst);
                    queue.retain(|x| x != &Event::Open);
                }
            }
            if *sl() == State::Closing {
                if queue.contains(&Event::Open) {
                    cancler.store(true, Ordering::SeqCst);
                    queue.retain(|x| x != &Event::Close);
                }
            }

            if thread.is_none() || thread.as_ref().is_some_and(|x| x.is_finished()) {
                if let Some(event) = queue.pop_front() {
                    cancler.store(false, Ordering::SeqCst);
                    println!("Doorstate: {:?} Doorevent: {:?}", *sl(), &event);
                    let door_arc_clone = door_arc.clone();
                    thread = Some(spawn(|| {
                        Door::process_event(door_arc_clone, event);
                    }));
                }
            }
        }
    });

    tx
}
