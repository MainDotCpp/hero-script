use crate::config::hero::Key;
use std::collections::HashMap;

pub struct KeyEventProcessor {
    key_states: HashMap<Key, bool>,
}

impl KeyEventProcessor {
    pub fn new() -> Self {
        Self {
            key_states: HashMap::new(),
        }
    }
    
    pub fn set_key_state(&mut self, key: Key, is_down: bool) {
        self.key_states.insert(key, is_down);
    }
    
    pub fn is_key_down(&self, key: &Key) -> bool {
        *self.key_states.get(key).unwrap_or(&false)
    }
    
    pub fn are_keys_down(&self, keys: &[Key]) -> bool {
        keys.iter().all(|key| self.is_key_down(key))
    }
} 