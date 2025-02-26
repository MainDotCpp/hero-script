use std::collections::HashSet;
use crate::config::hero::{HeroConfig, Key, KeyAction};

pub fn create_config() -> HeroConfig {
    let mut config = HeroConfig::new("yasuo");
    
    // 注意: 亚索的连招是E+R和E+D，不是E+Q和E+D
    // 添加详细注释，确保理解连招机制
    
    // 添加E+R连招: 当按下E后150ms内按R时，执行EQR连招并屏蔽R
    let mut block_r = HashSet::new();
    block_r.insert(Key::Character('r'));
    config.add_combo(
        "eqr_combo",
        vec![Key::Character('e'), Key::Character('r')], // 触发条件：E后R
        vec![
            KeyAction::Press(Key::Character('q')),      // 按下Q
            KeyAction::Press(Key::Character('r')),      // 按下R
        ],
        Some(150), // 150ms时间窗口：在按E后150ms内按R才会触发
        true,      // 屏蔽原始的R键
        block_r,   // 屏蔽的按键列表
    );
    
    // 添加E+D连招: 当按下E后150ms内按D时，执行EQD连招并屏蔽D
    let mut block_d = HashSet::new();
    block_d.insert(Key::Character('d'));
    config.add_combo(
        "eqd_combo",
        vec![Key::Character('e'), Key::Character('d')], // 触发条件：E后D
        vec![
            KeyAction::Press(Key::Character('q')),      // 按下Q
            KeyAction::Press(Key::Character('d')),      // 按下D
        ],
        Some(300), // 将时间窗口扩大到300ms，更容易触发
        true,      // 屏蔽原始的D键
        block_d,   // 屏蔽的按键列表
    );
    
    // 设置切换快捷键
    config.set_hotkey(vec![Key::F1]);
    config
} 