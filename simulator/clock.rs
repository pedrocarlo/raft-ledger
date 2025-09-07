use raft::clock::Clock;

pub struct SimClock;

impl Clock for SimClock {
    fn now(&mut self) -> std::time::Instant {
        std::time::Instant::now()
    }
}

// TODO: for now just just do Instant Now
