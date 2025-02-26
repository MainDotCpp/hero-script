use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboAction {
    /// 要模拟的按键序列
    pub keys: Vec<KeyAction>,
    /// 是否阻止原始按键继续传递
    pub block_original: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyAction {
    /// 按下并松开一个键
    Press(Key),
    /// 按下一个键
    Down(Key),
    /// 松开一个键
    Up(Key),
    /// 等待指定毫秒
    Delay(u64),
    /// 鼠标点击(坐标偏移, 按钮)
    MouseClick(i32, i32, MouseButton),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Key {
    Character(char),
    // 各种特殊按键...
    Escape, Tab, CapsLock, Shift, Control, Alt, Space, Enter,
    // 数字键和功能键...
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // 其他按键...
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComboTrigger {
    /// 触发连招的按键序列
    pub sequence: Vec<Key>,
    /// 序列中的按键必须在此时间窗口内完成 (毫秒)
    pub time_window: Option<u64>,
    /// 如果设置，这个连招将在这些键被屏蔽的情况下触发
    pub block_keys: HashSet<Key>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroConfig {
    /// 英雄名称
    pub name: String,
    /// 英雄的连招配置
    pub combos: HashMap<String, (ComboTrigger, ComboAction)>,
    /// 切换到此英雄的快捷键
    pub hotkey: Option<Vec<Key>>,
}

impl HeroConfig {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            combos: HashMap::new(),
            hotkey: None,
        }
    }
    
    pub fn add_combo(
        &mut self, 
        name: &str, 
        sequence: Vec<Key>, 
        actions: Vec<KeyAction>, 
        time_window: Option<u64>,
        block_original: bool,
        block_keys: HashSet<Key>,
    ) {
        let trigger = ComboTrigger {
            sequence,
            time_window,
            block_keys,
        };
        
        let action = ComboAction {
            keys: actions,
            block_original,
        };
        
        self.combos.insert(name.to_string(), (trigger, action));
    }
    
    pub fn set_hotkey(&mut self, keys: Vec<Key>) {
        self.hotkey = Some(keys);
    }
} 