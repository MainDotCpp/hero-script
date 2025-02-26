use std::time::{Duration, Instant};

pub struct TimingWindow {
    start_time: Instant,
    duration: Duration,
}

impl TimingWindow {
    pub fn new(duration_ms: u64) -> Self {
        Self {
            start_time: Instant::now(),
            duration: Duration::from_millis(duration_ms),
        }
    }
    
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
    }
    
    pub fn is_expired(&self) -> bool {
        Instant::now().duration_since(self.start_time) > self.duration
    }
    
    pub fn remaining_ms(&self) -> u64 {
        let elapsed = Instant::now().duration_since(self.start_time);
        if elapsed > self.duration {
            return 0;
        }
        (self.duration - elapsed).as_millis() as u64
    }
} 