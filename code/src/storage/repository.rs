use rusqlite::params;

use super::database::Database;
use super::models::{ClipRecord, NewClipRecord, SearchQuery};

/// 数据仓库：封装所有 CRUD 操作
pub struct Repository<'a> {
    pub(crate) db: &'a Database,
}

impl<'a> Repository<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// 插入新记录（若 content_hash 已存在则忽略）
    pub fn insert(&self, record: &NewClipRecord) -> Result<Option<i64>, rusqlite::Error> {
        let sql = "INSERT OR IGNORE INTO clip_records \
                   (content_hash, content_type, content_text, content_blob, source_app, source) \
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
        self.db.conn.execute(
            sql,
            params![
                record.content_hash,
                record.content_type,
                record.content_text,
                record.content_blob,
                record.source_app,
                record.source,
            ],
        )?;

        let id = self.db.conn.last_insert_rowid();
        // last_insert_rowid 返回 0 表示被 IGNORE 了
        if id == 0 {
            Ok(None)
        } else {
            Ok(Some(id))
        }
    }

    /// 根据 ID 查询记录
    pub fn find_by_id(&self, id: i64) -> Result<Option<ClipRecord>, rusqlite::Error> {
        let sql = "SELECT id, content_hash, content_type, content_text, content_blob, \
                   source_app, source, is_favorite, created_at, updated_at \
                   FROM clip_records WHERE id = ?1";
        let mut stmt = self.db.conn.prepare(sql)?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(ClipRecord {
                id: row.get(0)?,
                content_hash: row.get(1)?,
                content_type: row.get(2)?,
                content_text: row.get(3)?,
                content_blob: row.get(4)?,
                source_app: row.get(5)?,
                source: row.get(6)?,
                is_favorite: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })),
            None => Ok(None),
        }
    }

    /// 查询记录列表（按时间倒序，支持分页）
    pub fn list(&self, limit: i64, offset: i64) -> Result<Vec<ClipRecord>, rusqlite::Error> {
        let sql = "SELECT id, content_hash, content_type, content_text, content_blob, \
                   source_app, source, is_favorite, created_at, updated_at \
                   FROM clip_records ORDER BY created_at DESC LIMIT ?1 OFFSET ?2";
        let mut stmt = self.db.conn.prepare(sql)?;
        let rows = stmt.query_map(params![limit, offset], |row| {
            Ok(ClipRecord {
                id: row.get(0)?,
                content_hash: row.get(1)?,
                content_type: row.get(2)?,
                content_text: row.get(3)?,
                source_app: row.get(5)?,
                source: row.get(6)?,
                is_favorite: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                content_blob: None, // 列表查询不返回 BLOB 以节省内存（惰性加载）
            })
        })?;
        rows.collect()
    }

    /// 搜索记录（FTS5 全文搜索 + 联合过滤）
    pub fn search(&self, query: &SearchQuery) -> Result<Vec<ClipRecord>, rusqlite::Error> {
        let mut conditions = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        // 关键词搜索（FTS5）
        if let Some(ref keyword) = query.keyword {
            if !keyword.is_empty() {
                // FTS5 模糊搜索，大小写不敏感
                let fts_query = format!("\"{}\" OR {}*", keyword, keyword);
                conditions.push(format!(
                    "EXISTS (SELECT 1 FROM clip_records_fts \
                     WHERE clip_records_fts MATCH ?{idx} \
                     AND clip_records_fts.rowid = clip_records.id)",
                    idx = param_values.len() + 1
                ));
                param_values.push(Box::new(fts_query));
            }
        }

        // 内容类型过滤
        if let Some(ref ct) = query.content_type_filter {
            conditions.push(format!(
                "content_type = ?{}",
                param_values.len() + 1
            ));
            param_values.push(Box::new(ct.clone()));
        }

        // 仅收藏
        if query.favorite_only {
            conditions.push(format!("is_favorite = ?{}", param_values.len() + 1));
            param_values.push(Box::new(1i32));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        // 注意：FTS5 搜索结果无 BLOB（content_blob 为 NULL）
        let sql = format!(
            "SELECT id, content_hash, content_type, content_text, NULL, \
             source_app, source, is_favorite, created_at, updated_at \
             FROM clip_records {} ORDER BY created_at DESC LIMIT ?{} OFFSET ?{}",
            where_clause,
            param_values.len() + 1,
            param_values.len() + 2
        );

        let mut stmt = self.db.conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();

        let rows = stmt.query_map(
            rusqlite::params_from_iter(
                params_refs
                    .iter()
                    .chain(std::iter::once(&(&query.limit as &dyn rusqlite::types::ToSql)))
                    .chain(std::iter::once(&(&query.offset as &dyn rusqlite::types::ToSql))),
            ),
            |row| {
                Ok(ClipRecord {
                    id: row.get(0)?,
                    content_hash: row.get(1)?,
                    content_type: row.get(2)?,
                    content_text: row.get(3)?,
                    source_app: row.get(5)?,
                    source: row.get(6)?,
                    is_favorite: row.get::<_, i32>(7)? != 0,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    content_blob: None,
                })
            },
        )?;
        rows.collect()
    }

    /// 切换收藏状态
    pub fn toggle_favorite(&self, id: i64) -> Result<bool, rusqlite::Error> {
        self.db.conn.execute(
            "UPDATE clip_records SET is_favorite = CASE WHEN is_favorite = 0 THEN 1 ELSE 0 END, \
             updated_at = datetime('now','localtime') WHERE id = ?1",
            params![id],
        )?;
        // 返回新的收藏状态
        let new_val: i32 = self.db.conn.query_row(
            "SELECT is_favorite FROM clip_records WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(new_val != 0)
    }

    /// 删除单条记录
    pub fn delete(&self, id: i64) -> Result<bool, rusqlite::Error> {
        let affected = self
            .db
            .conn
            .execute("DELETE FROM clip_records WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    /// 清空所有记录
    pub fn delete_all(&self) -> Result<usize, rusqlite::Error> {
        let count = self.db.conn.execute("DELETE FROM clip_records", [])?;
        // 重建 FTS 索引
        self.db.conn.execute_batch(
            "DELETE FROM clip_records_fts; INSERT INTO clip_records_fts(clip_records_fts) VALUES('rebuild');"
        )?;
        Ok(count)
    }

    /// 淘汰最旧记录（根据 max_records 限制）
    pub fn trim_oldest(&self, max_records: i64) -> Result<usize, rusqlite::Error> {
        if max_records < 0 {
            return Ok(0); // 无限制模式
        }
        let count: i64 = self
            .db
            .conn
            .query_row("SELECT COUNT(*) FROM clip_records", [], |row| row.get(0))?;
        if count <= max_records {
            return Ok(0);
        }
        let to_delete = count - max_records;
        self.db.conn.execute(
            "DELETE FROM clip_records WHERE id IN (\
             SELECT id FROM clip_records ORDER BY created_at ASC LIMIT ?1\
             )",
            params![to_delete],
        )?;
        Ok(to_delete as usize)
    }

    /// 获取收藏列表
    pub fn list_favorites(&self) -> Result<Vec<ClipRecord>, rusqlite::Error> {
        let sql = "SELECT id, content_hash, content_type, content_text, content_blob, \
                   source_app, source, is_favorite, created_at, updated_at \
                   FROM clip_records WHERE is_favorite = 1 ORDER BY updated_at DESC";
        let mut stmt = self.db.conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(ClipRecord {
                id: row.get(0)?,
                content_hash: row.get(1)?,
                content_type: row.get(2)?,
                content_text: row.get(3)?,
                source_app: row.get(5)?,
                source: row.get(6)?,
                is_favorite: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                content_blob: None,
            })
        })?;
        rows.collect()
    }
}
