use crate::win32::*;

/// 列表视图辅助操作
pub fn get_selected_record_id(list_hwnd: HWND) -> Option<i64> {
    unsafe {
        let sel = SendMessageW(list_hwnd, LB_GETCURSEL, wparam(0), lparam(0));
        if sel >= 0 {
            let id = SendMessageW(list_hwnd, LB_GETITEMDATA, wparam(sel as u32), lparam(0));
            Some(id as i64)
        } else {
            None
        }
    }
}
