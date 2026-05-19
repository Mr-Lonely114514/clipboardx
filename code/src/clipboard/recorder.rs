use std::ptr;

use sha2::{Digest, Sha256};

use crate::storage::database::Database;
use crate::storage::models::NewClipRecord;
use crate::storage::repository::Repository;
use crate::utils::config::Config;
use crate::win32::*;

use super::listener::SkipMarker;
use super::types::{ContentType, RawClipContent};

pub struct ClipboardRecorder<'a> {
    repo: Repository<'a>,
    skip_marker: &'a SkipMarker,
    config: &'a Config,
}

impl<'a> ClipboardRecorder<'a> {
    pub fn new(db: &'a Database, skip_marker: &'a SkipMarker, config: &'a Config) -> Self {
        Self { repo: Repository::new(db), skip_marker, config }
    }

    pub fn capture_and_save(&self) {
        if self.skip_marker.should_skip() { log::debug!("跳过自触发"); return; }

        let source_app = self.get_source_app_name();
        if self.is_app_ignored(&source_app) { log::debug!("忽略应用 [{}]", source_app); return; }

        let content = match self.read_clipboard() { Some(c) => c, None => return };
        let hash = self.compute_hash(&content);
        let content_type = content.determine_content_type();

        let record = NewClipRecord {
            content_hash: hash,
            content_type: content_type.as_str().to_string(),
            content_text: content.get_content_text(),
            content_blob: content.get_content_blob(self.config.image_storage_enabled),
            source_app: Some(source_app),
            source: "local_clipboard".to_string(),
        };

        match self.repo.insert(&record) {
            Ok(Some(id)) => {
                log::info!("保存记录 id={}, type={}", id, record.content_type);
                if let Err(e) = self.repo.trim_oldest(self.config.max_records) {
                    log::error!("淘汰旧记录失败: {}", e);
                }
            }
            Ok(None) => log::debug!("重复内容跳过"),
            Err(e) => log::error!("保存失败: {}", e),
        }
    }

    pub fn writeback_record(&self, record: &crate::storage::models::ClipRecord) {
        self.skip_marker.set();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.do_writeback(record)));
        self.skip_marker.clear();
        if let Err(e) = result { log::error!("写回 panic: {:?}", e); }
    }

    fn do_writeback(&self, record: &crate::storage::models::ClipRecord) {
        unsafe {
            if OpenClipboard(HWND(0)) == FALSE { log::error!("OpenClipboard 失败"); return; }
            EmptyClipboard();
            match record.content_type.as_str() {
                "text" | "file" => {
                    if let Some(ref text) = record.content_text {
                        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                        let bytes = wide.len() * 2;
                        let hglobal = GlobalAlloc(GMEM_MOVEABLE, bytes);
                        if hglobal.0 != 0 {
                            let dst = GlobalLock(hglobal);
                            if !dst.is_null() {
                                ptr::copy_nonoverlapping(wide.as_ptr(), dst as *mut u16, wide.len());
                                GlobalUnlock(hglobal);
                            }
                            SetClipboardData(CF_UNICODETEXT, HANDLE(hglobal.0));
                        }
                    }
                }
                "image" => {
                    if let Some(ref blob) = record.content_blob {
                        let hglobal = GlobalAlloc(GMEM_MOVEABLE, blob.len());
                        if hglobal.0 != 0 {
                            let dst = GlobalLock(hglobal);
                            if !dst.is_null() {
                                ptr::copy_nonoverlapping(blob.as_ptr(), dst as *mut u8, blob.len());
                                GlobalUnlock(hglobal);
                            }
                            SetClipboardData(CF_DIB, HANDLE(hglobal.0));
                        }
                    }
                }
                _ => {}
            }
            CloseClipboard();
        }
    }

    fn read_clipboard(&self) -> Option<RawClipContent> {
        unsafe {
            if OpenClipboard(HWND(0)) == FALSE { return None; }

            let mut content = RawClipContent {
                text: None, image: None, file_paths: None, has_hdrop: false, source_app: None,
            };

            // CF_HDROP
            let hdrop = GetClipboardData(CF_HDROP);
            if hdrop.0 != 0 {
                content.has_hdrop = true;
                let count = DragQueryFileW(hdrop, 0xFFFFFFFF, ptr::null_mut(), 0);
                let mut paths = Vec::new();
                for i in 0..count {
                    let len = DragQueryFileW(hdrop, i, ptr::null_mut(), 0);
                    if len > 0 {
                        let mut buf = vec![0u16; len as usize + 1];
                        DragQueryFileW(hdrop, i, buf.as_mut_ptr(), len + 1);
                        paths.push(std::path::PathBuf::from(String::from_utf16_lossy(&buf[..len as usize])));
                    }
                }
                if !paths.is_empty() { content.file_paths = Some(paths); }
            }

            // CF_UNICODETEXT
            let htext = GetClipboardData(CF_UNICODETEXT);
            if htext.0 != 0 {
                let hg = HGLOBAL(htext.0);
                let ptr = GlobalLock(hg);
                if !ptr.is_null() {
                    let size = GlobalSize(hg);
                    let u16_slice = std::slice::from_raw_parts(ptr as *const u16, size / 2);
                    let len = u16_slice.iter().position(|&c| c == 0).unwrap_or(u16_slice.len());
                    let s = String::from_utf16_lossy(&u16_slice[..len]);
                    if !s.is_empty() { content.text = Some(s); }
                    GlobalUnlock(hg);
                }
            }

            // CF_DIB
            let hdib = GetClipboardData(CF_DIB);
            if hdib.0 != 0 {
                let hg = HGLOBAL(hdib.0);
                let ptr = GlobalLock(hg);
                if !ptr.is_null() {
                    let size = GlobalSize(hg);
                    let data = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
                    if !data.is_empty() { content.image = Some(data); }
                    GlobalUnlock(hg);
                }
            }

            CloseClipboard();

            // 文本路径检测
            if let Some(ref text) = content.text {
                if looks_like_file_path(text) {
                    let p = std::path::Path::new(text);
                    if p.exists() { content.file_paths = Some(vec![p.to_path_buf()]); }
                }
            }

            if content.text.is_none() && content.image.is_none() && content.file_paths.is_none() {
                return None;
            }
            Some(content)
        }
    }

    fn compute_hash(&self, content: &RawClipContent) -> String {
        let mut h = Sha256::new();
        if let Some(ref t) = content.text { h.update(t.as_bytes()); }
        if let Some(ref i) = content.image { h.update(i); }
        if let Some(ref p) = content.file_paths { for path in p { h.update(path.to_string_lossy().as_bytes()); } }
        hex::encode(h.finalize())
    }

    fn get_source_app_name(&self) -> String {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0 == 0 { return "Unknown".to_string(); }
            let mut buf = [0u16; 256];
            let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), 256);
            if len > 0 { return String::from_utf16_lossy(&buf[..len as usize]); }
        }
        "Unknown".to_string()
    }

    fn is_app_ignored(&self, app_name: &str) -> bool {
        if self.config.ignore_list.is_empty() { return false; }
        let al = app_name.to_lowercase();
        for rule in &self.config.ignore_list {
            let p = rule.pattern.to_lowercase();
            match rule.rule_type.as_str() {
                "window_title" => { if wildcard_match(&al, &p) { return true; } }
                "process_name" => { if al.contains(&p) { return true; } }
                "full_path" => { if al == p { return true; } }
                _ => {}
            }
        }
        false
    }
}

fn looks_like_file_path(text: &str) -> bool {
    if text.len() >= 3 {
        let b = text.as_bytes();
        if b[1] == b':' && b[2] == b'\\' && ((b[0] >= b'A' && b[0] <= b'Z') || (b[0] >= b'a' && b[0] <= b'z')) { return true; }
    }
    text.starts_with("\\\\")
}

fn wildcard_match(text: &str, pattern: &str) -> bool {
    let tc: Vec<char> = text.chars().collect();
    let pc: Vec<char> = pattern.chars().collect();
    let mut ti = 0; let mut pi = 0;
    let mut sti = None; let mut spi = None;
    while ti < tc.len() {
        if pi < pc.len() && (pc[pi] == '?' || pc[pi] == tc[ti]) { ti += 1; pi += 1; }
        else if pi < pc.len() && pc[pi] == '*' { sti = Some(ti); spi = Some(pi); pi += 1; }
        else if let Some(st) = sti { ti = st + 1; sti = Some(ti); pi = spi.unwrap() + 1; }
        else { return false; }
    }
    while pi < pc.len() && pc[pi] == '*' { pi += 1; }
    pi == pc.len()
}
