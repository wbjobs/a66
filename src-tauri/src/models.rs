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
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
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
    pub text_tokenized: String,
    pub paragraph: String,
    pub paragraph_idx: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_path: Option<String>,
    pub font_size: f32,
    pub color: (u8, u8, u8),
    pub stroke_color: Option<(u8, u8, u8)>,
    pub stroke_width: f32,
    pub bold: bool,
    pub italic: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_path: None,
            font_size: 24.0,
            color: (0, 0, 0),
            stroke_color: Some((255, 255, 255)),
            stroke_width: 1.0,
            bold: false,
            italic: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InpaintOptions {
    pub inpaint_radius: i32,
    pub text_style: TextStyle,
    pub export_format: String,
    pub quality: u8,
}

impl Default for InpaintOptions {
    fn default() -> Self {
        Self {
            inpaint_radius: 3,
            text_style: TextStyle::default(),
            export_format: "png".to_string(),
            quality: 95,
        }
    }
}
