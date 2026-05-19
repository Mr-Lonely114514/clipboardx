use std::sync::{Arc, Mutex};

use crate::app::AppState;
use crate::utils::config::{Config, InteractionMode};
use crate::win32::*;
use crate::hotkey::manager::HotkeyManager;

const ID_HOTKEY_EDIT: u32 = 2001;
const ID_MODE_COMBO: u32 = 2002;
const ID_MAX_RECORDS_EDIT: u32 = 2003;
const ID_IMAGE_STORAGE_CHK: u32 = 2004;
const ID_SAVE_BTN: u32 = 2005;
const ID_CANCEL_BTN: u32 = 2006;

static mut SETTINGS_STATE: Option<SettingsState> = None;

struct SettingsState {
    hwnd: HWND,
    app_state: Arc<Mutex<AppState>>,
}

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn register_settings_class(hinst: HINSTANCE) -> Result<(), String> {
    unsafe {
        let class_name = w("ClipBoardX_Settings");
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(settings_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: HICON(0),
            hCursor: LoadCursorW(HINSTANCE(0), IDC_ARROW),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH as i32)),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };
        if RegisterClassW(&wc as *const WNDCLASSW) == 0 {
            return Err("注册设置窗口类失败".to_string());
        }
    }
    Ok(())
}

pub fn create_settings_window(
    hinst: HINSTANCE, parent: HWND, app_state: Arc<Mutex<AppState>>,
) -> Result<HWND, String> {
    unsafe {
        let class_name = w("ClipBoardX_Settings");
        let title = w("ClipBoardX - 设置");
        let hwnd = CreateWindowExW(
            WS_EX_DLGMODALFRAME, class_name.as_ptr(), title.as_ptr(),
            WS_CAPTION | WS_SYSMENU | WS_VISIBLE,
            0, 0, 380, 300, parent, HMENU(0), hinst, std::ptr::null(),
        );
        if hwnd.0 == 0 { return Err("创建设置窗口失败".to_string()); }

        SETTINGS_STATE = Some(SettingsState { hwnd, app_state: app_state.clone() });

        let font_name = w("Microsoft YaHei UI");
        let hfont = CreateFontW(
            -14, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY, DEFAULT_PITCH | FF_DONTCARE, font_name.as_ptr(),
        );

        let config = app_state.lock().unwrap().config.clone();

        // 热键
        let label1 = w("全局热键：");
        let static_class = w("STATIC");
        CreateWindowExW(WS_EX_TRANSPARENT, static_class.as_ptr(), label1.as_ptr(),
            WS_CHILD | WS_VISIBLE | SS_RIGHT, 20, 15, 100, 20, hwnd, HMENU(0), hinst, std::ptr::null());

        let hotkey_str = HotkeyManager::hotkey_name(config.hotkey_modifiers, config.hotkey_vk);
        let hotkey_wide = w(&hotkey_str);
        let edit_class = w("EDIT");
        CreateWindowExW(WS_EX_STATICEDGE, edit_class.as_ptr(), hotkey_wide.as_ptr(),
            WS_CHILD | WS_VISIBLE | ES_READONLY | ES_CENTER,
            120, 15, 200, 22, hwnd, HMENU(ID_HOTKEY_EDIT as isize), hinst, std::ptr::null());

        // 交互模式
        let label2 = w("交互模式：");
        CreateWindowExW(WS_EX_TRANSPARENT, static_class.as_ptr(), label2.as_ptr(),
            WS_CHILD | WS_VISIBLE | SS_RIGHT, 20, 45, 100, 20, hwnd, HMENU(0), hinst, std::ptr::null());

        let combo_class = w("COMBOBOX");
        let combo_hwnd = CreateWindowExW(WS_EX_CLIENTEDGE, combo_class.as_ptr(), std::ptr::null(),
            WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST | CBS_HASSTRINGS,
            120, 45, 200, 80, hwnd, HMENU(ID_MODE_COMBO as isize), hinst, std::ptr::null());

        let auto_txt = w("自动粘贴模式");
        let confirm_txt = w("确认后粘贴模式");
        SendMessageW(combo_hwnd, CB_ADDSTRING, wparam(0), lparam(auto_txt.as_ptr() as isize));
        SendMessageW(combo_hwnd, CB_ADDSTRING, wparam(0), lparam(confirm_txt.as_ptr() as isize));
        let mode_idx = if matches!(config.interaction_mode, InteractionMode::ConfirmThenPaste) { 1 } else { 0 };
        SendMessageW(combo_hwnd, CB_SETCURSEL, wparam(mode_idx), lparam(0));

        // 最大条数
        let label3 = w("最大条数：");
        CreateWindowExW(WS_EX_TRANSPARENT, static_class.as_ptr(), label3.as_ptr(),
            WS_CHILD | WS_VISIBLE | SS_RIGHT, 20, 75, 100, 20, hwnd, HMENU(0), hinst, std::ptr::null());

        let max_str = if config.max_records == -1 { "无限制".to_string() } else { config.max_records.to_string() };
        let max_wide = w(&max_str);
        CreateWindowExW(WS_EX_STATICEDGE, edit_class.as_ptr(), max_wide.as_ptr(),
            WS_CHILD | WS_VISIBLE | ES_LEFT, 120, 75, 200, 22, hwnd, HMENU(ID_MAX_RECORDS_EDIT as isize), hinst, std::ptr::null());

        // 图片存储
        let button_class = w("BUTTON");
        let chk_txt = w("启用图片存储");
        let chk_hwnd = CreateWindowExW(WS_EX_TRANSPARENT, button_class.as_ptr(), chk_txt.as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
            120, 105, 200, 22, hwnd, HMENU(ID_IMAGE_STORAGE_CHK as isize), hinst, std::ptr::null());
        let chk_state = if config.image_storage_enabled { BST_CHECKED } else { BST_UNCHECKED };
        SendMessageW(chk_hwnd, BM_SETCHECK, wparam(chk_state), lparam(0));

        // 保存按钮
        let save_txt = w("保存");
        CreateWindowExW(WS_EX_TRANSPARENT, button_class.as_ptr(), save_txt.as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON | BS_DEFPUSHBUTTON,
            180, 220, 80, 28, hwnd, HMENU(ID_SAVE_BTN as isize), hinst, std::ptr::null());

        let cancel_txt = w("取消");
        CreateWindowExW(WS_EX_TRANSPARENT, button_class.as_ptr(), cancel_txt.as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            270, 220, 80, 28, hwnd, HMENU(ID_CANCEL_BTN as isize), hinst, std::ptr::null());

        Ok(hwnd)
    }
}

unsafe extern "system" fn settings_wnd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => { DestroyWindow(hwnd); 0 }
        WM_COMMAND => {
            let id = LOWORD(w as u32) as u32;
            match id {
                ID_SAVE_BTN => {
                    if let Some(ref mut state) = SETTINGS_STATE {
                        if let Ok(mut app) = state.app_state.lock() {
                            let mut new_config = app.config.clone();

                            let combo_hwnd = GetDlgItem(hwnd, ID_MODE_COMBO as i32);
                            let sel = SendMessageW(combo_hwnd, CB_GETCURSEL, wparam(0), lparam(0));
                            new_config.interaction_mode = if sel == 1 { InteractionMode::ConfirmThenPaste } else { InteractionMode::AutoPaste };

                            let max_hwnd = GetDlgItem(hwnd, ID_MAX_RECORDS_EDIT as i32);
                            let mut buf = [0u16; 32];
                            let len = GetWindowTextW(max_hwnd, buf.as_mut_ptr(), 32);
                            let max_str = String::from_utf16_lossy(&buf[..len as usize]);
                            new_config.max_records = if max_str.trim() == "无限制" || max_str.trim().is_empty() {
                                -1
                            } else {
                                max_str.trim().parse::<i64>().unwrap_or(5000)
                            };

                            let chk_hwnd = GetDlgItem(hwnd, ID_IMAGE_STORAGE_CHK as i32);
                            let check_state = SendMessageW(chk_hwnd, BM_GETCHECK, wparam(0), lparam(0));
                            new_config.image_storage_enabled = check_state as u32 == BST_CHECKED;

                            if let Err(e) = new_config.save() {
                                log::error!("保存配置失败: {}", e);
                            } else { app.config = new_config; }
                        }
                    }
                    DestroyWindow(hwnd); 0
                }
                ID_CANCEL_BTN => { DestroyWindow(hwnd); 0 }
                _ => DefWindowProcW(hwnd, msg, w, l),
            }
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}
