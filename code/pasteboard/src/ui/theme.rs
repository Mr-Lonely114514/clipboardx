/// 主题颜色定义（RGB 值）
pub struct ThemeColors {
    pub background: u32,      // 背景色
    pub selected_bg: u32,     // 选中项背景
    pub hover_bg: u32,        // 悬停项背景
    pub text: u32,            // 文本颜色
    pub text_secondary: u32,  // 次要文本
    pub border: u32,          // 边框
    pub favorite: u32,        // 收藏金色
    pub icon_text: u32,       // 文本图标蓝
    pub icon_image: u32,      // 图片图标绿
    pub icon_file: u32,       // 文件图标橙
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            background: 0xF5F5F5,
            selected_bg: 0x0078D4,
            hover_bg: 0xE5E5E5,
            text: 0x000000,
            text_secondary: 0x666666,
            border: 0xD0D0D0,
            favorite: 0xFFC107,
            icon_text: 0x0078D4,
            icon_image: 0x107C10,
            icon_file: 0xFF8C00,
        }
    }
}

/// 面板尺寸常量
pub struct PanelSizes {
    pub width: i32,
    pub max_height: i32,
    pub search_height: i32,
    pub item_height: i32,
}

impl Default for PanelSizes {
    fn default() -> Self {
        Self { width: 420, max_height: 600, search_height: 36, item_height: 48 }
    }
}
