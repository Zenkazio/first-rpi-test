use std::sync::{Arc, atomic::AtomicBool};

use crate::door::stepper::Stepper;

#[derive(Debug)]
enum State {
    Opened,
    Closed,
    Opening,
    Closing,
    Holding,
}
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
}
impl Door {
    pub fn new() -> Self {
        let lop = Stepper::new(17, 27, 22).unwrap();
        let t = Door {
            state: State::Closed,
            stepper_cancler: lop.get_cancler_clone(),
            stepper: lop,
        };
        t
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
            (Closed, Open) => self.open_door(),
            (Closed, IsOpen) => todo!(),
            (Opening, Close) => self.close_door(),
            (Opening, IsOpen) => self.state = Opened,
            (Closing, Open) => self.open_door(),
            (Closing, IsOpen) => todo!(),
            (Closing, IsClose) => self.state = Closed,
            (Holding, Unhold) => self.state = Opened,
            (_, _) => {}
        }
    }
    fn open_door(&mut self) {
        self.state = State::Opening;

        self.stepper.turn_to(10000);

        if self.stepper.get_step_count() == 10000 {
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
}
