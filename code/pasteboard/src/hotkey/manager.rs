use std::sync::{Arc, Mutex};

use crate::app::AppState;
use crate::win32::*;

pub struct HotkeyManager {
    registered: bool,
}

pub const HOTKEY_ID: i32 = 1001;

impl HotkeyManager {
    pub fn new() -> Self { Self { registered: false } }

    pub fn register(&mut self, hwnd: HWND, modifiers: u32, vk: u16) -> Result<(), String> {
        if self.registered { let _ = self.unregister(hwnd); }
        unsafe {
            if RegisterHotKey(hwnd, HOTKEY_ID, modifiers, vk) != FALSE {
                self.registered = true; Ok(())
            } else {
                self.registered = false; Err("热键注册失败".to_string())
            }
        }
    }

    pub fn unregister(&mut self, hwnd: HWND) -> Result<(), String> {
        if self.registered {
            unsafe {
                if UnregisterHotKey(hwnd, HOTKEY_ID) != FALSE {
                    self.registered = false; Ok(())
                } else { Err("注销热键失败".to_string()) }
            }
        } else { Ok(()) }
    }

    pub fn detect_conflict(hwnd: HWND, modifiers: u32, vk: u16) -> bool {
        unsafe {
            if RegisterHotKey(hwnd, 9999, modifiers, vk) != FALSE {
                UnregisterHotKey(hwnd, 9999); false
            } else { true }
        }
    }

    pub fn hotkey_name(modifiers: u32, vk: u16) -> String {
        let mut parts = Vec::new();
        if modifiers & MOD_ALT != 0 { parts.push("Alt"); }
        if modifiers & MOD_CONTROL != 0 { parts.push("Ctrl"); }
        if modifiers & MOD_SHIFT != 0 { parts.push("Shift"); }
        if modifiers & MOD_WIN != 0 { parts.push("Win"); }
        let key = match vk {
        48..=57 => format!("{}", (vk as u8 - 48 + b'0') as char),
        65..=90 => format!("{}", (vk as u8 - 65 + b'A') as char),
            0x56 => "V".to_string(),
            0x70..=0x7B => format!("F{}", vk - 0x70 + 1),
            _ => format!("{:02X}", vk),
        };
        parts.push(&key);
        parts.join("+")
    }
}
