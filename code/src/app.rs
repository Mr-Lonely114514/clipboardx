use std::sync::{Arc, Mutex};

use crate::clipboard::listener::SkipMarker;
use crate::clipboard::types::ContentType;
use crate::storage::database::Database;
use crate::storage::models::ClipRecord;
use crate::storage::repository::Repository;
use crate::utils::config::{Config, InteractionMode};

/// 应用全局状态
pub struct AppState {
    /// 数据库
    pub db: Database,
    /// 配置
    pub config: Config,
    /// 写回跳过标记
    pub skip_marker: SkipMarker,
    /// 当前面板显示/隐藏状态
    pub panel_visible: bool,
    /// 当前显示的记录列表
    pub records: Vec<ClipRecord>,
    /// 搜索关键词
    pub search_keyword: String,
    /// 当前选中的记录索引
    pub selected_index: i64,
    /// 确认后粘贴模式下的"待粘贴"记录ID
    pub pending_paste_id: Option<i64>,
}

impl AppState {
    pub fn new(db_path: &str, config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Database::open(db_path)?;
        Ok(Self {
            db,
            config,
            skip_marker: SkipMarker::new(),
            panel_visible: false,
            records: Vec::new(),
            search_keyword: String::new(),
            selected_index: -1,
            pending_paste_id: None,
        })
    }

    /// 重新加载记录列表（根据当前搜索关键词）
    pub fn refresh_records(&mut self) {
        let repo = Repository::new(&self.db);
        let limit = if self.config.max_records > 0 && self.config.max_records < 10000 {
            self.config.max_records
        } else {
            10000
        };

        if self.search_keyword.is_empty() {
            match repo.list(limit, 0) {
                Ok(records) => self.records = records,
                Err(e) => log::error!("加载记录列表失败: {}", e),
            }
        } else {
            let query = crate::storage::models::SearchQuery {
                keyword: Some(self.search_keyword.clone()),
                content_type_filter: None,
                favorite_only: false,
                limit,
                offset: 0,
            };
            match repo.search(&query) {
                Ok(records) => self.records = records,
                Err(e) => log::error!("搜索记录失败: {}", e),
            }
        }
    }

    /// 删除单条记录
    pub fn delete_record(&mut self, id: i64) -> bool {
        let repo = Repository::new(&self.db);
        match repo.delete(id) {
            Ok(true) => {
                self.records.retain(|r| r.id != id);
                true
            }
            _ => false,
        }
    }

    /// 清空所有记录
    pub fn clear_all_records(&mut self) -> bool {
        let repo = Repository::new(&self.db);
        match repo.delete_all() {
            Ok(_) => {
                self.records.clear();
                true
            }
            Err(e) => {
                log::error!("清空记录失败: {}", e);
                false
            }
        }
    }

    /// 切换收藏状态
    pub fn toggle_favorite(&mut self, id: i64) -> bool {
        let repo = Repository::new(&self.db);
        match repo.toggle_favorite(id) {
            Ok(is_fav) => {
                if let Some(record) = self.records.iter_mut().find(|r| r.id == id) {
                    record.is_favorite = is_fav;
                }
                is_fav
            }
            Err(e) => {
                log::error!("切换收藏失败: {}", e);
                false
            }
        }
    }

    /// 检查无限制模式下的安全阈值（F-10）
    pub fn check_storage_limits(&mut self) -> Vec<StorageWarning> {
        let mut warnings = Vec::new();

        if self.config.max_records != -1 {
            return warnings;
        }

        // 检查记录数
        let repo = Repository::new(&self.db);
        if let Ok(count) = repo.count() {
            if count > 100_000 {
                warnings.push(StorageWarning::RecordCountExceeded(count));
            }
        }

        // 检查数据库文件大小
        if let Ok(size) = self.db.file_size() {
            if size > 1_000_000_000 {
                // 1 GB
                warnings.push(StorageWarning::FileSizeExceeded(size));
            }
        }

        warnings
    }
}

/// 存储警告
#[derive(Debug, Clone)]
pub enum StorageWarning {
    RecordCountExceeded(i64),
    FileSizeExceeded(u64),
}

/// Repository 的辅助方法
impl<'a> Repository<'a> {
    /// 获取记录总数
    pub fn count(&self) -> Result<i64, rusqlite::Error> {
        self.db
            .conn
            .query_row("SELECT COUNT(*) FROM clip_records", [], |row| row.get(0))
    }
}
