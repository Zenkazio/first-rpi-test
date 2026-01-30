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
}
impl Door {
    pub fn new() -> Self {
        let mut t = Door {
            state: State::Closed,
        };
        t.process_event(Event::Open);
        t
    }
    pub fn process_event(&mut self, event: Event) {
        use Event::*;
        use State::*;

        match (&self.state, &event) {
            (Opened, Open) => {}
            (Opened, Close) => self.close_door(),
            (Opened, Hold) => self.state = Holding,
            (Closed, Open) => self.open_door(),
            (Closed, Close) => {}
            (Closed, Hold) => {}
            (Opening, Open) => {}
            (Opening, Close) => self.close_door(),
            (Opening, Hold) => {}
            (Closing, Open) => self.open_door(),
            (Closing, Close) => {}
            (Closing, Hold) => {}
            (Holding, Open) => {}
            (Holding, Close) => {}
            (Holding, Hold) => {}
            (Opened, IsOpen) => {}
            (Opened, IsClose) => panic!("something is wrong while Opened"),
            (Closed, IsOpen) => panic!("something is wrong while Closed"),
            (Closed, IsClose) => {}
            (Opening, IsOpen) => self.state = Opened,
            (Opening, IsClose) => panic!("something is wrong while Opening"),
            (Closing, IsOpen) => panic!("something is wrong while Closing is Open"),
            (Closing, IsClose) => self.state = Closed,
            (Holding, IsOpen) => {}
            (Holding, IsClose) => panic!("something is wrong while Holding isclose"),
            (Opened, Unhold) => {}
            (Closed, Unhold) => {}
            (Opening, Unhold) => {}
            (Closing, Unhold) => {}
            (Holding, Unhold) => self.state = Opened,
        }
    }
    fn open_door(&mut self) {
        self.state = State::Opening;
        //do whatever nec to open door
    }
    fn close_door(&mut self) {
        self.state = State::Closing;
        //actual closing
    }
}
