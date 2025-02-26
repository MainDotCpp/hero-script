use std::time::Duration;
use std::thread;
use log::{debug, error};
use enigo::{Enigo, Key as EnigoKey, KeyboardControllable, MouseControllable, MouseButton as EnigoMouseButton};
use crate::config::hero::{Key, KeyAction, MouseButton};

pub struct KeyboardSimulator {
    enigo: Enigo,
}

impl KeyboardSimulator {
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(),
        }
    }
    
    pub fn execute_actions(&mut self, actions: &[KeyAction]) {
        debug!("执行键盘动作: {:?}", actions);
        
        for action in actions {
            match action {
                KeyAction::Press(key) => {
                    self.press_key(key);
                    // 添加短暂延迟确保按键被识别
                    thread::sleep(Duration::from_millis(50));
                },
                &KeyAction::Delay(ms) => {
                    debug!("延迟 {}ms", ms);
                    thread::sleep(Duration::from_millis(ms as u64));
                },
                &KeyAction::MouseClick(x, y, ref button) => {
                    self.mouse_click(x, y, button);
                },
                KeyAction::Down(key) => {
                    debug!("只按下键: {:?}", key);
                    self.press_key(key);
                },
                KeyAction::Up(key) => {
                    debug!("只释放键: {:?}", key);
                    self.release_key(key);
                },
            }
        }
        
        debug!("键盘动作执行完成");
    }
    
    fn press_key(&mut self, key: &Key) {
        debug!("按下键: {:?}", key);
        match self.key_to_enigo_key(key) {
            Some(enigo_key) => {
                self.enigo.key_down(enigo_key);
            },
            None => {
                error!("无法映射按键: {:?}", key);
            }
        }
    }
    
    fn release_key(&mut self, key: &Key) {
        debug!("释放键: {:?}", key);
        match self.key_to_enigo_key(key) {
            Some(enigo_key) => {
                self.enigo.key_up(enigo_key);
            },
            None => {
                error!("无法映射按键: {:?}", key);
            }
        }
    }
    
    fn mouse_click(&mut self, x: i32, y: i32, button: &MouseButton) {
        // 保存当前鼠标位置
        let (curr_x, curr_y) = self.enigo.mouse_location();
        
        // 移动鼠标
        self.enigo.mouse_move_to(x, y);
        
        // 点击
        match button {
            MouseButton::Left => self.enigo.mouse_click(EnigoMouseButton::Left),
            MouseButton::Right => self.enigo.mouse_click(EnigoMouseButton::Right),
            MouseButton::Middle => self.enigo.mouse_click(EnigoMouseButton::Middle),
        }
        
        // 移回原位置
        self.enigo.mouse_move_to(curr_x, curr_y);
        
        debug!("模拟鼠标点击: 位置({}, {}), 按钮: {:?}", x, y, button);
    }
    
    fn key_to_enigo_key(&self, key: &Key) -> Option<EnigoKey> {
        match key {
            Key::Character(c) => {
                match c {
                    'a'..='z' | 'A'..='Z' | '0'..='9' => Some(EnigoKey::Layout(*c)),
                    _ => None,
                }
            },
            Key::F1 => Some(EnigoKey::F1),
            Key::F2 => Some(EnigoKey::F2),
            Key::F3 => Some(EnigoKey::F3),
            Key::F4 => Some(EnigoKey::F4),
            Key::F5 => Some(EnigoKey::F5),
            Key::F6 => Some(EnigoKey::F6),
            Key::F7 => Some(EnigoKey::F7),
            Key::F8 => Some(EnigoKey::F8),
            Key::F9 => Some(EnigoKey::F9),
            Key::F10 => Some(EnigoKey::F10),
            Key::F11 => Some(EnigoKey::F11),
            Key::F12 => Some(EnigoKey::F12),
            Key::Escape => Some(EnigoKey::Escape),
            Key::Tab => Some(EnigoKey::Tab),
            Key::CapsLock => Some(EnigoKey::CapsLock),
            Key::Shift => Some(EnigoKey::Shift),
            Key::Control => Some(EnigoKey::Control),
            Key::Alt => Some(EnigoKey::Alt),
            Key::Space => Some(EnigoKey::Space),
            Key::Enter => Some(EnigoKey::Return),
        }
    }
} 