use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use crate::app::AppState;
use crate::win32::*;
use crate::clipboard::recorder::ClipboardRecorder;
use crate::storage::repository::Repository;
use crate::ui::settings_window;

/// 面板状态
pub struct PanelState {
    pub hwnd: HWND,
    pub list_hwnd: HWND,
    pub search_hwnd: HWND,
    pub btn_delete: HWND,
    pub btn_confirm: HWND,
    pub btn_settings: HWND,
    pub btn_fav_filter: HWND,
    pub app_state: Arc<Mutex<AppState>>,
    pub prev_foreground: HWND,
    pub delete_mode: bool,
    pub checked_ids: HashSet<i64>,
    pub list_orig_proc: isize,
    pub search_orig_proc: isize,
    pub preview_hwnd: HWND,
    pub preview_edit_hwnd: HWND,
    pub preview_edit_orig_proc: isize,
    pub preview_opening: bool, // CreateWindowExW 期间预防 WM_ACTIVATE 误判
    pub in_dialog: bool,       // 正在弹模态对话框，不自动隐藏面板
}

static mut PANEL_STATE: Option<PanelState> = None;

const ID_LIST_BOX: u32 = 1002;
const ID_BTN_DELETE: u32 = 1003;
const ID_BTN_CONFIRM: u32 = 1004;
const ID_BTN_SETTINGS: u32 = 1005;
const ID_SEARCH_EDIT: u32 = 1006;
const ID_BTN_FAV_FILTER: u32 = 1007;
const EM_SETCUEBANNER: u32 = 0x1501;
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

        // 注册预览弹出窗口类
        let preview_class_name = w("ClipBoardX_Preview");
        let wc_preview = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(preview_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: HICON(0),
            hCursor: LoadCursorW(HINSTANCE(0), IDC_ARROW),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH as i32)),
            lpszMenuName: std::ptr::null(),
            lpszClassName: preview_class_name.as_ptr(),
        };
        if RegisterClassW(&wc_preview as *const WNDCLASSW) == 0 {
            return Err("注册预览窗口类失败".to_string());
        }
    }
    Ok(())
}

pub fn create_panel(hinst: HINSTANCE, app_state: Arc<Mutex<AppState>>) -> Result<HWND, String> {
    unsafe {
        let class_name = w("ClipBoardX_Panel");
        let title = w("ClipBoardX");

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_TOPMOST,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP | WS_CAPTION | WS_SYSMENU,
            0, 0, 460, 1,
            HWND(0),
            HMENU(0),
            hinst,
            std::ptr::null(),
        );
        if hwnd.0 == 0 {
            return Err("创建面板窗口失败".to_string());
        }

        SetLayeredWindowAttributes(hwnd, 0, 240, LWA_ALPHA);

        let hfont = CreateFontW(
            -14, 0, 0, 0, FW_NORMAL as i32, 0, 0, 0,
            DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY, DEFAULT_PITCH | FF_DONTCARE, w("Microsoft YaHei UI").as_ptr(),
        );

        // 搜索框（标题栏和列表框之间）
        let edit_class = w("EDIT");
        let search_hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            edit_class.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_LEFT | ES_AUTOHSCROLL,
            4, 6, 378, 24,
            hwnd, HMENU(ID_SEARCH_EDIT as isize), hinst, std::ptr::null(),
        );
        SendMessageW(search_hwnd, WM_SETFONT, wparam(hfont.0 as u32), lparam(0));
        // 提示文字
        let placeholder = w("搜索历史记录…");
        SendMessageW(search_hwnd, EM_SETCUEBANNER, wparam(1), lparam(placeholder.as_ptr() as isize));

        // "收藏"过滤按钮（搜索框右侧）
        let btn_class = w("BUTTON");
        let btn_fav_filter = CreateWindowExW(
            0,
            btn_class.as_ptr(),
            w("☆ 收藏").as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            388, 6, 68, 24,
            hwnd, HMENU(ID_BTN_FAV_FILTER as isize), hinst, std::ptr::null(),
        );
        SendMessageW(btn_fav_filter, WM_SETFONT, wparam(hfont.0 as u32), lparam(0));

        // 列表框（搜索框下方）
        let list_class = w("LISTBOX");
        let list_hwnd = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            list_class.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_TABSTOP | LBS_NOTIFY | LBS_NOINTEGRALHEIGHT | LBS_OWNERDRAWFIXED | LBS_HASSTRINGS,
            0, 34, 460, 418,
            hwnd, HMENU(ID_LIST_BOX as isize), hinst, std::ptr::null(),
        );
        SendMessageW(list_hwnd, WM_SETFONT, wparam(hfont.0 as u32), lparam(0));

        // "删除" 按钮
        let btn_class = w("BUTTON");
        let btn_delete = CreateWindowExW(
            0,
            btn_class.as_ptr(),
            w("删除").as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            80, 456, 130, 28,
            hwnd, HMENU(ID_BTN_DELETE as isize), hinst, std::ptr::null(),
        );
        SendMessageW(btn_delete, WM_SETFONT, wparam(hfont.0 as u32), lparam(0));

        // "设置" 按钮（删除按钮右侧）
        let btn_settings = CreateWindowExW(
            0,
            btn_class.as_ptr(),
            w("设置").as_ptr(),
            WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
            250, 456, 130, 28,
            hwnd, HMENU(ID_BTN_SETTINGS as isize), hinst, std::ptr::null(),
        );
        SendMessageW(btn_settings, WM_SETFONT, wparam(hfont.0 as u32), lparam(0));

        // "确认删除" 按钮（初始隐藏，进入删除模式后显示）
        let btn_confirm = CreateWindowExW(
            0,
            btn_class.as_ptr(),
            w("确认删除").as_ptr(),
            WS_CHILD | BS_PUSHBUTTON,
            250, 456, 130, 28,
            hwnd, HMENU(ID_BTN_CONFIRM as isize), hinst, std::ptr::null(),
        );
        SendMessageW(btn_confirm, WM_SETFONT, wparam(hfont.0 as u32), lparam(0));

        // 子类化列表框：拦截键盘消息
        let list_orig_proc = SetWindowLongPtrW(list_hwnd, GWLP_WNDPROC, list_box_proc as isize);
        // 子类化搜索框：拦截键盘消息（ESC 清空+HIDE, 下箭头跳转列表框）
        let search_orig_proc = SetWindowLongPtrW(search_hwnd, GWLP_WNDPROC, search_edit_proc as isize);
        let state = PanelState {
            hwnd, list_hwnd, search_hwnd,
            btn_delete, btn_confirm, btn_settings, btn_fav_filter,
            app_state, prev_foreground: HWND(0),
            delete_mode: false, checked_ids: HashSet::new(),
            list_orig_proc,
            search_orig_proc,
            preview_hwnd: HWND(0),
            preview_edit_hwnd: HWND(0),
            preview_edit_orig_proc: 0,
            preview_opening: false,
            in_dialog: false,
        };
        PANEL_STATE = Some(state);
        Ok(hwnd)
    }
}

pub fn show_panel(hwnd: HWND) {
    unsafe {
        // 打开面板前关闭任何遗留的预览窗口
        close_preview();
        // 重置删除模式
        if let Some(ref mut state) = PANEL_STATE {
            state.delete_mode = false;
            state.checked_ids.clear();
            SetWindowTextW(state.btn_delete, w("删除").as_ptr());
            ShowWindow(state.btn_confirm, SW_HIDE);
            ShowWindow(state.btn_settings, SW_SHOW);
            // 重置收藏过滤（按钮文本 + 状态）
            SetWindowTextW(state.btn_fav_filter, w("☆ 收藏").as_ptr());
            // 清空搜索框
            SetWindowTextW(state.search_hwnd, std::ptr::null());
        }
        // 重置搜索关键词和收藏过滤
        if let Some(ref state) = PANEL_STATE {
            if let Ok(mut app) = state.app_state.lock() {
                app.search_keyword.clear();
                app.favorite_filter = false;
            }
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

        // 获取鼠标所在监视器的工作区
        let monitor = MonitorFromPoint(pt, 2); // MONITOR_DEFAULTTONEAREST
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            rcMonitor: RECT { left: 0, top: 0, right: 0, bottom: 0 },
            rcWork: RECT { left: 0, top: 0, right: 0, bottom: 0 },
            dwFlags: 0,
        };
        GetMonitorInfoW(monitor, &mut mi);

        let panel_w = 460;
        let panel_h = 530; // 给 WS_CAPTION 标题栏留出空间（~30px）

        // X：水平居中于鼠标，并钳制在工作区内
        let mut x = pt.x - panel_w / 2;
        if x < mi.rcWork.left {
            x = mi.rcWork.left + 4;
        }
        if x + panel_w > mi.rcWork.right {
            x = mi.rcWork.right - panel_w - 4;
        }

        // Y：默认在鼠标下方，若超出底部则移到鼠标上方
        let mut y = pt.y + 10;
        if y + panel_h > mi.rcWork.bottom {
            y = pt.y - panel_h - 10;
            if y < mi.rcWork.top {
                y = mi.rcWork.top + 4;
            }
        }

        SetWindowPos(hwnd, HWND_TOPMOST, x, y, panel_w, panel_h, SWP_SHOWWINDOW);
        if let Some(ref state) = PANEL_STATE {
            SetFocus(state.search_hwnd);
        }
    }
}

pub fn hide_panel(hwnd: HWND) {
    unsafe {
        close_preview();
        // 清空搜索状态
        if let Some(ref state) = PANEL_STATE {
            SetWindowTextW(state.search_hwnd, std::ptr::null());
        }
        if let Some(ref state) = PANEL_STATE {
            if let Ok(mut app) = state.app_state.lock() {
                app.search_keyword.clear();
            }
        }
        ShowWindow(hwnd, SW_HIDE);
    }
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

pub unsafe fn refresh_list() {
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
    let max = 55usize;
    let preview = match record.content_type.as_str() {
        "image" => "[图片]".to_string(),
        "file" => record.content_text.as_ref().map(|t| truncate(t, max)).unwrap_or_else(|| "[文件]".to_string()),
        _ => record.content_text.as_ref().map(|t| truncate(t, max)).unwrap_or_else(|| "[空]".to_string()),
    };
    let time = record.created_at.format("%Y.%m.%d %H:%M").to_string();
    format!("{}  {}", preview, time)
}

fn truncate(s: &str, max: usize) -> String {
    let line = s.lines().next().unwrap_or(s);
    if line.len() > max {
        let mut end = max;
        while end > 0 && !line.is_char_boundary(end) {
            end -= 1;
        }
        if end == 0 { return "...".to_string(); }
        format!("{}...", &line[..end])
    } else {
        line.to_string()
    }
}

/// 预览内容显示的最大字符数
const MAX_PREVIEW_CHARS: usize = 8000;

/// 预览弹出窗口的窗口过程（处理关闭、缩放、Escape）
unsafe extern "system" fn preview_wnd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => {
            DestroyWindow(hwnd);
            return 0;
        }
        WM_KEYDOWN => {
            let vk = (w as u32 & 0xFFFF) as u16;
            if vk == VK_ESCAPE {
                DestroyWindow(hwnd);
                return 0;
            }
        }
        WM_SIZE => {
            // 调整内部 EDIT 子控件填满客户区
            if let Some(ref state) = PANEL_STATE {
                if state.preview_hwnd == hwnd {
                    let edit_hwnd = state.preview_edit_hwnd;
                    if edit_hwnd.0 != 0 {
                        let width = LOWORD(l as u32) as i32;
                        let height = HIWORD(l as u32) as i32;
                        SetWindowPos(edit_hwnd, HWND_TOP, 0, 0, width, height, SWP_SHOWWINDOW);
                    }
                }
            }
        }
        WM_DESTROY => {
            // 清除状态
            if let Some(ref mut state) = PANEL_STATE {
                if state.preview_hwnd == hwnd {
                    state.preview_hwnd = HWND(0);
                    state.preview_edit_hwnd = HWND(0);
                    state.preview_edit_orig_proc = 0;
                }
            }
            return 0;
        }
        _ => {}
    }
    DefWindowProcW(hwnd, msg, w, l)
}

/// EDIT 子控件的子类过程（仅拦截 Escape 关闭预览窗口）
unsafe extern "system" fn preview_edit_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_KEYDOWN => {
            let vk = (w as u32 & 0xFFFF) as u16;
            if vk == VK_ESCAPE {
                // 获取父级预览弹出窗口并销毁它
                if let Some(ref mut state) = PANEL_STATE {
                    if state.preview_edit_hwnd == hwnd {
                        DestroyWindow(state.preview_hwnd);
                    }
                }
                return 0;
            }
        }
        _ => {}
    }
    // 其余消息（包含 WM_DESTROY）交给原始 EDIT 过程
    let orig = PANEL_STATE.as_ref().map(|s| s.preview_edit_orig_proc).unwrap_or(0);
    if orig != 0 {
        CallWindowProcW(orig, hwnd, msg, w, l)
    } else {
        DefWindowProcW(hwnd, msg, w, l)
    }
}

/// 关闭预览弹出窗口
unsafe fn close_preview() {
    if let Some(ref mut state) = PANEL_STATE {
        if state.preview_hwnd.0 != 0 {
            DestroyWindow(state.preview_hwnd);
            state.preview_hwnd = HWND(0);
        }
    }
}

/// 显示内容预览弹出窗（只读、可滚动、约 7 行高度）
unsafe fn show_preview(record_id: i64) {
    // 关闭之前的预览
    close_preview();

    let state = match PANEL_STATE.as_ref() { Some(s) => s, None => return };

    // 从数据库获取记录全文 + 时间
    let (content, created_at) = if let Ok(app) = state.app_state.lock() {
        let repo = Repository::new(&app.db);
        match repo.find_by_id(record_id) {
            Ok(Some(record)) => {
                let text = match record.content_type.as_str() {
                    "image" => "[图片]".to_string(),
                    _ => record.content_text.unwrap_or_else(|| "[空]".to_string()),
                };
                (text, record.created_at)
            }
            _ => return,
        }
    } else { return };

    // 截断过长内容
    let display = if content.chars().count() > MAX_PREVIEW_CHARS {
        let truncated: String = content.chars().take(MAX_PREVIEW_CHARS).collect();
        format!("{}\n\n...（内容过长，仅显示前 {} 字符）", truncated, MAX_PREVIEW_CHARS)
    } else {
        content
    };

    // 获取面板位置
    let mut panel_rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    GetWindowRect(state.hwnd, &mut panel_rect);

    // 从列表框获取字体
    let hfont = SendMessageW(state.list_hwnd, WM_GETFONT, wparam(0), lparam(0));

    // 先标记，防止 CreateWindowExW 触发的 WM_ACTIVATE 误判
    if let Some(ref mut s) = PANEL_STATE {
        s.preview_opening = true;
    }

    // 1. 创建预览弹出窗口（带标题栏、关闭按钮、可调整大小）
    let preview_hwnd = CreateWindowExW(
        WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
        w("ClipBoardX_Preview").as_ptr(),
        w("").as_ptr(), // 标题在 SetWindowTextW 时设置
        WS_POPUP | WS_VISIBLE | WS_CAPTION | WS_SYSMENU | WS_SIZEBOX,
        0, 0, 600, 400,
        HWND(0),
        HMENU(0),
        GetModuleHandleW(std::ptr::null()),
        std::ptr::null(),
    );
    if let Some(ref mut s) = PANEL_STATE {
        s.preview_opening = false;
    }
    if preview_hwnd.0 == 0 { return; }

    // 设置窗口标题为时间
    let title_str = format!("{}", created_at.format("%Y-%m-%d %H:%M"));
    let title_wide = w(&title_str);
    SetWindowTextW(preview_hwnd, title_wide.as_ptr());

    // 2. 创建 EDIT 子控件（只读、多行、可垂直滚动）
    let edit_hwnd = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        w("EDIT").as_ptr(),
        std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_VSCROLL | ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL | ES_NOHIDESEL | ES_DISABLENOSCROLL,
        0, 0, 600, 400,
        preview_hwnd,
        HMENU(0),
        GetModuleHandleW(std::ptr::null()),
        std::ptr::null(),
    );
    if edit_hwnd.0 == 0 { DestroyWindow(preview_hwnd); return; }

    // 3. 设置字体
    if hfont != 0 {
        SendMessageW(edit_hwnd, WM_SETFONT, wparam(hfont as u32), lparam(0));
    }

    // 4. 设置文本内容（创建后设置，确保换行计算正确）
    let wide = w(&display);
    SetWindowTextW(edit_hwnd, wide.as_ptr());

    // 5. 让编辑控件重新计算排版（换行），确保 WS_VSCROLL 滚动条正确显示
    SendMessageW(edit_hwnd, EM_SETTARGETDEVICE, wparam(0), lparam(0));

    // 6. 子类化 EDIT 控件，仅处理 Escape 关闭
    let orig_edit = SetWindowLongPtrW(edit_hwnd, GWLP_WNDPROC, preview_edit_proc as isize);
    if let Some(ref mut state) = PANEL_STATE {
        state.preview_edit_hwnd = edit_hwnd;
        state.preview_edit_orig_proc = orig_edit;
    }

    // 定位：面板右侧，若超出屏幕则放在左侧
    let preview_w = 600i32;
    let preview_h = 400i32;
    let mut x = panel_rect.right + 2;
    let y = panel_rect.top;
    // 粗略判断屏幕宽度（取面板所在监视器）
    let monitor = MonitorFromPoint(POINT { x: panel_rect.left, y: panel_rect.top }, 0);
    let mut mi = MONITORINFO { cbSize: 0, rcMonitor: RECT { left: 0, top: 0, right: 0, bottom: 0 }, rcWork: RECT { left: 0, top: 0, right: 0, bottom: 0 }, dwFlags: 0 };
    mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    if GetMonitorInfoW(monitor, &mut mi) != FALSE && x + preview_w > mi.rcWork.right {
        x = panel_rect.left - preview_w - 2;
        if x < mi.rcWork.left {
            x = mi.rcWork.left + 4; // 紧贴左边缘
        }
    }

    SetWindowPos(preview_hwnd, HWND_TOPMOST, x, y, preview_w, preview_h, SWP_SHOWWINDOW | SWP_NOACTIVATE);

    // 保存句柄
    if let Some(ref mut state) = PANEL_STATE {
        state.preview_hwnd = preview_hwnd;
    }
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
        // 隐藏/显示"确认删除"和"设置"按钮（互斥，避免重叠抢点击）
        ShowWindow(state.btn_confirm, if state.delete_mode { SW_SHOW } else { SW_HIDE });
        ShowWindow(state.btn_settings, if state.delete_mode { SW_HIDE } else { SW_SHOW });
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
        state.in_dialog = true;
        let msg = w(&format!("确定要删除 {} 条记录吗？", state.checked_ids.len()));
        let title = w("确认删除");
        let ret = MessageBoxW(state.hwnd, msg.as_ptr(), title.as_ptr(), MB_YESNO | MB_ICONWARNING);
        state.in_dialog = false;
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
        ShowWindow(state.btn_settings, SW_SHOW);
    }
    refresh_list();
}

/// 切换当前选中记录的勾选状态
unsafe fn toggle_check_selected() {
    // 第一步：只读方式获取当前选中记录的 ID
    let (item_id, top_index) = {
        let state = match PANEL_STATE.as_ref() { Some(s) => s, None => return };
        if !state.delete_mode { return; }
        let sel = SendMessageW(state.list_hwnd, LB_GETCURSEL, wparam(0), lparam(0));
        if sel < 0 { return; }
        let id = SendMessageW(state.list_hwnd, LB_GETITEMDATA, wparam(sel as u32), lparam(0)) as i64;
        let top = SendMessageW(state.list_hwnd, LB_GETTOPINDEX, wparam(0), lparam(0)) as i32;
        (id, top)
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
    // 恢复滚动位置，避免勾选时跳到顶部
    if let Some(ref state) = PANEL_STATE {
        SendMessageW(state.list_hwnd, LB_SETTOPINDEX, wparam(top_index as u32), lparam(0));
    }
}

/// 列表框子类化窗口过程 -- 拦截 Escape 键
unsafe extern "system" fn list_box_proc(
    hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM,
) -> LRESULT {
    if msg == WM_LBUTTONDOWN {
        let x = ((l as u32) & 0xFFFF) as i32;
        let y = (((l as u32) >> 16) & 0xFFFF) as i32;
        let mut rc = RECT::default();
        GetClientRect(hwnd, &mut rc);
        let width = rc.right;

        // 获取点击位置所在的列表项索引
        let lparam_val = MAKELPARAM(x as u16, y as u16);
        let result = SendMessageW(hwnd, LB_ITEMFROMPOINT, wparam(0), lparam(lparam_val));
        let item_idx = (result & 0xFFFF) as i32;
        if item_idx < 0 { return 0; }
        let item_id = SendMessageW(hwnd, LB_GETITEMDATA, wparam(item_idx as u32), lparam(0));
        if item_id == -1 { return 0; }

        // star area: width-48 ~ width-24 → 切换收藏
        if x >= width - 48 && x < width - 24 {
            if let Some(ref state) = PANEL_STATE {
                if let Ok(mut app) = state.app_state.lock() {
                    app.toggle_favorite(item_id as i64);
                }
            }
            refresh_list();
            // 恢复滚动位置
            SendMessageW(hwnd, LB_SETTOPINDEX, wparam(item_idx as u32), lparam(0));
            return 0;
        }
        // ">" area: rightmost 24px → 打开预览
        if x >= width - 24 && x < width {
            show_preview(item_id as i64);
            return 0;
        }
        // otherwise fall through to default
    }
    if msg == WM_KEYDOWN {
        let vk = (w as u32 & 0xFFFF) as u16;
        match vk {
            VK_ESCAPE => {
                if let Some(ref state) = PANEL_STATE {
                    hide_panel(state.hwnd);
                }
                return 0;
            }

            _ => {}
        }
    }
    // 其余消息走默认列表框处理
    let orig = PANEL_STATE.as_ref().map(|s| s.list_orig_proc).unwrap_or(0);
    if orig == 0 {
        DefWindowProcW(hwnd, msg, w, l)
    } else {
        CallWindowProcW(orig, hwnd, msg, w, l)
    }
}

/// 搜索框子类化窗口过程 -- ESC 清空/隐藏, 下箭头跳转列表框
unsafe extern "system" fn search_edit_proc(
    hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM,
) -> LRESULT {
    if msg == WM_KEYDOWN {
        let vk = (w as u32 & 0xFFFF) as u16;
        match vk {
            VK_ESCAPE => {
                if let Some(ref state) = PANEL_STATE {
                    // 清空搜索框内容
                    SetWindowTextW(hwnd, std::ptr::null());
                    if let Ok(mut app) = state.app_state.lock() {
                        app.search_keyword.clear();
                    }
                    refresh_list();
                    hide_panel(state.hwnd);
                }
                return 0;
            }
            VK_DOWN => {
                // 下箭头：焦点跳转到列表框
                if let Some(ref state) = PANEL_STATE {
                    SetFocus(state.list_hwnd);
                }
                return 0;
            }
            _ => {}
        }
    }
    // 其余消息走默认编辑框处理
    let orig = PANEL_STATE.as_ref().map(|s| s.search_orig_proc).unwrap_or(0);
    if orig == 0 {
        DefWindowProcW(hwnd, msg, w, l)
    } else {
        CallWindowProcW(orig, hwnd, msg, w, l)
    }
}

unsafe extern "system" fn panel_wnd_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_CLOSE => { hide_panel(hwnd); 0 }
        WM_NCHITTEST => {
            // 获取鼠标点击的屏幕坐标
            let l32 = l as u32;
            let screen_x = LOWORD(l32) as i16 as i32;
            let screen_y = HIWORD(l32) as i16 as i32;
            let mut pt = POINT { x: screen_x, y: screen_y };
            ScreenToClient(hwnd, &mut pt);
            // 判断是否点击在子控件上（列表框 / 搜索框 / 按钮），让它们正常处理
            // 顶部区域（搜索框 + 收藏过滤按钮）：(4,6)-(456,30)
            if pt.x >= 4 && pt.x <= 456 && pt.y >= 6 && pt.y <= 30 {
                return DefWindowProcW(hwnd, msg, w, l);
            }
            // 列表框区域：(0,34)-(460,452)
            if pt.x >= 0 && pt.x <= 460 && pt.y >= 34 && pt.y <= 452 {
                return DefWindowProcW(hwnd, msg, w, l);
            }
            // 按钮行区域：y=[456,484]
            if pt.y >= 456 && pt.y <= 484 {
                return DefWindowProcW(hwnd, msg, w, l);
            }
            // 其余空白区域允许拖拽移动窗口
            HTCAPTION as LRESULT
        }
        WM_MEASUREITEM => {
            // lParam points to MEASUREITEMSTRUCT
            let mis = &mut *(l as *mut MEASUREITEMSTRUCT);
            mis.itemHeight = 24; // each item 24 pixels tall
            1
        }
        WM_DRAWITEM => {
            let dis = &*(l as *const DRAWITEMSTRUCT);
            if dis.itemID == 0xFFFFFFFF { return 1; } // LB_ERR
            let hdc = dis.hDC;

            // 获取记录 ID，用于查询收藏状态
            let item_data = SendMessageW(dis.hwndItem, LB_GETITEMDATA, wparam(dis.itemID), lparam(0));
            let is_fav = if item_data != -1 {
                if let Some(ref state) = PANEL_STATE {
                    if let Ok(app) = state.app_state.lock() {
                        app.records.iter().any(|r| r.id == item_data as i64 && r.is_favorite)
                    } else { false }
                } else { false }
            } else { false };

            // Fill background
            if (dis.itemState & ODS_SELECTED) != 0 {
                FillRect(hdc, &dis.rcItem, GetSysColorBrush(COLOR_HIGHLIGHT));
                SetTextColor(hdc, 0x00FFFFFF); // white text on highlight
            } else {
                FillRect(hdc, &dis.rcItem, GetSysColorBrush(COLOR_WINDOW));
                SetTextColor(hdc, 0x00000000); // black text on white
            }
            SetBkMode(hdc, TRANSPARENT);

            // Draw item text (left side, leave space for star + ">" on right)
            let mut txt_rc = dis.rcItem;
            txt_rc.left += 4;
            txt_rc.right -= 48; // leave space for star (24px) + ">" (24px)

            let mut buf = [0u16; 512];
            let len = SendMessageW(dis.hwndItem, LB_GETTEXT, wparam(dis.itemID), lparam(buf.as_mut_ptr() as isize));
            if len > 0 && len < 512 {
                DrawTextW(hdc, buf.as_ptr(), len as i32, &mut txt_rc, DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX);
            }

            // Draw star (★ or ☆) in the 24px area left of ">"
            let mut star_rc = dis.rcItem;
            star_rc.left = star_rc.right - 48;
            star_rc.right = star_rc.right - 24;
            if (dis.itemState & ODS_SELECTED) != 0 {
                SetTextColor(hdc, 0x0000D7FF); // gold/yellow on selected (0x00BBGGRR)
            } else if is_fav {
                SetTextColor(hdc, 0x0000D7FF); // gold/yellow when favorited (0x00BBGGRR)
            } else {
                SetTextColor(hdc, 0x00CCCCCC); // light gray when not favorited
            }
            let star_char = if is_fav { [0x2605u16, 0u16] } else { [0x2606u16, 0u16] }; // ★ / ☆
            DrawTextW(hdc, star_char.as_ptr(), 1, &mut star_rc, DT_SINGLELINE | DT_VCENTER | DT_CENTER);

            // Draw ">" button
            let mut btn_rc = dis.rcItem;
            btn_rc.left = btn_rc.right - 24;
            if (dis.itemState & ODS_SELECTED) != 0 {
                SetTextColor(hdc, 0x00AAAAAA); // gray when selected
            } else {
                SetTextColor(hdc, 0x00000000); // black otherwise
            }
            let btn_txt = [0x003Eu16, 0u16]; // ">" + null
            DrawTextW(hdc, btn_txt.as_ptr(), 1, &mut btn_rc, DT_SINGLELINE | DT_VCENTER | DT_CENTER);

            1
        }
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
                ID_BTN_SETTINGS if code == BN_CLICKED => {
                    if let Some(ref state) = PANEL_STATE {
                        let hinst = GetModuleHandleW(std::ptr::null());
                        let _ = settings_window::create_settings_window(hinst, state.hwnd, state.app_state.clone());
                    }
                    0
                }
                ID_BTN_FAV_FILTER if code == BN_CLICKED => {
                    if let Some(ref state) = PANEL_STATE {
                        if let Ok(mut app) = state.app_state.lock() {
                            app.favorite_filter = !app.favorite_filter;
                            let text = if app.favorite_filter { "★ 收藏" } else { "☆ 收藏" };
                            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                            SetWindowTextW(state.btn_fav_filter, wide.as_ptr());
                        }
                    }
                    refresh_list();
                    0
                }
                ID_SEARCH_EDIT if code == EN_CHANGE => {
                    if let Some(ref state) = PANEL_STATE {
                        let txt = crate::ui::search_bar::get_search_text(state.search_hwnd);
                        if let Ok(mut app) = state.app_state.lock() {
                            app.search_keyword = txt;
                        }
                    }
                    refresh_list();
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
                                drop(app);
                                writeback_only(item_id as i64);
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
                _ => DefWindowProcW(hwnd, msg, w, l),
            }
        }
        WM_ACTIVATE => {
            if (w & 0xFFFF) as u32 == WA_INACTIVE {
                // 如果是预览窗口激活（点击预览窗口标题栏等），面板不隐藏
                let deactivated_hwnd = HWND(l as isize);
                if let Some(ref state) = PANEL_STATE {
                    if state.preview_opening || state.in_dialog || deactivated_hwnd == state.preview_hwnd {
                        return DefWindowProcW(hwnd, msg, w, l);
                    }
                }
                hide_panel(hwnd);
            }
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
