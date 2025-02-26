use std::sync::atomic::{AtomicPtr, Ordering};
use winapi::shared::minwindef::{DWORD, LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::winuser;

static HOOK_HANDLE: AtomicPtr<HHOOK__> = AtomicPtr::new(std::ptr::null_mut());
static CALLBACK_WINDOW: AtomicPtr<i32> = AtomicPtr::new(std::ptr::null_mut());

#[no_mangle]
pub extern "system" fn install_hook(window_handle: *mut i32) -> bool {
    unsafe {
        CALLBACK_WINDOW.store(window_handle, Ordering::SeqCst);
        
        let hook = winuser::SetWindowsHookExA(
            winuser::WH_KEYBOARD_LL,
            Some(keyboard_hook_proc),
            std::ptr::null_mut(),
            0,
        );
        
        if hook.is_null() {
            return false;
        }
        
        HOOK_HANDLE.store(hook, Ordering::SeqCst);
        true
    }
}

#[no_mangle]
pub extern "system" fn uninstall_hook() -> bool {
    unsafe {
        let hook = HOOK_HANDLE.load(Ordering::SeqCst);
        if !hook.is_null() {
            let result = winuser::UnhookWindowsHookEx(hook);
            HOOK_HANDLE.store(std::ptr::null_mut(), Ordering::SeqCst);
            return result != 0;
        }
        false
    }
}

unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let window = CALLBACK_WINDOW.load(Ordering::SeqCst);
        if !window.is_null() {
            let key_info = *(lparam as *const winuser::KBDLLHOOKSTRUCT);
            winuser::PostMessageA(
                window as HWND,
                winuser::WM_USER + 1, // 自定义消息
                wparam,
                lparam,
            );
        }
    }
    winuser::CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
} 