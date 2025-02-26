pub mod template;
pub mod yasuo;
pub mod riven;

use std::collections::HashMap;
use crate::config::hero::HeroConfig;

pub struct HeroRegistry {
    heroes: HashMap<String, HeroConfig>,
}

impl HeroRegistry {
    pub fn new() -> Self {
        Self {
            heroes: HashMap::new(),
        }
    }
    
    pub fn register_hero(&mut self, name: &str, config: HeroConfig) {
        self.heroes.insert(name.to_string(), config);
    }
    
    pub fn get_hero(&self, name: &str) -> Option<&HeroConfig> {
        self.heroes.get(name)
    }
    
    pub fn get_hero_names(&self) -> Vec<String> {
        self.heroes.keys().cloned().collect()
    }
} 