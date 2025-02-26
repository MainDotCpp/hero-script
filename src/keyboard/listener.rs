use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use log::{debug, error, info};
use winapi::um::winuser;
use winapi::shared::windef::HWND;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::macro_engine::MacroEngine;
use crate::config::hero::Key;
use std::sync::mpsc::{channel, Sender, Receiver};

// 低级键盘钩子使用的全局变量
thread_local! {
    static HOOK_HANDLE: Cell<Option<winapi::shared::minwindef::HHOOK>> = Cell::new(None);
    static MACRO_ENGINE: Cell<Option<Arc<MacroEngine>>> = Cell::new(None);
}

static HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

pub struct KeyboardListener {
    macro_engine: Arc<MacroEngine>,
    key_sender: Mutex<Option<Sender<(Key, bool)>>>,
}

impl KeyboardListener {
    pub fn new(macro_engine: Arc<MacroEngine>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            macro_engine,
            key_sender: Mutex::new(None),
        })
    }
    
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 创建通道用于消息传递
        let (sender, receiver) = channel();
        {
            let mut key_sender = self.key_sender.lock().unwrap();
            *key_sender = Some(sender);
        }
        
        // 设置线程本地变量
        MACRO_ENGINE.with(|cell| {
            cell.set(Some(self.macro_engine.clone()));
        });
        
        // 启动处理键盘事件的线程
        let engine = self.macro_engine.clone();
        thread::spawn(move || {
            Self::process_key_events(receiver, engine);
        });
        
        // 设置钩子
        unsafe {
            let hook_handle = winuser::SetWindowsHookExA(
                winuser::WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                std::ptr::null_mut(),
                0,
            );
            
            if hook_handle.is_null() {
                let error = std::io::Error::last_os_error();
                error!("设置键盘钩子失败: {}", error);
                return Err(Box::new(error));
            }
            
            HOOK_HANDLE.with(|cell| {
                cell.set(Some(hook_handle));
            });
        }
        
        HOOK_RUNNING.store(true, Ordering::SeqCst);
        info!("键盘监听器已启动");
        
        Ok(())
    }
    
    fn process_key_events(receiver: Receiver<(Key, bool)>, engine: Arc<MacroEngine>) {
        info!("按键处理线程已启动");
        while let Ok((key, is_down)) = receiver.recv() {
            debug!("处理按键: {:?}, 状态: {}", key, if is_down { "按下" } else { "释放" });
            engine.process_key_event(key, is_down);
        }
        info!("按键处理线程已停止");
    }
    
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 清理钩子
        HOOK_HANDLE.with(|cell| {
            if let Some(hook) = cell.take() {
                unsafe {
                    if winuser::UnhookWindowsHookEx(hook) == 0 {
                        let error = std::io::Error::last_os_error();
                        error!("清理键盘钩子失败: {}", error);
                    }
                }
            }
        });
        
        HOOK_RUNNING.store(false, Ordering::SeqCst);
        info!("键盘监听器已停止");
        
        Ok(())
    }
}

// 处理键盘事件的回调函数
unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: usize, lparam: isize) -> isize {
    if !HOOK_RUNNING.load(Ordering::SeqCst) || code < 0 {
        return winuser::CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam);
    }
    
    let key_info = *(lparam as *const winuser::KBDLLHOOKSTRUCT);
    let key_code = key_info.vkCode as u32;
    
    let is_keydown = wparam as u32 == winuser::WM_KEYDOWN || wparam as u32 == winuser::WM_SYSKEYDOWN;
    let is_keyup = wparam as u32 == winuser::WM_KEYUP || wparam as u32 == winuser::WM_SYSKEYUP;
    
    if !is_keydown && !is_keyup {
        return winuser::CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam);
    }
    
    let key = virtual_key_to_key(key_code);
    
    // 打印每个按键事件以便调试
    debug!("捕获按键: {:?}, 虚拟键码: {:#x}, 状态: {}", 
           key, key_code, if is_keydown { "按下" } else { "释放" });
    
    let mut handled = false;
    
    MACRO_ENGINE.with(|cell| {
        if let Some(engine) = cell.take() {
            handled = engine.process_key_event(key, is_keydown);
            cell.set(Some(engine));
        }
    });
    
    if handled {
        // 如果事件被处理，则阻止系统继续处理
        return 1;
    }
    
    winuser::CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

fn virtual_key_to_key(vk: u32) -> Key {
    // 将Windows虚拟键码转换为我们的Key类型
    match vk {
        0x1B => Key::Escape,
        0x09 => Key::Tab,
        0x14 => Key::CapsLock,
        0x10 => Key::Shift,
        0x11 => Key::Control,
        0x12 => Key::Alt,
        0x20 => Key::Space,
        0x0D => Key::Enter,
        // 功能键
        0x70 => Key::F1,
        0x71 => Key::F2,
        0x72 => Key::F3,
        0x73 => Key::F4,
        0x74 => Key::F5,
        0x75 => Key::F6,
        0x76 => Key::F7,
        0x77 => Key::F8,
        0x78 => Key::F9,
        0x79 => Key::F10,
        0x7A => Key::F11,
        0x7B => Key::F12,
        
        // 字母和数字
        vk @ 0x30..=0x39 => Key::Character((vk as u8 - 0x30 + b'0') as char),
        vk @ 0x41..=0x5A => Key::Character((vk as u8 - 0x41 + b'a') as char),
        
        // 未映射的键
        _ => {
            debug!("未映射的键: {:#x}", vk);
            Key::Character('\0')
        }
    }
} 