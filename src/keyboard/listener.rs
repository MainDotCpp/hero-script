use std::sync::Arc;
use std::time::{Duration, Instant};
use std::thread;
use log::{debug, error, info};
use winapi::um::winuser;
use winapi::shared::windef::HWND;
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::macro_engine::MacroEngine;
use crate::config::hero::Key;

// 低级键盘钩子使用的全局变量
thread_local! {
    static HOOK_HANDLE: Cell<Option<winapi::shared::windef::HHOOK>> = Cell::new(None);
    static MACRO_ENGINE: Cell<Option<Arc<MacroEngine>>> = Cell::new(None);
}

static HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

pub struct KeyboardListener {
    macro_engine: Arc<MacroEngine>,
}

impl KeyboardListener {
    pub fn new(macro_engine: Arc<MacroEngine>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            macro_engine,
        })
    }
    
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        if HOOK_RUNNING.load(Ordering::SeqCst) {
            return Ok(()); // 已经启动
        }
        
        // 设置全局变量
        MACRO_ENGINE.with(|cell| {
            cell.set(Some(self.macro_engine.clone()));
        });
        
        // 启动低级键盘钩子
        let result = unsafe {
            // 定义键盘钩子回调
            extern "system" fn keyboard_hook_proc(code: i32, wparam: usize, lparam: isize) -> isize {
                let handled = process_key_event(code, wparam, lparam);
                
                if handled {
                    return 1; // 阻止按键继续传递
                } else {
                    // 调用下一个钩子
                    return unsafe { winuser::CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) };
                }
            }
            
            // 设置钩子
            let hook = winuser::SetWindowsHookExA(
                winuser::WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                std::ptr::null_mut(),
                0,
            );
            
            if hook.is_null() {
                Err(std::io::Error::last_os_error())
            } else {
                HOOK_HANDLE.with(|cell| {
                    cell.set(Some(hook));
                });
                HOOK_RUNNING.store(true, Ordering::SeqCst);
                Ok(())
            }
        };
        
        if result.is_err() {
            error!("键盘钩子设置失败: {:?}", result.err().unwrap());
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other, 
                "键盘钩子设置失败"
            )));
        }
        
        info!("键盘监听器已启动");
        Ok(())
    }
    
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !HOOK_RUNNING.load(Ordering::SeqCst) {
            return Ok(()); // 已经停止
        }
        
        // 移除钩子
        HOOK_HANDLE.with(|cell| {
            if let Some(hook) = cell.get() {
                unsafe {
                    winuser::UnhookWindowsHookEx(hook);
                }
                cell.set(None);
            }
        });
        
        HOOK_RUNNING.store(false, Ordering::SeqCst);
        info!("键盘监听器已停止");
        
        Ok(())
    }
}

// 处理键盘事件
fn process_key_event(code: i32, wparam: usize, lparam: isize) -> bool {
    if code < 0 {
        return false;
    }
    
    let key_info = unsafe { *(lparam as *const winuser::KBDLLHOOKSTRUCT) };
    let key_code = key_info.vkCode as u32;
    
    let is_keydown = wparam as u32 == winuser::WM_KEYDOWN || wparam as u32 == winuser::WM_SYSKEYDOWN;
    let is_keyup = wparam as u32 == winuser::WM_KEYUP || wparam as u32 == winuser::WM_SYSKEYUP;
    
    if !is_keydown && !is_keyup {
        return false;
    }
    
    let key = virtual_key_to_key(key_code);
    
    // 增加日志以便追踪
    if is_keydown {
        debug!("键盘钩子: 按键按下 - VK: {:#x}, 转换为: {:?}", key_code, key);
    } else {
        debug!("键盘钩子: 按键释放 - VK: {:#x}, 转换为: {:?}", key_code, key);
    }
    
    let mut handled = false;
    
    MACRO_ENGINE.with(|cell| {
        if let Some(engine) = cell.take() {
            // 传递按键事件到宏引擎
            handled = engine.process_key_event(key, is_keydown);
            debug!("宏引擎处理结果: {}", if handled { "已处理" } else { "未处理" });
            // 放回引用计数的值
            cell.set(Some(engine));
        } else {
            error!("无法访问宏引擎");
        }
    });
    
    handled
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