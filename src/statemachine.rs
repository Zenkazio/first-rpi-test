use std::sync::{Arc, Mutex};
trait StateBehaviour {
    fn on_enter(&self) {}
    fn on_exit(&self) {}
}
enum State {
    Close,
    Medium,
    Large,
    OutOfRange,
    Initializing,
}

impl StateBehaviour for State {
    fn on_enter(&self) {
        use State::*;
    }
}

struct StateMachine {
    state: Arc<Mutex<State>>,
}

impl StateMachine {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State::Initializing)),
        }
    }
    fn handle_event(&self, distance: f64) {
        let mut state = self.state.lock().unwrap();
        use State::*;

        *state = match (&*state, distance) {
            (_, _) => {}
        }
    }
}
