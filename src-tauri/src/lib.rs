#![allow(dead_code)]

pub mod models;
pub mod db;
pub mod ocr;
pub mod translator;
pub mod tokenizer;
pub mod utils;
pub mod commands;
pub mod image_inpaint;

use std::sync::Arc;

pub use models::{AppResult, AppError};

pub struct AppState {
    pub ocr_mode: String,
    pub translate_mode: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::try_init();

    let ocr_mode = if ocr::OCR.is_fallback() {
        "模拟模式".to_string()
    } else {
        "Tesseract引擎".to_string()
    };

    let translate_mode = if translator::TRANSLATOR.is_fallback() {
        "字典模式".to_string()
    } else {
        "Bergamot/NLLB引擎".to_string()
    };

    log::info!("OCR引擎状态: {}", ocr_mode);
    log::info!("翻译引擎状态: {}", translate_mode);
    log::info!("SQLite数据库路径: 应用数据目录/ocr_translate.db");

    let state = Arc::new(AppState {
        ocr_mode,
        translate_mode,
    });

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::init_database,
            commands::save_image,
            commands::run_ocr,
            commands::get_ocr_blocks,
            commands::query_blocks_in_region,
            commands::translate_text,
            commands::translate_blocks_in_region,
            commands::save_translation,
            commands::search_fulltext,
            commands::get_image_history,
            commands::get_image_by_id,
            commands::get_image_by_hash,
            commands::get_translations,
            commands::init_inpainter,
            commands::export_inpainted_image,
            commands::list_available_fonts,
        ])
        .run(tauri::generate_context!())
        .expect("启动Tauri应用失败");
}
