use std::collections::HashSet;
use crate::config::hero::{HeroConfig, Key, KeyAction};

pub fn create_config() -> HeroConfig {
    let mut config = HeroConfig::new("yasuo");
    // 添加新连招1: 当按下E后150ms内按R时，执行EQR连招并屏蔽R
    let mut block_r = HashSet::new();
    block_r.insert(Key::Character('r'));
    config.add_combo(
        "eqr_combo",
        vec![Key::Character('e'), Key::Character('r')],
        vec![
            KeyAction::Press(Key::Character('q')),
            KeyAction::Press(Key::Character('r')),
        ],
        Some(150), // 150ms时间窗口
        true,     // 屏蔽原始按键
        block_r,
    );
    
    // 添加新连招2: 当按下E后150ms内按D时，执行EQD连招并屏蔽D
    let mut block_d = HashSet::new();
    block_d.insert(Key::Character('d'));
    config.add_combo(
        "eqd_combo",
        vec![Key::Character('e'), Key::Character('d')],
        vec![
            KeyAction::Press(Key::Character('q')),
            KeyAction::Press(Key::Character('d')),
        ],
        Some(150), // 150ms时间窗口
        true,     // 屏蔽原始按键
        block_d,
    );
    
    // 设置切换快捷键
    config.set_hotkey(vec![Key::F1]);
    config
} 