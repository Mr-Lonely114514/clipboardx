use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use crate::app::AppState;
use crate::utils::config::InteractionMode;
use crate::win32::*;
use crate::clipboard::recorder::ClipboardRecorder;
use crate::storage::repository::Repository;

/// 面板状态
pub struct PanelState {
    pub hwnd: HWND,
    pub list_hwnd: HWND,
    pub search_hwnd: HWND,
    pub btn_delete: HWND,
    pub btn_confirm: HWND,
    pub app_state: Arc<Mutex<AppState>>,
    pub prev_foreground: HWND,
    pub delete_mode: bool,
    pub checked_ids: HashSet<i64>,
    pub list_orig_proc: isize,
}

static mut PANEL_STATE: Option<PanelState> = None;

const ID_SEARCH_BOX: u32 = 1001;
const ID_LIST_BOX: u32 = 1002;
const ID_BTN_DELETE: u32 = 1003;
const ID_BTN_CONFIRM: u32 = 1004;

fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn get_panel_state() -> Option<&'static PanelState> {
    unsafe { PANEL_STATE.as_ref() }
}

pub fn register_panel_class(hinst: HINSTANCE) -> Result<(), String> {
    unsafe {
        let class_name = w("ClipBoardX_Panel");
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
            lpfnWndProc: Some(panel_wnd_proc),
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
            return Err("注册面板窗口类失败".to_string());
        }
    }
    Ok(())
}

pub fn create_panel(hinst: HINSTANCE, app_state: Arc<Mutex<AppState>>) -> Result<HWND, String> {
    unsafe {
        let class_name = w("ClipBoardX_Panel");
        let title = w("ClipBoardX");

        let hwnd = CreateWindowExW(
            WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_TOPMOST,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            0, 0, 420, 1,
            HWND(0),
            HMENU(0),
            hinst,
            std::ptr::null(),
        );
        if hwnd.0 == 0 {
            return Err("创建面板窗口失败".to_string());
        }

        SetLayeredWindowAttributes(hwnd, 0, 240, LWA_ALPHA);

        // 搜索框
        let edit_class = w("EDIT");
        let search_hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            edit_class.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | ES_LEFT | ES_AUTOHSCROLL,
            8, 6, 404, 28,
            hwnd, HMENU(ID_SEARCH_BOX as isize), hinst, std::ptr::null(),
        );

        let font_name = w("Microsoft YaHei UI");
        let hfont = CreateFontW(
            -14, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY, DEFAULT_PITCH | FF_DONTCARE, font_name.as_ptr(),
        );
        SendMessageW(search_hwnd, WM_SETFONT, wparam(hfont.0 as u32), lparam(0));

        // 列表框
        let list_class = w("LISTBOX");
        let list_hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            list_class.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_TABSTOP | LBS_NOTIFY | LBS_NOINTEGRALHEIGHT,
            0, 40, 420, 410,
            hwnd, HMENU(ID_LIST_BOX as isize), hinst, std::ptr::null(),
        );

        let hfont_list = CreateFontW(
            -13, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY, DEFAULT_PITCH | FF_DONTCARE, font_name.as_ptr(),
        );
        SendMessageW(list_hwnd, WM_SETFONT, wparam(hfont_list.0 as u32), lparam(0));

        // "删除" 按钮
        let btn_class = w("BUTTON");
        let btn_delete = CreateWindowExW(
            0,
            btn_class.as_ptr(),
            w("删除").as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            60, 456, 130, 28,
            hwnd, HMENU(ID_BTN_DELETE as isize), hinst, std::ptr::null(),
        );
        SendMessageW(btn_delete, WM_SETFONT, wparam(hfont.0 as u32), lparam(0));

        // "确认删除" 按钮
        let btn_confirm = CreateWindowExW(
            0,
            btn_class.as_ptr(),
            w("确认删除").as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            220, 456, 130, 28,
            hwnd, HMENU(ID_BTN_CONFIRM as isize), hinst, std::ptr::null(),
        );
        SendMessageW(btn_confirm, WM_SETFONT, wparam(hfont.0 as u32), lparam(0));

        // 子类化列表框：拦截键盘消息
        let orig_proc = SetWindowLongPtrW(list_hwnd, GWLP_WNDPROC, list_box_proc as isize);
        let state = PanelState {
            hwnd, list_hwnd, search_hwnd,
            btn_delete, btn_confirm,
            app_state, prev_foreground: HWND(0),
            delete_mode: false, checked_ids: HashSet::new(),
            list_orig_proc: orig_proc,
        };
        PANEL_STATE = Some(state);
        Ok(hwnd)
    }
}

pub fn show_panel(hwnd: HWND) {
    unsafe {
        // 重置删除模式
        if let Some(ref mut state) = PANEL_STATE {
            state.delete_mode = false;
            state.checked_ids.clear();
            // 更新按钮文字
            SetWindowTextW(state.btn_delete, w("删除").as_ptr());
        }
        // 记录打开面板前的前景窗口
        let fg = GetForegroundWindow();
        if fg != hwnd {
            if let Some(ref mut state) = PANEL_STATE {
                state.prev_foreground = fg;
            }
        }
        refresh_list();
        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);
        let x = if pt.x - 210 < 0 { 10 } else { pt.x - 210 };
        let y = pt.y + 10;
        SetWindowPos(hwnd, HWND_TOPMOST, x, y, 420, 500, SWP_SHOWWINDOW | SWP_NOACTIVATE);
        if let Some(ref state) = PANEL_STATE {
            SetFocus(state.list_hwnd);
        }
    }
}

pub fn hide_panel(hwnd: HWND) {
    unsafe { ShowWindow(hwnd, SW_HIDE); }
}

pub fn toggle_panel(hwnd: HWND) {
    unsafe {
        if IsWindowVisible(hwnd) != FALSE {
            hide_panel(hwnd);
        } else {
            show_panel(hwnd);
        }
    }
}

unsafe fn refresh_list() {
    let state = match PANEL_STATE.as_mut() { Some(s) => s, None => return };
    SendMessageW(state.list_hwnd, LB_RESETCONTENT, wparam(0), lparam(0));
    if let Ok(mut app) = state.app_state.lock() {
        app.refresh_records();
        let records = app.records.clone();
        let delete_mode = state.delete_mode;
        drop(app);
        for record in &records {
            let checked = if delete_mode {
                if state.checked_ids.contains(&record.id) { "☑ " } else { "☐ " }
            } else {
                ""
            };
            let preview = format_preview(record);
            let wide = w(&format!("{}{}", checked, preview));
            let idx = SendMessageW(state.list_hwnd, LB_ADDSTRING, wparam(0), lparam(wide.as_ptr() as isize));
            SendMessageW(state.list_hwnd, LB_SETITEMDATA, wparam(idx as u32), lparam(record.id as isize));
        }
    }
}

fn format_preview(record: &crate::storage::models::ClipRecord) -> String {
    let max = 70usize;
    let preview = match record.content_type.as_str() {
        "image" => "[图片]".to_string(),
        "file" => record.content_text.as_ref().map(|t| truncate(t, max)).unwrap_or_else(|| "[文件]".to_string()),
        _ => record.content_text.as_ref().map(|t| truncate(t, max)).unwrap_or_else(|| "[空]".to_string()),
    };
    let time = record.created_at.format("%H:%M").to_string();
    let fav = if record.is_favorite { " ★" } else { "" };
    format!("{}{}  {}", preview, fav, time)
}

fn truncate(s: &str, max: usize) -> String {
    let line = s.lines().next().unwrap_or(s);
    if line.len() > max { format!("{}...", &line[..max]) } else { line.to_string() }
}

unsafe fn simulate_ctrl_v() {
    use crate::win32::*;
    let prev = if let Some(ref state) = PANEL_STATE { state.prev_foreground } else { HWND(0) };

    if prev.0 == 0 {
        return;
    }

    // 1. 激活目标窗口
    SetForegroundWindow(prev);
    std::thread::sleep(std::time::Duration::from_millis(30));

    // 2. 用 keybd_event 模拟 Ctrl+V（比 SendInput 更可靠，跨进程兼容性好）
    // 按 Ctrl
    keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_EXTENDEDKEY, 0);
    // 按 V
    keybd_event(0x56u8, 0, KEYEVENTF_EXTENDEDKEY, 0);
    // 松 V
    keybd_event(0x56u8, 0, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0);
    // 松 Ctrl
    keybd_event(VK_CONTROL as u8, 0, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP, 0);
}

unsafe fn do_paste(record_id: i64) {
    if let Some(ref state) = PANEL_STATE {
        if let Ok(app) = state.app_state.lock() {
            let repo = Repository::new(&app.db);
            if let Ok(Some(record)) = repo.find_by_id(record_id) {
                let recorder = ClipboardRecorder::new(&app.db, &app.skip_marker, &app.config);
                recorder.writeback_record(&record);
                simulate_ctrl_v();
            }
        }
    }
}

unsafe fn writeback_only(record_id: i64) {
    if let Some(ref state) = PANEL_STATE {
        if let Ok(app) = state.app_state.lock() {
            let repo = Repository::new(&app.db);
            if let Ok(Some(record)) = repo.find_by_id(record_id) {
                let recorder = ClipboardRecorder::new(&app.db, &app.skip_marker, &app.config);
                recorder.writeback_record(&record);
            }
        }
    }
}

/// 切换删除模式（开关）
unsafe fn toggle_delete_mode() {
    if let Some(ref mut state) = PANEL_STATE {
        state.delete_mode = !state.delete_mode;
        if !state.delete_mode {
            // 退出删除模式，清空勾选
            state.checked_ids.clear();
        }
        SetWindowTextW(state.btn_delete, w(if state.delete_mode { "取消" } else { "删除" }).as_ptr());
        // 隐藏/显示"确认删除"按钮
        ShowWindow(state.btn_confirm, if state.delete_mode { SW_SHOW } else { SW_HIDE });
    }
    refresh_list();
}

/// 确认删除：删除所有勾选的记录
unsafe fn confirm_delete() {
    if let Some(ref mut state) = PANEL_STATE {
        if state.checked_ids.is_empty() {
            return;
        }
        // 显示确认对话框
        let msg = w(&format!("确定要删除 {} 条记录吗？", state.checked_ids.len()));
        let title = w("确认删除");
        let ret = MessageBoxW(state.hwnd, msg.as_ptr(), title.as_ptr(), MB_YESNO | MB_ICONWARNING);
        if ret != IDYES {
            return;
        }
        // 删除所有勾选的记录
        for id in state.checked_ids.drain() {
            if let Ok(mut app) = state.app_state.lock() {
                app.delete_record(id);
            }
        }
        state.delete_mode = false;
        SetWindowTextW(state.btn_delete, w("删除").as_ptr());
        ShowWindow(state.btn_confirm, SW_HIDE);
    }
    refresh_list();
}

/// 切换当前选中记录的勾选状态
unsafe fn toggle_check_selected() {
    // 第一步：只读方式获取当前选中记录的 ID
    let item_id = {
        let state = match PANEL_STATE.as_ref() { Some(s) => s, None => return };
        if !state.delete_mode { return; }
        let sel = SendMessageW(state.list_hwnd, LB_GETCURSEL, wparam(0), lparam(0));
        if sel < 0 { return; }
        SendMessageW(state.list_hwnd, LB_GETITEMDATA, wparam(sel as u32), lparam(0)) as i64
    };
    // 第二步：通过可变引用修改 checked_ids
    if let Some(ref mut state) = PANEL_STATE {
        if state.checked_ids.contains(&item_id) {
            state.checked_ids.remove(&item_id);
        } else {
            state.checked_ids.insert(item_id);
        }
    }
    refresh_list();
}

/// 列表框子类化窗口过程 -- 拦截 Escape 键和 Space 键（删除模式下切换勾选）
unsafe extern "system" fn list_box_proc(
    hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM,
) -> LRESULT {
    if msg == WM_KEYDOWN {
        let vk = (w as u32 & 0xFFFF) as u16;
        match vk {
            VK_ESCAPE => {
                if let Some(ref state) = PANEL_STATE {
                    hide_panel(state.hwnd);
                }
                return 0;
            }
            VK_SPACE => {
                // 删除模式下按空格切换勾选
                if let Some(ref state) = PANEL_STATE {
                    if state.delete_mode {
                        toggle_check_selected();
                        return 0;
                    }
                }
            }
            _ => {}
        }
    }
    // 其余消息走默认列表框处理
    let orig = PANEL_STATE.as_ref().map(|s| s.list_orig_proc).unwrap_or(0);
    CallWindowProcW(orig, hwnd, msg, w, l)
}

unsafe extern "system" fn panel_wnd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => { hide_panel(hwnd); 0 }
        WM_COMMAND => {
            let id = LOWORD(w as u32) as u32;
            let code = HIWORD(w as u32) as u32;
            match id {
                ID_BTN_DELETE if code == BN_CLICKED => {
                    toggle_delete_mode();
                    0
                }
                ID_BTN_CONFIRM if code == BN_CLICKED => {
                    confirm_delete();
                    0
                }
                ID_LIST_BOX if code == LBN_SELCHANGE => {
                    if let Some(ref state) = PANEL_STATE {
                        if state.delete_mode {
                            // 删除模式下点击 = 切换勾选
                            toggle_check_selected();
                        } else if let Ok(app) = state.app_state.lock() {
                            let sel = SendMessageW(state.list_hwnd, LB_GETCURSEL, wparam(0), lparam(0));
                            if sel >= 0 {
                                let item_id = SendMessageW(state.list_hwnd, LB_GETITEMDATA, wparam(sel as u32), lparam(0));
                                if app.config.interaction_mode == InteractionMode::AutoPaste {
                                    drop(app);
                                    hide_panel(hwnd);
                                    std::thread::sleep(std::time::Duration::from_millis(50));
                                    if let Some(ref state) = PANEL_STATE {
                                        if state.prev_foreground.0 != 0 {
                                            SetForegroundWindow(state.prev_foreground);
                                        }
                                    }
                                    std::thread::sleep(std::time::Duration::from_millis(20));
                                    do_paste(item_id as i64);
                                } else {
                                    drop(app);
                                    writeback_only(item_id as i64);
                                }
                            }
                        }
                    }
                    0
                }
                ID_LIST_BOX if code == LBN_DBLCLK => {
                    if let Some(ref state) = PANEL_STATE {
                        if state.delete_mode {
                            toggle_check_selected();
                            return 0;
                        }
                    }
                    // 正常模式：双击粘贴
                    let sel = SendMessageW(PANEL_STATE.as_ref().unwrap().list_hwnd, LB_GETCURSEL, wparam(0), lparam(0));
                    if sel >= 0 {
                        let item_id = SendMessageW(PANEL_STATE.as_ref().unwrap().list_hwnd, LB_GETITEMDATA, wparam(sel as u32), lparam(0));
                        hide_panel(hwnd);
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        if let Some(ref state) = PANEL_STATE {
                            if state.prev_foreground.0 != 0 {
                                SetForegroundWindow(state.prev_foreground);
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(20));
                        do_paste(item_id as i64);
                    }
                    0
                }
                ID_SEARCH_BOX if code == EN_CHANGE => {
                    if let Some(ref state) = PANEL_STATE {
                        let mut buf = [0u16; 256];
                        let len = GetWindowTextW(state.search_hwnd, buf.as_mut_ptr(), 256);
                        let keyword = String::from_utf16_lossy(&buf[..len as usize]);
                        if let Ok(mut app) = state.app_state.lock() {
                            app.search_keyword = keyword.trim().to_string();
                        }
                        refresh_list();
                    }
                    0
                }
                _ => DefWindowProcW(hwnd, msg, w, l),
            }
        }
        WM_ACTIVATE => {
            if (w & 0xFFFF) as u32 == WA_INACTIVE { hide_panel(hwnd); }
            DefWindowProcW(hwnd, msg, w, l)
        }
        WM_KEYDOWN => {
            let vk = (w as u32 & 0xFFFF) as u16;
            if vk == VK_ESCAPE { hide_panel(hwnd); 0 }
            else { DefWindowProcW(hwnd, msg, w, l) }
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}
