use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::models::*;
use crate::AppResult;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: PathBuf) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;
            PRAGMA synchronous = NORMAL;

            CREATE TABLE IF NOT EXISTS images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                hash TEXT NOT NULL UNIQUE,
                mime_type TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_images_hash ON images(hash);
            CREATE INDEX IF NOT EXISTS idx_images_created_at ON images(created_at);

            CREATE TABLE IF NOT EXISTS ocr_blocks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                image_id INTEGER NOT NULL,
                text TEXT NOT NULL,
                paragraph TEXT NOT NULL,
                paragraph_idx INTEGER NOT NULL DEFAULT 0,
                x INTEGER NOT NULL,
                y INTEGER NOT NULL,
                width INTEGER NOT NULL,
                height INTEGER NOT NULL,
                confidence REAL NOT NULL DEFAULT 0,
                lang TEXT NOT NULL DEFAULT 'unknown',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_ocr_blocks_image_id ON ocr_blocks(image_id);
            CREATE INDEX IF NOT EXISTS idx_ocr_blocks_coords ON ocr_blocks(image_id, x, y);

            CREATE TABLE IF NOT EXISTS translations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ocr_block_id INTEGER NOT NULL,
                source_text TEXT NOT NULL,
                translated_text TEXT NOT NULL,
                source_lang TEXT NOT NULL,
                target_lang TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (ocr_block_id) REFERENCES ocr_blocks(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_translations_ocr_block ON translations(ocr_block_id);

            CREATE VIRTUAL TABLE IF NOT EXISTS ocr_blocks_fts USING fts5(
                text,
                content='ocr_blocks',
                content_rowid='id',
                tokenize='unicode61'
            );

            CREATE TRIGGER IF NOT EXISTS ocr_blocks_ai AFTER INSERT ON ocr_blocks BEGIN
                INSERT INTO ocr_blocks_fts(rowid, text) VALUES (new.id, new.text);
            END;

            CREATE TRIGGER IF NOT EXISTS ocr_blocks_ad AFTER DELETE ON ocr_blocks BEGIN
                INSERT INTO ocr_blocks_fts(ocr_blocks_fts, rowid, text) VALUES('delete', old.id, old.text);
            END;

            CREATE TRIGGER IF NOT EXISTS ocr_blocks_au AFTER UPDATE ON ocr_blocks BEGIN
                INSERT INTO ocr_blocks_fts(ocr_blocks_fts, rowid, text) VALUES('delete', old.id, old.text);
                INSERT INTO ocr_blocks_fts(rowid, text) VALUES (new.id, new.text);
            END;

            CREATE VIRTUAL TABLE IF NOT EXISTS translations_fts USING fts5(
                source_text,
                translated_text,
                content='translations',
                content_rowid='id',
                tokenize='unicode61'
            );

            CREATE TRIGGER IF NOT EXISTS translations_ai AFTER INSERT ON translations BEGIN
                INSERT INTO translations_fts(rowid, source_text, translated_text)
                VALUES (new.id, new.source_text, new.translated_text);
            END;

            CREATE TRIGGER IF NOT EXISTS translations_ad AFTER DELETE ON translations BEGIN
                INSERT INTO translations_fts(translations_fts, rowid, source_text, translated_text)
                VALUES('delete', old.id, old.source_text, old.translated_text);
            END;

            CREATE TRIGGER IF NOT EXISTS translations_au AFTER UPDATE ON translations BEGIN
                INSERT INTO translations_fts(translations_fts, rowid, source_text, translated_text)
                VALUES('delete', old.id, old.source_text, old.translated_text);
                INSERT INTO translations_fts(rowid, source_text, translated_text)
                VALUES (new.id, new.source_text, new.translated_text);
            END;
            "#,
        )?;
        Ok(())
    }

    pub fn save_image(&self, record: &ImageRecord) -> AppResult<i64> {
        let conn = self.conn.lock().unwrap();
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM images WHERE hash = ?1",
                params![record.hash],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(id) = existing {
            return Ok(id);
        }
        conn.execute(
            "INSERT INTO images (file_path, file_name, file_size, width, height, hash, mime_type, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.file_path,
                record.file_name,
                record.file_size,
                record.width,
                record.height,
                record.hash,
                record.mime_type,
                record.created_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_image(&self, id: i64) -> AppResult<ImageRecord> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, file_path, file_name, file_size, width, height, hash, mime_type, created_at
             FROM images WHERE id = ?1",
            params![id],
            row_to_image,
        )
        .map_err(|e| e.into())
    }

    pub fn get_image_by_hash(&self, hash: &str) -> AppResult<Option<ImageRecord>> {
        let conn = self.conn.lock().unwrap();
        Ok(conn
            .query_row(
                "SELECT id, file_path, file_name, file_size, width, height, hash, mime_type, created_at
                 FROM images WHERE hash = ?1",
                params![hash],
                row_to_image,
            )
            .optional()?)
    }

    pub fn list_images(&self, page: i64, page_size: i64) -> AppResult<(Vec<ImageRecord>, i64)> {
        let conn = self.conn.lock().unwrap();
        let total: i64 = conn.query_row("SELECT COUNT(*) FROM images", [], |row| row.get(0))?;
        let offset = (page - 1) * page_size;
        let mut stmt = conn.prepare(
            "SELECT id, file_path, file_name, file_size, width, height, hash, mime_type, created_at
             FROM images ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )?;
        let images = stmt
            .query_map(params![page_size, offset], row_to_image)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((images, total))
    }

    pub fn save_ocr_blocks(&self, blocks: &[OcrTextBlock]) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        for block in blocks {
            conn.execute(
                "INSERT INTO ocr_blocks (image_id, text, paragraph, paragraph_idx, x, y, width, height, confidence, lang, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    block.image_id,
                    block.text,
                    block.paragraph,
                    block.paragraph_idx,
                    block.x,
                    block.y,
                    block.width,
                    block.height,
                    block.confidence,
                    block.lang,
                    block.created_at,
                ],
            )?;
        }
        Ok(())
    }

    pub fn delete_ocr_blocks(&self, image_id: i64) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM ocr_blocks WHERE image_id = ?1", params![image_id])?;
        Ok(())
    }

    pub fn get_ocr_blocks(&self, image_id: i64) -> AppResult<Vec<OcrTextBlock>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, image_id, text, paragraph, paragraph_idx, x, y, width, height, confidence, lang, created_at
             FROM ocr_blocks WHERE image_id = ?1 ORDER BY paragraph_idx, y, x",
        )?;
        let blocks = stmt
            .query_map(params![image_id], row_to_block)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(blocks)
    }

    pub fn query_blocks_in_region(
        &self,
        image_id: i64,
        rect: &Rect,
        threshold: f64,
    ) -> AppResult<Vec<OcrTextBlock>> {
        let conn = self.conn.lock().unwrap();
        let rx1 = rect.x as f64;
        let ry1 = rect.y as f64;
        let rx2 = (rect.x + rect.width) as f64;
        let ry2 = (rect.y + rect.height) as f64;

        let mut stmt = conn.prepare(
            "SELECT id, image_id, text, paragraph, paragraph_idx, x, y, width, height, confidence, lang, created_at
             FROM ocr_blocks WHERE image_id = ?1",
        )?;
        let blocks: Vec<OcrTextBlock> = stmt
            .query_map(params![image_id], row_to_block)?
            .collect::<Result<Vec<_>, _>>()?;

        let filtered: Vec<OcrTextBlock> = blocks
            .into_iter()
            .filter(|b| {
                let bx1 = b.x as f64;
                let by1 = b.y as f64;
                let bx2 = (b.x + b.width) as f64;
                let by2 = (b.y + b.height) as f64;
                let ix1 = bx1.max(rx1);
                let iy1 = by1.max(ry1);
                let ix2 = bx2.min(rx2);
                let iy2 = by2.min(ry2);
                if ix2 <= ix1 || iy2 <= iy1 {
                    return false;
                }
                let inter = (ix2 - ix1) * (iy2 - iy1);
                let block_area = (bx2 - bx1) * (by2 - by1);
                inter / block_area >= threshold
            })
            .collect();
        Ok(filtered)
    }

    pub fn save_translation(&self, t: &TranslateResult) -> AppResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO translations (ocr_block_id, source_text, translated_text, source_lang, target_lang, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                t.ocr_block_id,
                t.source_text,
                t.translated_text,
                t.source_lang,
                t.target_lang,
                t.created_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_translations(&self, ocr_block_id: i64) -> AppResult<Vec<TranslateResult>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, ocr_block_id, source_text, translated_text, source_lang, target_lang, created_at
             FROM translations WHERE ocr_block_id = ?1 ORDER BY created_at DESC",
        )?;
        let list = stmt
            .query_map(params![ocr_block_id], row_to_translation)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(list)
    }

    pub fn search_fulltext(
        &self,
        keyword: &str,
        date_from: Option<&str>,
        date_to: Option<&str>,
        image_hash: Option<&str>,
    ) -> AppResult<SearchResult> {
        let conn = self.conn.lock().unwrap();
        let kw = keyword.trim();
        let has_kw = !kw.is_empty();

        let mut image_ids_from_blocks: Vec<i64> = Vec::new();
        let mut matched_blocks = Vec::new();

        if has_kw {
            let fts_query = format_fts_query(kw);
            let mut block_stmt = conn.prepare(
                "SELECT b.id, b.image_id, b.text, b.paragraph, b.paragraph_idx, b.x, b.y, b.width, b.height, b.confidence, b.lang, b.created_at
                 FROM ocr_blocks_fts f
                 JOIN ocr_blocks b ON b.id = f.rowid
                 WHERE ocr_blocks_fts MATCH ?1
                 ORDER BY rank",
            )?;
            let block_rows = block_stmt
                .query_map(params![fts_query], row_to_block)?
                .collect::<Result<Vec<OcrTextBlock>, _>>()?;
            for b in block_rows {
                image_ids_from_blocks.push(b.image_id);
                matched_blocks.push(b);
            }

            let mut trans_stmt = conn.prepare(
                "SELECT b.id, b.image_id, b.text, b.paragraph, b.paragraph_idx, b.x, b.y, b.width, b.height, b.confidence, b.lang, b.created_at
                 FROM translations_fts f
                 JOIN translations t ON t.id = f.rowid
                 JOIN ocr_blocks b ON b.id = t.ocr_block_id
                 WHERE translations_fts MATCH ?1
                 ORDER BY rank",
            )?;
            let trans_rows = trans_stmt
                .query_map(params![fts_query], row_to_block)?
                .collect::<Result<Vec<OcrTextBlock>, _>>()?;
            for b in trans_rows {
                if !matched_blocks.iter().any(|x| x.id == b.id) {
                    image_ids_from_blocks.push(b.image_id);
                    matched_blocks.push(b);
                }
            }
        }

        let mut sql =
            "SELECT DISTINCT i.id, i.file_path, i.file_name, i.file_size, i.width, i.height, i.hash, i.mime_type, i.created_at
             FROM images i".to_string();
        let mut conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        let mut param_idx = 1;

        if let Some(h) = image_hash {
            if !h.trim().is_empty() {
                conditions.push(format!("i.hash = ?{}", param_idx));
                param_values.push(Box::new(h.to_string()));
                param_idx += 1;
            }
        }

        if let Some(d) = date_from {
            if !d.trim().is_empty() {
                conditions.push(format!("i.created_at >= ?{}", param_idx));
                param_values.push(Box::new(format!("{} 00:00:00", d)));
                param_idx += 1;
            }
        }

        if let Some(d) = date_to {
            if !d.trim().is_empty() {
                conditions.push(format!("i.created_at <= ?{}", param_idx));
                param_values.push(Box::new(format!("{} 23:59:59", d)));
                param_idx += 1;
            }
        }

        if has_kw {
            if image_ids_from_blocks.is_empty() {
                return Ok(SearchResult {
                    images: vec![],
                    blocks: matched_blocks,
                });
            }
            let placeholders: Vec<String> = (0..image_ids_from_blocks.len())
                .map(|_| {
                    let p = format!("?{}", param_idx);
                    param_idx += 1;
                    p
                })
                .collect();
            for id in &image_ids_from_blocks {
                param_values.push(Box::new(*id));
            }
            conditions.push(format!("i.id IN ({})", placeholders.join(",")));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }
        sql.push_str(" ORDER BY i.created_at DESC LIMIT 500");

        let mut stmt = conn.prepare(&sql)?;
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            param_values.iter().map(|b| b.as_ref()).collect();
        let images = stmt
            .query_map(params_refs.as_slice(), row_to_image)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SearchResult {
            images,
            blocks: matched_blocks,
        })
    }
}

pub static DB: Lazy<Database> = Lazy::new(|| {
    let app_data_dir = tauri::api::path::app_data_dir(
        &tauri::generate_context!().config()
    ).expect("无法获取应用数据目录");
    let db_path = app_data_dir.join("ocr_translate.db");
    Database::new(db_path).expect("数据库初始化失败")
});

fn row_to_image(row: &rusqlite::Row) -> rusqlite::Result<ImageRecord> {
    Ok(ImageRecord {
        id: row.get(0)?,
        file_path: row.get(1)?,
        file_name: row.get(2)?,
        file_size: row.get(3)?,
        width: row.get(4)?,
        height: row.get(5)?,
        hash: row.get(6)?,
        mime_type: row.get(7)?,
        created_at: row.get(8)?,
    })
}

fn row_to_block(row: &rusqlite::Row) -> rusqlite::Result<OcrTextBlock> {
    Ok(OcrTextBlock {
        id: row.get(0)?,
        image_id: row.get(1)?,
        text: row.get(2)?,
        paragraph: row.get(3)?,
        paragraph_idx: row.get(4)?,
        x: row.get(5)?,
        y: row.get(6)?,
        width: row.get(7)?,
        height: row.get(8)?,
        confidence: row.get(9)?,
        lang: row.get(10)?,
        created_at: row.get(11)?,
    })
}

fn row_to_translation(row: &rusqlite::Row) -> rusqlite::Result<TranslateResult> {
    Ok(TranslateResult {
        id: row.get(0)?,
        ocr_block_id: row.get(1)?,
        source_text: row.get(2)?,
        translated_text: row.get(3)?,
        source_lang: row.get(4)?,
        target_lang: row.get(5)?,
        created_at: row.get(6)?,
    })
}

fn format_fts_query(kw: &str) -> String {
    let parts: Vec<&str> = kw.split_whitespace().collect();
    if parts.len() == 1 {
        format!("{}*", parts[0])
    } else {
        parts
            .iter()
            .map(|p| format!("{}*", p))
            .collect::<Vec<_>>()
            .join(" AND ")
    }
}
