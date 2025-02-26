use crate::config::hero::{ComboTrigger, ComboAction};
use std::time::{Duration, Instant};

pub struct ComboTracker {
    pub name: String,
    pub trigger: ComboTrigger,
    pub action: ComboAction,
    pub last_key_time: Instant,
}

impl ComboTracker {
    pub fn new(name: &str, trigger: ComboTrigger, action: ComboAction) -> Self {
        Self {
            name: name.to_string(),
            trigger,
            action,
            last_key_time: Instant::now(),
        }
    }
    
    pub fn update_time(&mut self) {
        self.last_key_time = Instant::now();
    }
    
    pub fn is_expired(&self, timeout: Duration) -> bool {
        Instant::now().duration_since(self.last_key_time) > timeout
    }
} 