use std::collections::HashSet;
use crate::config::hero::{HeroConfig, Key, KeyAction, MouseButton};

pub fn create_config() -> HeroConfig {
    let mut config = HeroConfig::new("riven");
    
    // 设置Q取消连招
    config.add_combo(
        "q_cancel",
        vec![Key::Character('q')],
        vec![
            KeyAction::Press(Key::Character('q')),
            KeyAction::Delay(50),
            // 移动鼠标取消动画
            KeyAction::MouseClick(10, 0, MouseButton::Right),
        ],
        None,     // 无时间窗口限制
        false,    // 不屏蔽原始按键
        HashSet::new(),
    );
    
    // 设置切换快捷键
    config.set_hotkey(vec![Key::F2]);
    
    config
} 