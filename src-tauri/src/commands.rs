use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use sha2::{Sha256, Digest};
use image::GenericImageView;
use crate::models::*;
use crate::db::DB;
use crate::ocr::OCR;
use crate::translator::TRANSLATOR;
use crate::utils::now_str;
use crate::{AppResult, AppError};

fn compute_hash(path: &str) -> AppResult<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn detect_mime(ext: &str) -> String {
    match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "png" => "image/png".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

#[tauri::command]
pub fn init_database() -> AppResult<()> {
    let _ = &*DB;
    Ok(())
}

#[tauri::command]
pub fn save_image(file_path: String) -> AppResult<ImageRecord> {
    let path = Path::new(&file_path);
    if !path.exists() {
        return Err(AppError::InvalidParam(format!(
            "文件不存在: {}",
            file_path
        )));
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();
    let mime = detect_mime(&ext);
    if ext.is_empty() || !mime.starts_with("image/") {
        return Err(AppError::InvalidParam(
            "只支持 JPEG/PNG 格式".to_string(),
        ));
    }

    let meta = fs::metadata(path)?;
    let file_size = meta.len() as i64;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    let hash = compute_hash(&file_path)?;

    let img = image::open(path)
        .map_err(|e| AppError::Image(format!("无法读取图片尺寸: {}", e)))?;
    let (width, height) = img.dimensions();

    let created_at = now_str();
    let created_at_clone = created_at.clone();

    let record = ImageRecord {
        id: 0,
        file_path: path.to_string_lossy().to_string(),
        file_name,
        file_size,
        width: width as i32,
        height: height as i32,
        hash: hash.clone(),
        mime_type: mime.clone(),
        created_at,
    };

    let id = DB.save_image(&record)?;

    Ok(ImageRecord {
        id,
        file_path: record.file_path,
        file_name: record.file_name,
        file_size,
        width: record.width,
        height: record.height,
        hash,
        mime_type: mime,
        created_at: created_at_clone,
    })
}

#[derive(Serialize, Deserialize)]
struct _UnusedRect {
    rect: (),
}

#[tauri::command]
pub fn run_ocr(
    image_id: i64,
    options: Option<OcrProcessOptions>,
) -> AppResult<Vec<OcrTextBlock>> {
    let image = DB.get_image(image_id)?;

    DB.delete_ocr_blocks(image_id)?;

    let blocks = OCR.recognize(&image.file_path, options.as_ref(), image_id)?;

    if blocks.is_empty() {
        return Ok(vec![]);
    }

    DB.save_ocr_blocks(&blocks)?;

    let stored = DB.get_ocr_blocks(image_id)?;

    Ok(stored)
}

#[tauri::command]
pub fn get_ocr_blocks(image_id: i64) -> AppResult<Vec<OcrTextBlock>> {
    DB.get_ocr_blocks(image_id)
}

#[tauri::command]
pub fn query_blocks_in_region(
    image_id: i64,
    rect: Rect,
    threshold: Option<f64>,
) -> AppResult<Vec<OcrTextBlock>> {
    let th = threshold.unwrap_or(0.5);
    DB.query_blocks_in_region(image_id, &rect, th)
}

#[tauri::command]
pub fn translate_text(
    source_text: String,
    options: Option<TranslateOptions>,
) -> AppResult<TranslateResult> {
    let text = source_text.trim();
    if text.is_empty() {
        return Err(AppError::InvalidParam("文本不能为空".to_string()));
    }

    let (translated, source_lang, target_lang) =
        TRANSLATOR.translate(text, options.as_ref())?;

    let result = TranslateResult {
        id: 0,
        ocr_block_id: 0,
        source_text: text.to_string(),
        translated_text: translated,
        source_lang,
        target_lang,
        created_at: now_str(),
    };

    Ok(result)
}

#[tauri::command]
pub fn translate_blocks_in_region(
    image_id: i64,
    rect: Rect,
    options: Option<TranslateOptions>,
) -> AppResult<Vec<BlockWithTranslation>> {
    let blocks = DB.query_blocks_in_region(image_id, &rect, 0.5)?;
    let mut results = Vec::with_capacity(blocks.len());

    for block in blocks {
        let text = block.text.trim();
        if text.is_empty() {
            continue;
        }

        let (translated, source_lang, target_lang) =
            TRANSLATOR.translate(text, options.as_ref())?;

        let mut t = TranslateResult {
            id: 0,
            ocr_block_id: block.id,
            source_text: text.to_string(),
            translated_text: translated,
            source_lang,
            target_lang,
            created_at: now_str(),
        };

        let saved_id = DB.save_translation(&t)?;
        t.id = saved_id;

        results.push(BlockWithTranslation {
            block,
            translation: t,
        });
    }

    Ok(results)
}

#[tauri::command]
pub fn save_translation(
    ocr_block_id: i64,
    source_text: String,
    translated_text: String,
    source_lang: String,
    target_lang: String,
) -> AppResult<TranslateResult> {
    let t = TranslateResult {
        id: 0,
        ocr_block_id,
        source_text: source_text.trim().to_string(),
        translated_text: translated_text.trim().to_string(),
        source_lang,
        target_lang,
        created_at: now_str(),
    };

    let id = DB.save_translation(&t)?;

    Ok(TranslateResult { id, ..t })
}

#[tauri::command]
pub fn search_fulltext(
    keyword: String,
    date_from: Option<String>,
    date_to: Option<String>,
    image_hash: Option<String>,
) -> AppResult<SearchResult> {
    DB.search_fulltext(
        &keyword,
        date_from.as_deref(),
        date_to.as_deref(),
        image_hash.as_deref(),
    )
}

#[tauri::command]
pub fn get_image_history(
    page: Option<i64>,
    page_size: Option<i64>,
) -> AppResult<HistoryResult> {
    let p = page.unwrap_or(1).max(1);
    let ps = page_size.unwrap_or(20).clamp(1, 100);
    let (images, total) = DB.list_images(p, ps)?;
    Ok(HistoryResult { images, total })
}

#[tauri::command]
pub fn get_image_by_id(image_id: i64) -> AppResult<ImageRecord> {
    DB.get_image(image_id)
}

#[tauri::command]
pub fn get_image_by_hash(hash: String) -> AppResult<Option<ImageRecord>> {
    DB.get_image_by_hash(&hash)
}

#[tauri::command]
pub fn get_translations(ocr_block_id: i64) -> AppResult<Vec<TranslateResult>> {
    DB.get_translations(ocr_block_id)
}
