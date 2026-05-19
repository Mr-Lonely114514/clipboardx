use serde::{Deserialize, Serialize};

/// 交互模式
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InteractionMode {
    /// 自动粘贴模式（默认）：单击后直接写入剪切板 + 自动粘贴
    AutoPaste,
    /// 确认后粘贴模式：单击仅写入剪切板（不隐藏面板），双击/回车才粘贴
    ConfirmThenPaste,
}

impl Default for InteractionMode {
    fn default() -> Self {
        InteractionMode::AutoPaste
    }
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 全局热键修饰键 (MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN 的组合)
    pub hotkey_modifiers: u32,
    /// 全局热键虚拟键码 (默认 VK_V = 0x56)
    pub hotkey_vk: u16,
    /// 交互模式
    pub interaction_mode: InteractionMode,
    /// 最大保留条数，-1 表示无限制
    pub max_records: i64,
    /// 图片存储开关（F-01 特性开关）
    pub image_storage_enabled: bool,
    /// 忽略应用列表（F-17）
    pub ignore_list: Vec<IgnoreRule>,
}

/// 忽略应用规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreRule {
    /// 规则类型: "process_name" | "full_path" | "window_title"
    pub rule_type: String,
    /// 匹配模式（支持通配符）
    pub pattern: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // 默认 Alt+V (MOD_ALT=0x0001, VK_V=0x56)
            hotkey_modifiers: 0x0001,
            hotkey_vk: 0x56,
            interaction_mode: InteractionMode::default(),
            max_records: 5000,
            image_storage_enabled: true,
            ignore_list: Vec::new(),
        }
    }
}

impl Config {
    /// 配置文件路径
    pub fn config_path() -> std::path::PathBuf {
        let mut path = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("clipboardx.exe"));
        path.set_extension("json");
        path
    }

    /// 从 JSON 文件加载配置，若文件不存在则返回默认配置
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => return config,
                        Err(e) => {
                            log::warn!("配置解析失败 ({}), 使用默认配置", e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("读取配置文件失败 ({}), 使用默认配置", e);
                }
            }
        }
        Self::default()
    }

    /// 保存配置到 JSON 文件
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
