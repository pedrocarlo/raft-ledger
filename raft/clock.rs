pub trait Clock {
    fn now(&mut self) -> std::time::Instant;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&mut self) -> std::time::Instant {
        std::time::Instant::now()
    }
}
