use std::sync::atomic::{AtomicBool, Ordering};

/// 写回跳过标记（F-01 写回跳过机制）
/// 使用内存标记位防止程序自身写回操作被重复捕获
pub struct SkipMarker {
    /// 标记位：true = 当前正在写回，监听器应跳过本轮捕获
    enabled: AtomicBool,
}

impl SkipMarker {
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
        }
    }

    /// 检查是否应跳过（检查并消费标记）
    pub fn should_skip(&self) -> bool {
        self.enabled.swap(false, Ordering::SeqCst)
    }

    /// 设置跳过标记（写回前调用）
    pub fn set(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }

    /// 清除跳过标记（写回完成后的 finally 中调用）
    pub fn clear(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }
}
