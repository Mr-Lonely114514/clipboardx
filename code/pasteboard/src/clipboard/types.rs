use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 剪切板内容的数据表示
#[derive(Debug, Clone)]
pub enum ClipData {
    /// 纯文本内容
    Text(String),
    /// 图片二进制数据 (BMP/PNG 等)
    Image(Vec<u8>),
    /// 文件路径 (从 CF_HDROP 或文本路径检测得到)
    File(String),
}

/// 内容类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
    File,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Text => "text",
            ContentType::Image => "image",
            ContentType::File => "file",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "image" => ContentType::Image,
            "file" => ContentType::File,
            _ => ContentType::Text,
        }
    }
}

/// 从剪切板提取的原始内容（多格式共存时的中间表示）
#[derive(Debug, Clone)]
pub struct RawClipContent {
    /// 纯文本内容（UTF-16 解码后）
    pub text: Option<String>,
    /// 图片二进制（DIB 或 BMP）
    pub image: Option<Vec<u8>>,
    /// 文件路径列表（从 CF_HDROP 提取）
    pub file_paths: Option<Vec<PathBuf>>,
    /// 是否有 CF_HDROP 格式
    pub has_hdrop: bool,
    /// 来源应用名称
    pub source_app: Option<String>,
}

impl RawClipContent {
    /// 根据 F-01 和 F-07 规则确定最终的 content_type
    pub fn determine_content_type(&self) -> ContentType {
        // 规则优先级: text > image > file
        if self.text.is_some() {
            ContentType::Text
        } else if self.image.is_some() {
            ContentType::Image
        } else if self.file_paths.is_some() {
            ContentType::File
        } else {
            // 兜底：默认文本
            ContentType::Text
        }
    }

    /// 获取用于 content_text 字段的值
    pub fn get_content_text(&self) -> Option<String> {
        if let Some(ref text) = self.text {
            if !text.is_empty() {
                return Some(text.clone());
            }
        }
        // 若纯文本为空但有文件路径，取首个文件路径字符串
        if let Some(ref paths) = self.file_paths {
            if let Some(first) = paths.first() {
                return Some(first.to_string_lossy().to_string());
            }
        }
        None
    }

    /// 获取用于 content_blob 字段的值（图片数据）
    pub fn get_content_blob(&self, image_storage_enabled: bool) -> Option<Vec<u8>> {
        if !image_storage_enabled {
            return None;
        }
        self.image.clone()
    }
}
