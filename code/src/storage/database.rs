use rusqlite::{Connection, Result};

/// 数据库管理器
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// 打开或创建 SQLite 数据库
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// 初始化表结构和 FTS5 全文索引
    fn initialize(&self) -> Result<()> {
        // 启用 WAL 模式提升并发性能
        self.conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // 启用外键约束
        self.conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        // 主表
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clip_records (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                content_hash    TEXT UNIQUE NOT NULL,
                content_type    TEXT NOT NULL DEFAULT 'text',
                content_text    TEXT,
                content_blob    BLOB,
                source_app      TEXT,
                source          TEXT NOT NULL DEFAULT 'local_clipboard',
                is_favorite     INTEGER NOT NULL DEFAULT 0,
                created_at      DATETIME NOT NULL DEFAULT (datetime('now','localtime')),
                updated_at      DATETIME NOT NULL DEFAULT (datetime('now','localtime'))
            );"
        )?;

        // 建立索引
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_clip_records_created_at ON clip_records(created_at DESC);
             CREATE INDEX IF NOT EXISTS idx_clip_records_content_type ON clip_records(content_type);
             CREATE INDEX IF NOT EXISTS idx_clip_records_is_favorite ON clip_records(is_favorite);"
        )?;

        // FTS5 全文搜索虚拟表
        self.conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS clip_records_fts USING fts5(
                content_text, source_app,
                content='clip_records',
                content_rowid='id'
            );"
        )?;

        // 自动同步 FTS 的触发器（插入时）
        self.conn.execute_batch(
            "CREATE TRIGGER IF NOT EXISTS clip_records_ai AFTER INSERT ON clip_records BEGIN
                INSERT INTO clip_records_fts(rowid, content_text, source_app)
                VALUES (new.id, new.content_text, new.source_app);
            END;"
        )?;

        // 自动同步 FTS 的触发器（删除时）
        self.conn.execute_batch(
            "CREATE TRIGGER IF NOT EXISTS clip_records_ad AFTER DELETE ON clip_records BEGIN
                INSERT INTO clip_records_fts(clip_records_fts, rowid, content_text, source_app)
                VALUES ('delete', old.id, old.content_text, old.source_app);
            END;"
        )?;

        // 自动同步 FTS 的触发器（更新时）
        self.conn.execute_batch(
            "CREATE TRIGGER IF NOT EXISTS clip_records_au AFTER UPDATE ON clip_records BEGIN
                INSERT INTO clip_records_fts(clip_records_fts, rowid, content_text, source_app)
                VALUES ('delete', old.id, old.content_text, old.source_app);
                INSERT INTO clip_records_fts(rowid, content_text, source_app)
                VALUES (new.id, new.content_text, new.source_app);
            END;"
        )?;

        Ok(())
    }

    /// 获取数据库文件大小（字节）
    pub fn file_size(&self) -> Result<u64> {
        let mut stmt = self.conn.prepare("PRAGMA page_count;")?;
        let page_count: i64 = stmt.query_row([], |row| row.get(0))?;
        let mut stmt = self.conn.prepare("PRAGMA page_size;")?;
        let page_size: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok((page_count as u64) * (page_size as u64))
    }

    /// 获取总记录数
    pub fn record_count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM clip_records", [], |row| row.get(0))
    }

    /// 执行 VACUUM 回收空间
    pub fn vacuum(&self) -> Result<()> {
        self.conn.execute_batch("VACUUM;")?;
        Ok(())
    }
}
