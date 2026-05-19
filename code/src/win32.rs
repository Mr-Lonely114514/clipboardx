//! Windows API 手动 FFI 声明
//! 使用 extern "system" 直接链接 Win32 API，避免 windows crate 的类型包装问题

#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ffi::c_void;

// ─── 基础类型 ───
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HWND(pub isize);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HINSTANCE(pub isize);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HMENU(pub isize);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HICON(pub isize);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HCURSOR(pub isize);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HFONT(pub isize);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HBRUSH(pub isize);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HGLOBAL(pub isize);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HANDLE(pub isize);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HIMAGELIST(pub isize);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HMONITOR(pub isize);

pub type HDC = isize;
pub type COLORREF = u32;

pub type BOOL = i32;
pub type LRESULT = isize;
pub type LPARAM = isize;
pub type WPARAM = usize;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
}

impl Default for MSG {
    fn default() -> Self {
        Self {
            hwnd: HWND(0),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DRAWITEMSTRUCT {
    pub CtlType: u32,
    pub CtlID: u32,
    pub itemID: u32,
    pub itemAction: u32,
    pub itemState: u32,
    pub hwndItem: HWND,
    pub hDC: HDC,
    pub rcItem: RECT,
    pub itemData: isize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MEASUREITEMSTRUCT {
    pub CtlType: u32,
    pub CtlID: u32,
    pub itemID: u32,
    pub itemWidth: u32,
    pub itemHeight: u32,
    pub itemData: isize,
}

#[repr(C)]
pub struct MONITORINFO {
    pub cbSize: u32,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: u32,
}

#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: HWND,
    pub uID: u32,
    pub uFlags: u32,
    pub uCallbackMessage: u32,
    pub hIcon: HICON,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub uVersion: u32,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: [u8; 16],
    pub hBalloonIcon: HICON,
}

#[repr(C)]
pub struct WNDCLASSW {
    pub style: u32,
    pub lpfnWndProc: Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
}

// ─── INPUT 结构（SendInput） ───
#[repr(C)]
pub struct KEYBDINPUT {
    pub wVk: u16,
    pub wScan: u16,
    pub dwFlags: u32,
    pub time: u32,
    pub dwExtraInfo: usize,
}

#[repr(C)]
pub struct INPUT_u {
    pub ki: KEYBDINPUT,
}

impl INPUT_u {
    pub fn from_keybd(ki: KEYBDINPUT) -> Self {
        INPUT_u { ki }
    }
}

#[repr(C)]
pub struct INPUT {
    pub r#type: u32,
    pub u: INPUT_u,
}

// ─── 常量 ───
// 窗口样式
pub const WS_POPUP: u32 = 0x80000000;
pub const WS_BORDER: u32 = 0x00800000;
pub const WS_VISIBLE: u32 = 0x10000000;
pub const WS_CHILD: u32 = 0x40000000;
pub const WS_CAPTION: u32 = 0x00C00000;
pub const WS_SYSMENU: u32 = 0x00080000;
pub const WS_VSCROLL: u32 = 0x00200000;
pub const WS_TABSTOP: u32 = 0x00010000;

pub const WS_EX_NOACTIVATE: u32 = 0x08000000;
pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;
pub const WS_EX_LAYERED: u32 = 0x00080000;
pub const WS_EX_TOPMOST: u32 = 0x00000008;
pub const WS_EX_DLGMODALFRAME: u32 = 0x00000001;
pub const WS_EX_CLIENTEDGE: u32 = 0x00000200;
pub const WS_EX_STATICEDGE: u32 = 0x00020000;
pub const WS_EX_TRANSPARENT: u32 = 0x00000020;
pub const BN_CLICKED: u32 = 0;
pub const BS_PUSHBUTTON: u32 = 0x00000000;
pub const BS_DEFPUSHBUTTON: u32 = 0x00000001;
pub const BS_AUTOCHECKBOX: u32 = 0x00000003;
pub const BST_CHECKED: u32 = 0x0001;
pub const BST_UNCHECKED: u32 = 0x0000;
pub const BM_SETCHECK: u32 = 0x00F1;
pub const BM_GETCHECK: u32 = 0x00F0;
pub const CBS_DROPDOWNLIST: u32 = 0x0003;

// 类样式
pub const CS_HREDRAW: u32 = 0x0002;
pub const CS_VREDRAW: u32 = 0x0001;
pub const CS_DBLCLKS: u32 = 0x0008;

// 窗口消息
pub const WM_APP: u32 = 0x8000;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_SIZE: u32 = 0x0005;
pub const WM_NOTIFY: u32 = 0x004E;
pub const WM_ACTIVATE: u32 = 0x0006;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_GETFONT: u32 = 0x0031;
pub const WM_HOTKEY: u32 = 0x0312;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_LBUTTONDBLCLK: u32 = 0x0203;
pub const WM_RBUTTONUP: u32 = 0x0205;
pub const WM_PASTE: u32 = 0x0302;
pub const WA_INACTIVE: u32 = 0;

pub const SW_HIDE: i32 = 0;
pub const SW_SHOW: i32 = 5;

pub const SWP_SHOWWINDOW: u32 = 0x0040;
pub const SWP_HIDEWINDOW: u32 = 0x0080;
pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOACTIVATE: u32 = 0x0010;

pub const HWND_TOP: HWND = HWND(0);
pub const HWND_TOPMOST: HWND = HWND(-1isize as isize);
pub const HWND_BOTTOM: HWND = HWND(1);
pub const HWND_NOTOPMOST: HWND = HWND(-2isize as isize);

// 标准 ID
pub const IDC_ARROW: *const u16 = 32512usize as *const u16;
pub const IDC_IBEAM: *const u16 = 32513usize as *const u16;
pub const IDI_APPLICATION: *const u16 = 32512usize as *const u16;
pub const IDYES: i32 = 6;
pub const IDNO: i32 = 7;

// SHELL_NOTIFYICON
pub const NIF_ICON: u32 = 0x0002;
pub const NIF_MESSAGE: u32 = 0x0001;
pub const NIF_TIP: u32 = 0x0004;
pub const NIF_SHOWTIP: u32 = 0x0080;
pub const NIM_ADD: u32 = 0;
pub const NIM_DELETE: u32 = 2;
pub const NIM_MODIFY: u32 = 1;

// 菜单
pub const MF_STRING: u32 = 0x0000;
pub const MF_SEPARATOR: u32 = 0x0800;
pub const TPM_RETURNCMD: u32 = 0x0100;
pub const TPM_NONOTIFY: u32 = 0x0080;

// 消息框
pub const MB_OK: u32 = 0x00000000;
pub const MB_YESNO: u32 = 0x00000004;
pub const MB_ICONINFORMATION: u32 = 0x00000040;
pub const MB_ICONWARNING: u32 = 0x00000030;

// 控件样式
pub const SS_RIGHT: u32 = 0x0002;
pub const ES_LEFT: u32 = 0x0000;
pub const ES_AUTOHSCROLL: u32 = 0x0080;
pub const ES_MULTILINE: u32 = 0x0004;
pub const ES_AUTOVSCROLL: u32 = 0x0040;
pub const ES_NOHIDESEL: u32 = 0x0100;
pub const ES_READONLY: u32 = 0x0800;
pub const ES_CENTER: u32 = 0x0001;
pub const ES_NUMBER: u32 = 0x2000;
pub const CBS_HASSTRINGS: u32 = 0x0200;
pub const CB_ADDSTRING: u32 = 0x0143;
pub const CB_SETCURSEL: u32 = 0x014E;
pub const CB_GETCURSEL: u32 = 0x0147;
pub const LBS_NOTIFY: u32 = 0x0001;
pub const LBS_NOINTEGRALHEIGHT: u32 = 0x0100;
pub const LBS_OWNERDRAWFIXED: u32 = 0x0010;
pub const LBS_HASSTRINGS: u32 = 0x0040;

pub const WM_DRAWITEM: u32 = 0x002B;
pub const WM_MEASUREITEM: u32 = 0x002C;

pub const ODS_SELECTED: u32 = 0x0001;

// ListBox 消息
pub const LB_ADDSTRING: u32 = 0x0180;
pub const LB_INSERTSTRING: u32 = 0x0181;
pub const LB_DELETESTRING: u32 = 0x0182;
pub const LB_RESETCONTENT: u32 = 0x0184;
pub const LB_GETCURSEL: u32 = 0x0188;
pub const LB_SETCURSEL: u32 = 0x0186;
pub const LB_GETTEXT: u32 = 0x0189;
pub const LB_GETTEXTLEN: u32 = 0x018A;
pub const LB_GETCOUNT: u32 = 0x018B;
pub const LB_GETITEMDATA: u32 = 0x0199;
pub const LB_SETITEMDATA: u32 = 0x019A;
pub const LB_ITEMFROMPOINT: u32 = 0x01A9;

// ListBox 通知
pub const LBN_SELCHANGE: u32 = 1;
pub const LBN_DBLCLK: u32 = 2;

pub const DT_SINGLELINE: u32 = 0x00000020;
pub const DT_VCENTER: u32 = 0x00000004;
pub const DT_RIGHT: u32 = 0x00000002;
pub const DT_CENTER: u32 = 0x00000001;
pub const DT_END_ELLIPSIS: u32 = 0x00008000;
pub const DT_NOPREFIX: u32 = 0x00000800;

pub const COLOR_WINDOW: i32 = 5;
pub const COLOR_WINDOWTEXT: i32 = 8;
pub const COLOR_HIGHLIGHT: i32 = 13;
pub const COLOR_HIGHLIGHTTEXT: i32 = 14;

pub const TRANSPARENT: i32 = 1;

// Edit 通知
pub const EN_CHANGE: u32 = 0x0300;

// 剪切板格式
pub const CF_TEXT: u32 = 1;
pub const CF_BITMAP: u32 = 2;
pub const CF_DIB: u32 = 8;
pub const CF_UNICODETEXT: u32 = 13;
pub const CF_HDROP: u32 = 15;

// 内存分配
pub const GMEM_MOVEABLE: u32 = 0x0002;

// 输入
pub const INPUT_KEYBOARD: u32 = 1;
pub const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;
pub const KEYEVENTF_KEYUP: u32 = 0x0002;

// 虚拟键码
pub const VK_BACK: u16 = 0x08;
pub const VK_TAB: u16 = 0x09;
pub const VK_CLEAR: u16 = 0x0C;
pub const VK_RETURN: u16 = 0x0D;
pub const VK_SHIFT: u16 = 0x10;
pub const VK_CONTROL: u16 = 0x11;
pub const VK_MENU: u16 = 0x12;
pub const VK_PAUSE: u16 = 0x13;
pub const VK_CAPITAL: u16 = 0x14;
pub const VK_ESCAPE: u16 = 0x1B;
pub const VK_SPACE: u16 = 0x20;
pub const VK_PRIOR: u16 = 0x21;
pub const VK_NEXT: u16 = 0x22;
pub const VK_END: u16 = 0x23;
pub const VK_HOME: u16 = 0x24;
pub const VK_LEFT: u16 = 0x25;
pub const VK_UP: u16 = 0x26;
pub const VK_RIGHT: u16 = 0x27;
pub const VK_DOWN: u16 = 0x28;
pub const VK_DELETE: u16 = 0x2E;
pub const VK_0: u16 = 0x30;
pub const VK_1: u16 = 0x31;
pub const VK_2: u16 = 0x32;
pub const VK_3: u16 = 0x33;
pub const VK_4: u16 = 0x34;
pub const VK_5: u16 = 0x35;
pub const VK_6: u16 = 0x36;
pub const VK_7: u16 = 0x37;
pub const VK_8: u16 = 0x38;
pub const VK_9: u16 = 0x39;
pub const VK_A: u16 = 0x41;
pub const VK_B: u16 = 0x42;
pub const VK_C: u16 = 0x43;
pub const VK_D: u16 = 0x44;
pub const VK_E: u16 = 0x45;
pub const VK_F: u16 = 0x46;
pub const VK_G: u16 = 0x47;
pub const VK_H: u16 = 0x48;
pub const VK_I: u16 = 0x49;
pub const VK_J: u16 = 0x4A;
pub const VK_K: u16 = 0x4B;
pub const VK_L: u16 = 0x4C;
pub const VK_M: u16 = 0x4D;
pub const VK_N: u16 = 0x4E;
pub const VK_O: u16 = 0x4F;
pub const VK_P: u16 = 0x50;
pub const VK_Q: u16 = 0x51;
pub const VK_R: u16 = 0x52;
pub const VK_S: u16 = 0x53;
pub const VK_T: u16 = 0x54;
pub const VK_U: u16 = 0x55;
pub const VK_V: u16 = 0x56;
pub const VK_W: u16 = 0x57;
pub const VK_X: u16 = 0x58;
pub const VK_Y: u16 = 0x59;
pub const VK_Z: u16 = 0x5A;

// MOD 常量（RegisterHotKey）
pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_WIN: u32 = 0x0008;
pub const MOD_NOREPEAT: u32 = 0x4000;

// GDI
pub const WHITE_BRUSH: u32 = 0;
pub const GWLP_WNDPROC: i32 = -4;
pub const LWA_ALPHA: u32 = 0x00000002;
pub const FW_NORMAL: u32 = 400;
pub const DEFAULT_CHARSET: u32 = 1;
pub const OUT_DEFAULT_PRECIS: u32 = 0;
pub const CLIP_DEFAULT_PRECIS: u32 = 0;
pub const DEFAULT_QUALITY: u32 = 0;
pub const DEFAULT_PITCH: u32 = 0;
pub const FF_DONTCARE: u32 = 0;

// Monitor
pub const MONITOR_DEFAULTTONULL: u32 = 0;
pub const MONITOR_DEFAULTTOPRIMARY: u32 = 1;
pub const MONITOR_DEFAULTTONEAREST: u32 = 2;

pub const MONITORINFOF_PRIMARY: u32 = 1;

// GDI 对象
pub const OBJ_BRUSH: u32 = 2;

// ─── 辅助函数 ───
pub fn wparam(v: u32) -> WPARAM { v as WPARAM }
pub fn lparam(v: isize) -> LPARAM { v as LPARAM }
pub fn LOWORD(dw: u32) -> u16 { (dw & 0xFFFF) as u16 }
pub fn HIWORD(dw: u32) -> u16 { ((dw >> 16) & 0xFFFF) as u16 }

pub fn MAKELPARAM(lo: u16, hi: u16) -> LPARAM {
    ((lo as u32) | ((hi as u32) << 16)) as LPARAM
}

pub fn RGB(r: u8, g: u8, b: u8) -> COLORREF {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

// ─── FFI 声明 ───

#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassW(wc: *const WNDCLASSW) -> u16;
    pub fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        x: i32, y: i32, nWidth: i32, nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: *const c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> BOOL;
    pub fn PeekMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32, wRemoveMsg: u32) -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    pub fn PostQuitMessage(nExitCode: i32);
    pub fn SetWindowPos(hWnd: HWND, hWndInsertAfter: HWND, X: i32, Y: i32, cx: i32, cy: i32, uFlags: u32) -> BOOL;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    pub fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> BOOL;
    pub fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: i32) -> i32;
    pub fn SetFocus(hWnd: HWND) -> HWND;
    pub fn GetDlgItem(hDlg: HWND, nIDDlgItem: i32) -> HWND;
    pub fn IsWindowVisible(hWnd: HWND) -> BOOL;
    pub fn GetForegroundWindow() -> HWND;
    pub fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    pub fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
    pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: *const u16) -> HCURSOR;
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: *const u16) -> HICON;
    pub fn GetStockObject(fnObject: i32) -> isize;
    pub fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    pub fn CreatePopupMenu() -> HMENU;
    pub fn AppendMenuW(hMenu: HMENU, uFlags: u32, uIDNewItem: usize, lpNewItem: *const u16) -> BOOL;
    pub fn TrackPopupMenu(hMenu: HMENU, uFlags: u32, x: i32, y: i32, nReserved: i32, hWnd: HWND, prcRect: *const RECT) -> BOOL;
    pub fn DestroyMenu(hMenu: HMENU) -> BOOL;
    pub fn MessageBoxW(hWnd: HWND, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
    pub fn CallWindowProcW(lpPrevWndFunc: isize, hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn SetLayeredWindowAttributes(hwnd: HWND, crKey: u32, bAlpha: u8, dwFlags: u32) -> BOOL;
    pub fn SendMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn PostMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub fn FindWindowExW(hWndParent: HWND, hWndChildAfter: HWND, lpszClass: *const u16, lpszWindow: *const u16) -> HWND;
    pub fn GetClassNameW(hWnd: HWND, lpClassName: *mut u16, nMaxCount: i32) -> i32;
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    pub fn MonitorFromPoint(pt: POINT, dwFlags: u32) -> HMONITOR;
    pub fn GetMonitorInfoW(hMonitor: HMONITOR, lpmi: *mut MONITORINFO) -> BOOL;
    pub fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;
    pub fn CloseClipboard() -> BOOL;
    pub fn GetClipboardData(uFormat: u32) -> HANDLE;
    pub fn SetClipboardData(uFormat: u32, hMem: HANDLE) -> HANDLE;
    pub fn EmptyClipboard() -> BOOL;
    pub fn AddClipboardFormatListener(hwnd: HWND) -> BOOL;
    pub fn RemoveClipboardFormatListener(hwnd: HWND) -> BOOL;
    pub fn RegisterHotKey(hwnd: HWND, id: i32, fsModifiers: u32, vk: u16) -> BOOL;
    pub fn UnregisterHotKey(hwnd: HWND, id: i32) -> BOOL;
    pub fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32;
    pub fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
    pub fn FillRect(hDC: HDC, lprc: *const RECT, hbr: isize) -> i32;
    pub fn DrawTextW(hDC: HDC, lpchText: *const u16, cchText: i32, lprc: *mut RECT, format: u32) -> i32;
    pub fn GetSysColorBrush(nIndex: i32) -> isize;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateSolidBrush(crColor: u32) -> HBRUSH;
    pub fn CreateFontW(
        cHeight: i32, cWidth: i32, cEscapement: i32, cOrientation: i32,
        cWeight: i32, bItalic: u32, bUnderline: u32, bStrikeOut: u32,
        iCharSet: u32, iOutPrecision: u32, iClipPrecision: u32,
        iQuality: u32, iPitchAndFamily: u32, pszFaceName: *const u16,
    ) -> HFONT;
    pub fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    pub fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> HGLOBAL;
    pub fn GlobalLock(hMem: HGLOBAL) -> *mut c_void;
    pub fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    pub fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    pub fn GlobalSize(hMem: HGLOBAL) -> usize;
    // 注意: GlobalSize 签名可能有歧义，用 GetClipboardData 返回的 HANDLE
}

#[link(name = "shell32")]
extern "system" {
    pub fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> BOOL;
    pub fn DragQueryFileW(hDrop: HANDLE, iFile: u32, lpszFile: *mut u16, cch: u32) -> u32;
    pub fn DragFinish(hDrop: HANDLE);
}
