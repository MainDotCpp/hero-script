use crate::config::hero::{HeroConfig, Key};

pub fn create_empty_config(name: &str) -> HeroConfig {
    let mut config = HeroConfig::new(name);
    
    // 设置切换快捷键
    config.set_hotkey(vec![Key::F1]);
    
    config
} 