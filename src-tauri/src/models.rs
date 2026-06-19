use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("图片处理错误: {0}")]
    Image(String),
    #[error("OCR引擎错误: {0}")]
    Ocr(String),
    #[error("翻译引擎错误: {0}")]
    Translate(String),
    #[error("未找到记录: {0}")]
    NotFound(String),
    #[error("参数错误: {0}")]
    InvalidParam(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub width: i32,
    pub height: i32,
    pub hash: String,
    pub mime_type: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrTextBlock {
    pub id: i64,
    pub image_id: i64,
    pub text: String,
    pub paragraph: String,
    pub paragraph_idx: i32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub confidence: f64,
    pub lang: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateResult {
    pub id: i64,
    pub ocr_block_id: i64,
    pub source_text: String,
    pub translated_text: String,
    pub source_lang: String,
    pub target_lang: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub images: Vec<ImageRecord>,
    pub blocks: Vec<OcrTextBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResult {
    pub images: Vec<ImageRecord>,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrProcessOptions {
    pub lang: Option<String>,
    pub psm: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateOptions {
    pub source_lang: Option<String>,
    pub target_lang: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockWithTranslation {
    pub block: OcrTextBlock,
    pub translation: TranslateResult,
}
