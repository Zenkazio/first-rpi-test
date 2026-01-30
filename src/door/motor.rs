pub trait MotorObserver: Send {
    fn on_step(&mut self, step: i64);
}
