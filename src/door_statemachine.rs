#![allow(unused)]
#[derive(Debug)]
enum State {
    Opened,
    Closed,
    Opening,
    Closing,
}
pub enum Event {
    Open,
    Close,
}
struct Door {
    state: State,
}
impl Door {
    fn process_event(&mut self, event: Event) {
        use Event::*;
        use State::*;

        match (&self.state, event) {
            (Opened, Open) => {}
            (Opened, Close) => self.state = Closed,
            (Closed, Open) => self.state = Opened,
            (Closed, Close) => {}
            (Opening, Open) => {}
            (Opening, Close) => {}
            (Closing, Open) => {}
            (Closing, Close) => {}
        }
    }
}

// struct HoldOpened;
// struct Closing;
// struct Opening;
// struct Locking;
// struct Locked;
// struct Unlocking;
