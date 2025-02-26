use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use log::{debug, error, info};
use winapi::um::winuser;
use winapi::shared::minwindef::{WPARAM, LPARAM, LRESULT, DWORD};
use winapi::shared::windef::{HHOOK, HWND};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::macro_engine::MacroEngine;
use crate::config::hero::Key;
use std::sync::mpsc::{channel, Sender, Receiver};
use winapi::um::libloaderapi::{LoadLibraryA, GetProcAddress};
use std::ffi::CString;

// 低级键盘钩子使用的全局变量
thread_local! {
    static HOOK_HANDLE: Cell<Option<HHOOK>> = Cell::new(None);
    static MACRO_ENGINE: Cell<Option<Arc<MacroEngine>>> = Cell::new(None);
}

static HOOK_RUNNING: AtomicBool = AtomicBool::new(false);

type InstallHookFn = unsafe extern "system" fn(*mut i32) -> bool;
type UninstallHookFn = unsafe extern "system" fn() -> bool;

pub struct KeyboardListener {
    macro_engine: Arc<MacroEngine>,
    dll_handle: Mutex<Option<HINSTANCE>>,
    window_handle: *mut i32, // 用于接收消息的窗口句柄
}

impl KeyboardListener {
    pub fn new(macro_engine: Arc<MacroEngine>) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            macro_engine,
            dll_handle: Mutex::new(None),
            window_handle: std::ptr::null_mut(),
        })
    }
    
    pub fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            // 加载 DLL
            let dll_name = CString::new("keyboard_hook.dll")?;
            let handle = LoadLibraryA(dll_name.as_ptr());
            
            if handle.is_null() {
                return Err("Failed to load keyboard_hook.dll".into());
            }
            
            // 获取函数地址
            let install_hook: InstallHookFn = std::mem::transmute(
                GetProcAddress(handle, CString::new("install_hook")?.as_ptr())
            );
            
            // 安装钩子
            if !install_hook(self.window_handle) {
                return Err("Failed to install keyboard hook".into());
            }
            
            *self.dll_handle.lock().unwrap() = Some(handle);
        }
        
        info!("键盘监听器已启动");
        Ok(())
    }
    
    pub fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            if let Some(handle) = *self.dll_handle.lock().unwrap() {
                let uninstall_hook: UninstallHookFn = std::mem::transmute(
                    GetProcAddress(handle, CString::new("uninstall_hook")?.as_ptr())
                );
                
                if !uninstall_hook() {
                    return Err("Failed to uninstall keyboard hook".into());
                }
            }
        }
        
        info!("键盘监听器已停止");
        Ok(())
    }
}

// 处理键盘事件的回调函数
unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    info!("键盘钩子被调用: code={}, wparam={}, lparam={}", code, wparam, lparam);
    
    if !HOOK_RUNNING.load(Ordering::SeqCst) || code < 0 {
        return winuser::CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam);
    }
    
    let key_info = *(lparam as *const winuser::KBDLLHOOKSTRUCT);
    let key_code = key_info.vkCode as u32;
    
    let is_keydown = wparam as u32 == winuser::WM_KEYDOWN || wparam as u32 == winuser::WM_SYSKEYDOWN;
    let is_keyup = wparam as u32 == winuser::WM_KEYUP || wparam as u32 == winuser::WM_SYSKEYUP;
    
    // 添加原始按键信息日志
    debug!("原始按键事件 - 虚拟键码: {:#x}, wparam: {:#x}, 扫描码: {:#x}, 标志: {:#x}", 
           key_info.vkCode, 
           wparam,
           key_info.scanCode,
           key_info.flags);
    
    if !is_keydown && !is_keyup {
        return winuser::CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam);
    }
    
    let key = virtual_key_to_key(key_code);
    
    // 打印每个按键事件的详细信息
    info!("键盘事件: {} - 按键: {:?}, 虚拟键码: {:#x}", 
          if is_keydown { "按下" } else { "释放" },
          key,
          key_code);
    
    let mut handled = false;
    
    MACRO_ENGINE.with(|cell| {
        if let Some(engine) = cell.take() {
            handled = engine.process_key_event(key.clone(), is_keydown);
            if handled {
                debug!("按键被宏引擎处理: {:?}", key);
            }
            cell.set(Some(engine));
        }
    });
    
    if handled {
        debug!("按键被屏蔽: {:?}", key);
        return 1;
    }
    
    winuser::CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

fn virtual_key_to_key(vk: u32) -> Key {
    let key = match vk {
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
            debug!("未映射的虚拟键码: {:#x}", vk);
            Key::Character('\0')
        }
    };
    
    debug!("虚拟键码 {:#x} 映射为 {:?}", vk, key);
    key
} 