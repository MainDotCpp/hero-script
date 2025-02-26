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
        for action in actions {
            match action {
                KeyAction::Press(key) => {
                    self.press_key(key);
                },
                KeyAction::Down(key) => {
                    self.key_down(key);
                },
                KeyAction::Up(key) => {
                    self.key_up(key);
                },
                KeyAction::Delay(ms) => {
                    thread::sleep(Duration::from_millis(*ms));
                },
                KeyAction::MouseClick(x, y, button) => {
                    self.mouse_click(*x, *y, button);
                },
            }
        }
    }
    
    fn press_key(&mut self, key: &Key) {
        let enigo_key = self.convert_key(key);
        self.enigo.key_click(enigo_key);
        debug!("模拟按键: {:?}", key);
    }
    
    fn key_down(&mut self, key: &Key) {
        let enigo_key = self.convert_key(key);
        self.enigo.key_down(enigo_key);
        debug!("模拟按下: {:?}", key);
    }
    
    fn key_up(&mut self, key: &Key) {
        let enigo_key = self.convert_key(key);
        self.enigo.key_up(enigo_key);
        debug!("模拟松开: {:?}", key);
    }
    
    fn mouse_click(&mut self, x: i32, y: i32, button: &MouseButton) {
        // 保存当前鼠标位置
        let (curr_x, curr_y) = self.enigo.mouse_location();
        
        // 移动鼠标
        self.enigo.mouse_move_to(curr_x + x, curr_y + y);
        
        // 点击
        match button {
            MouseButton::Left => self.enigo.mouse_click(EnigoMouseButton::Left),
            MouseButton::Right => self.enigo.mouse_click(EnigoMouseButton::Right),
            MouseButton::Middle => self.enigo.mouse_click(EnigoMouseButton::Middle),
        }
        
        // 移回原位置
        self.enigo.mouse_move_to(curr_x, curr_y);
        
        debug!("模拟鼠标点击: 偏移({}, {}), 按钮: {:?}", x, y, button);
    }
    
    fn convert_key(&self, key: &Key) -> EnigoKey {
        match key {
            Key::Character(c) => EnigoKey::Layout(*c),
            Key::Escape => EnigoKey::Escape,
            Key::Tab => EnigoKey::Tab,
            Key::CapsLock => EnigoKey::CapsLock,
            Key::Shift => EnigoKey::Shift,
            Key::Control => EnigoKey::Control,
            Key::Alt => EnigoKey::Alt,
            Key::Space => EnigoKey::Space,
            Key::Enter => EnigoKey::Return,
            // 功能键
            Key::F1 => EnigoKey::F1,
            Key::F2 => EnigoKey::F2,
            // 更多按键映射...
            _ => {
                error!("未实现的按键类型: {:?}", key);
                EnigoKey::Layout('\0')
            }
        }
    }
} 