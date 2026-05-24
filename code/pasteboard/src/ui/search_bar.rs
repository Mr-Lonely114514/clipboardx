use crate::win32::*;

/// 搜索框辅助操作
pub fn get_search_text(search_hwnd: HWND) -> String {
    unsafe {
        let mut buf = [0u16; 256];
        let len = GetWindowTextW(search_hwnd, buf.as_mut_ptr(), 256);
        if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }
}

pub fn clear_search(search_hwnd: HWND) {
    unsafe {
        SetWindowTextW(search_hwnd, std::ptr::null());
    }
}
