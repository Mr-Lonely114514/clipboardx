#![windows_subsystem = "windows"]

use std::sync::{Arc, Mutex};

mod app;
mod clipboard;
mod hotkey;
mod storage;
mod ui;
mod utils;
mod win32;

use win32::*;
use hotkey::manager::HotkeyManager;

const TRAY_CALLBACK_MSG: u32 = WM_APP + 1;
const TRAY_ICON_ID: u32 = 1;

const MENU_SHOW: usize = 1001;
const MENU_SETTINGS: usize = 1002;
const MENU_ABOUT: usize = 1003;
const MENU_EXIT: usize = 1004;
const MENU_CLEAR: usize = 1005;

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn main() {
    // 设置 panic hook 用于调试
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {}", info);
        log::error!("{}", msg);
        let _ = std::fs::write("C:\\Users\\lenovo\\Desktop\\pasteboard\\panic_info.txt", &msg);
    }));

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis().init();
    log::info!("ClipBoardX 启动...");

    let config = utils::config::Config::load();
    log::info!("配置已加载");

    let db_path = get_db_path();
    let app_state = match app::AppState::new(&db_path, config) {
        Ok(s) => Arc::new(Mutex::new(s)),
        Err(e) => { log::error!("数据库初始化失败: {}", e); return; }
    };
    log::info!("数据库已初始化");

    unsafe { run_message_loop(app_state); }
}

fn get_db_path() -> String {
    let mut p = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("clipboardx.exe"));
    p.set_file_name("clipboardx"); p.set_extension("db");
    p.to_string_lossy().to_string()
}

unsafe fn run_message_loop(app_state: Arc<Mutex<app::AppState>>) {
    let hinst = GetModuleHandleW(std::ptr::null());

    if register_window_classes(hinst).is_err() {
        log::error!("注册窗口类失败"); return;
    }

    // 创建隐藏监听窗口
    let monitor_class = w("ClipBoardX_Monitor");
    let monitor = CreateWindowExW(
        WS_EX_TOOLWINDOW, monitor_class.as_ptr(), std::ptr::null(),
        WS_POPUP, 0, 0, 0, 0, HWND(0), HMENU(0), hinst, std::ptr::null(),
    );
    log::info!("监听窗口已创建");

    // 注册剪切板监听
    if AddClipboardFormatListener(monitor) != FALSE {
        log::info!("剪切板监听已注册");
    } else {
        log::warn!("AddClipboardFormatListener 失败");
    }

    // 注册热键
    let config = app_state.lock().unwrap().config.clone();
    if HotkeyManager::detect_conflict(monitor, config.hotkey_modifiers, config.hotkey_vk) {
        let name = hotkey::manager::HotkeyManager::hotkey_name(config.hotkey_modifiers, config.hotkey_vk);
        log::warn!("热键 {} 已被占用", name);
    }
    let mut hm = hotkey::manager::HotkeyManager::new();
    if let Err(e) = hm.register(monitor, config.hotkey_modifiers, config.hotkey_vk) {
        log::warn!("热键注册失败: {}", e);
    }

    // 创建托盘
    create_tray_icon(monitor);

    // 创建面板
    if let Err(e) = ui::panel::create_panel(hinst, app_state.clone()) {
        log::error!("创建面板失败: {}", e); return;
    }
    log::info!("面板已创建");

    // 消息循环
    let mut msg = MSG::default();
    loop {
        let ret = GetMessageW(&mut msg, HWND(0), 0, 0);
        if ret == 0 || ret == -1 { break; }
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    RemoveClipboardFormatListener(monitor);
    cleanup_tray(monitor);
    log::info!("ClipBoardX 已退出");
}

unsafe fn register_window_classes(hinst: HINSTANCE) -> Result<(), String> {
    let name = w("ClipBoardX_Monitor");
    let wc = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(monitor_wnd_proc),
        cbClsExtra: 0, cbWndExtra: 0,
        hInstance: hinst,
        hIcon: HICON(0),
        hCursor: LoadCursorW(HINSTANCE(0), IDC_ARROW),
        hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH as i32)),
        lpszMenuName: std::ptr::null(),
        lpszClassName: name.as_ptr(),
    };
    if RegisterClassW(&wc as *const WNDCLASSW) == 0 {
        return Err("注册监听器窗口类失败".to_string());
    }
    ui::panel::register_panel_class(hinst)?;
    ui::settings_window::register_settings_class(hinst)?;
    Ok(())
}

unsafe fn create_tray_icon(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = TRAY_ICON_ID;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP;
    nid.uCallbackMessage = TRAY_CALLBACK_MSG;
    nid.hIcon = LoadIconW(HINSTANCE(0), IDI_APPLICATION);
    let tip = w("ClipBoardX - 剪切板管理器");
    let mut i = 0;
    for &ch in &tip {
        if i >= 127 { break; }
        nid.szTip[i] = ch; i += 1;
    }
    nid.szTip[i] = 0;
    Shell_NotifyIconW(NIM_ADD, &nid as *const NOTIFYICONDATAW);
}

unsafe fn cleanup_tray(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = TRAY_ICON_ID;
    nid.uFlags = NIF_ICON;
    Shell_NotifyIconW(NIM_DELETE, &nid as *const NOTIFYICONDATAW);
}

unsafe extern "system" fn monitor_wnd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        0x031D => { handle_clipboard_update(); 0 }
        WM_HOTKEY => { handle_hotkey(); 0 }
        msg if msg == TRAY_CALLBACK_MSG => {
            match l as u32 {
                WM_LBUTTONUP | WM_LBUTTONDBLCLK => handle_hotkey(),
                WM_RBUTTONUP => show_tray_menu(hwnd),
                _ => {}
            }
            0
        }
        WM_DESTROY => { PostQuitMessage(0); 0 }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}

unsafe fn handle_clipboard_update() {
    if let Some(s) = ui::panel::get_panel_state() {
        if let Ok(app) = s.app_state.lock() {
            let r = clipboard::recorder::ClipboardRecorder::new(&app.db, &app.skip_marker, &app.config);
            r.capture_and_save();
        }
    }
}

unsafe fn handle_hotkey() {
    if let Some(s) = ui::panel::get_panel_state() {
        ui::panel::toggle_panel(s.hwnd);
    }
}

unsafe fn show_tray_menu(hwnd: HWND) {
    let hmenu = CreatePopupMenu();
    if hmenu.0 == 0 { return; }
    let s1 = w("显示剪切板面板");
    let s2 = w("设置");
    let s3 = w("清空全部历史");
    let s4 = w("关于");
    let s5 = w("退出");
    AppendMenuW(hmenu, MF_STRING, MENU_SHOW, s1.as_ptr());
    AppendMenuW(hmenu, MF_STRING, MENU_SETTINGS, s2.as_ptr());
    AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(hmenu, MF_STRING, MENU_CLEAR, s3.as_ptr());
    AppendMenuW(hmenu, MF_STRING, MENU_ABOUT, s4.as_ptr());
    AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(hmenu, MF_STRING, MENU_EXIT, s5.as_ptr());

    let mut pt = POINT { x: 0, y: 0 };
    GetCursorPos(&mut pt);
    SetForegroundWindow(hwnd);
    let cmd = TrackPopupMenu(hmenu, TPM_RETURNCMD | TPM_NONOTIFY, pt.x, pt.y, 0, hwnd, std::ptr::null());
    handle_tray_cmd(cmd);
    DestroyMenu(hmenu);
}

unsafe fn handle_tray_cmd(cmd: BOOL) {
    match cmd as usize {
        MENU_SHOW => handle_hotkey(),
        MENU_SETTINGS => {
            if let Some(s) = ui::panel::get_panel_state() {
                let hinst = GetModuleHandleW(std::ptr::null());
                let _ = ui::settings_window::create_settings_window(hinst, s.hwnd, s.app_state.clone());
            }
        }
        MENU_ABOUT => {
            let t = w("ClipBoardX v0.1.0\nWindows 全局剪切板管理器\n基于 Rust + Win32 API");
            let c = w("关于 ClipBoardX");
            MessageBoxW(HWND(0), t.as_ptr(), c.as_ptr(), MB_OK | MB_ICONINFORMATION);
        }
        MENU_CLEAR => {
            let t = w("确定要清空所有剪切板历史记录吗？此操作不可撤销。");
            let c = w("确认清空");
            let ret = MessageBoxW(HWND(0), t.as_ptr(), c.as_ptr(), MB_YESNO | MB_ICONWARNING);
            if ret == IDYES {
                if let Some(s) = ui::panel::get_panel_state() {
                    if let Ok(mut app) = s.app_state.lock() { app.clear_all_records(); }
                }
            }
        }
        MENU_EXIT => { PostQuitMessage(0); }
        _ => {}
    }
}
