const DOOR_COOLDOWN: Duration = Duration::from_secs(5);
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Sender, channel},
    },
    thread::{JoinHandle, sleep, spawn},
    time::Duration,
};

use rppal::gpio::{Gpio, InputPin};

use crate::door::stepper::Stepper;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Opened,
    Closed,
    Opening,
    Closing,
    Holding,
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
}
pub struct Door {
    state: Arc<Mutex<State>>,
    stepper: Stepper,
    stepper_cancler: Arc<AtomicBool>,
    close: InputPin,
    door_dog: Option<Sender<()>>,
}
impl Door {
    pub fn new() -> Arc<Mutex<Self>> {
        let lop = Stepper::new(17, 27, 22, 1600).unwrap();
        let t = Door {
            state: Arc::new(Mutex::new(State::Undefined)),
            stepper_cancler: lop.get_cancler_clone(),
            stepper: lop,
            close: Gpio::new().unwrap().get(23).unwrap().into_input_pullup(),
            door_dog: None,
        };
        let dooro = Arc::new(Mutex::new(t));
        Door::calibrate(dooro.clone());
        dooro
    }
    fn calibrate(door_arc: Arc<Mutex<Door>>) {
        println!("Start door calibration");
        {
            let mut door = door_arc.lock().unwrap();
            let Door {
                ref mut stepper,
                ref close,
                ..
            } = *door;

            stepper.turn_while(|| close.is_low(), -1);
            stepper.turn_while(|| close.is_high(), 1);

            sleep(Duration::from_millis(500));
            // dbg!(door.stepper.get_step_count());
            let steps = door.stepper.rot_ref(5110, 1600);
            door.stepper.set_step_count(steps);
        }
        Door::close_door(door_arc.clone());
        println!("Finished door calibration");
    }
    pub fn get_cancler(&self) -> Arc<AtomicBool> {
        self.stepper_cancler.clone()
    }
    pub fn get_state_arc(&self) -> Arc<Mutex<State>> {
        self.state.clone()
    }
    pub fn process_event(door_arc: Arc<Mutex<Door>>, event: Event) {
        use Event::*;
        use State::*;
        let state_clone = door_arc.lock().unwrap().get_state_arc();
        let old_state = state_clone.lock().unwrap().clone();
        match (&old_state, &event) {
            (Opened, Close) => Door::close_door(door_arc),
            (Opening, Close) => Door::close_door(door_arc),
            (Closing, Close) => Door::close_door(door_arc),
            (Opened, Hold) => *state_clone.lock().unwrap() = Holding,
            (Opened, IsClose) => todo!(),
            (Closed, Open) => {
                door_arc.lock().unwrap().send_open_signal();
                Door::open_door(door_arc);
            }
            (Closing, Open) => {
                door_arc.lock().unwrap().send_open_signal();
                Door::open_door(door_arc);
            }
            (Opening, Open) => {
                door_arc.lock().unwrap().send_open_signal();
                Door::open_door(door_arc);
            }
            (Closed, IsOpen) => todo!(),
            // (Opening, Close) => self.close_door(),
            (Opening, IsOpen) => {
                *state_clone.lock().unwrap() = Opened;
                door_arc.lock().unwrap().send_open_signal();
            }
            // (Closing, Open) => self.open_door(),
            (Closing, IsOpen) => todo!(),
            (Closing, IsClose) => *state_clone.lock().unwrap() = Closed,
            (Holding, Release) => *state_clone.lock().unwrap() = Opened,
            (_, Open) => {
                door_arc.lock().unwrap().send_open_signal();
            }
            (Closed, Lock) => *state_clone.lock().unwrap() = Locked,
            (Locked, Unlock) => *state_clone.lock().unwrap() = Closed,
            (_, _) => {}
        }
    }
    fn open_door(door_arc: Arc<Mutex<Door>>) {
        let condi;
        {
            let mut door = door_arc.lock().unwrap();
            let state_clone = door.get_state_arc();

            *state_clone.lock().unwrap() = State::Opening;
            let open = door.stepper.rot_ref(3900, 800);
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

    pub fn set_watch_dog(&mut self, dog: Sender<()>) {
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
        loop {
            if queue.is_empty() {
                queue.push_back(rx.recv().unwrap());
            }
            while let Ok(event) = rx.try_recv() {
                queue.push_back(event);
            }
            if *state.lock().unwrap() == State::Opening {
                if queue.contains(&Event::Close) {
                    cancler.store(true, Ordering::SeqCst);
                    queue.retain(|x| x != &Event::Open);
                }
            }
            if *state.lock().unwrap() == State::Closing {
                if queue.contains(&Event::Open) {
                    cancler.store(true, Ordering::SeqCst);
                    queue.retain(|x| x != &Event::Close);
                }
            }

            if thread.is_none() || thread.as_ref().is_some_and(|x| x.is_finished()) {
                if let Some(event) = queue.pop_front() {
                    cancler.store(false, Ordering::SeqCst);
                    println!(
                        "Doorevent: {:?} Doorstate {:?}",
                        &event,
                        *state.lock().unwrap()
                    );
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
