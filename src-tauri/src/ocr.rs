use std::path::Path;
use image::GenericImageView;
use crate::models::*;
use crate::AppResult;
use crate::AppError;
use crate::tokenizer::tokenize_chinese;
use crate::utils::now_str;

pub struct OcrEngine {
    #[cfg(feature = "tesseract")]
    tess: Option<leptess::LepTess>,
    fallback_mode: bool,
}

impl OcrEngine {
    pub fn new() -> Self {
        #[cfg(feature = "tesseract")]
        {
            let tess = match leptess::LepTess::new(None, Some("eng+chi_sim")) {
                Ok(t) => {
                    log::info!("Tesseract OCR引擎初始化成功");
                    Some(t)
                }
                Err(e) => {
                    log::warn!("Tesseract初始化失败，使用模拟模式: {}", e);
                    None
                }
            };
            let fallback_mode = tess.is_none();
            Self { tess, fallback_mode }
        }
        #[cfg(not(feature = "tesseract"))]
        {
            log::info!("OCR模块运行在模拟模式（未启用tesseract feature）");
            Self { fallback_mode: true }
        }
    }

    pub fn is_fallback(&self) -> bool {
        self.fallback_mode
    }

    pub fn recognize(
        &self,
        image_path: &str,
        options: Option<&OcrProcessOptions>,
        image_id: i64,
    ) -> AppResult<Vec<OcrTextBlock>> {
        let path = Path::new(image_path);
        let img = image::open(path)
            .map_err(|e| AppError::Image(format!("无法打开图片: {}", e)))?;
        let (img_w, img_h) = img.dimensions();
        let img_w_f = img_w as f64;
        let img_h_f = img_h as f64;

        let _lang = options
            .and_then(|o| o.lang.as_deref())
            .unwrap_or("eng+chi_sim");

        let mut blocks = if self.fallback_mode {
            self.mock_recognize(img_w as i32, img_h as i32, image_id)
        } else {
            #[cfg(feature = "tesseract")]
            {
                if let Some(ref tess) = self.tess {
                    self.run_tesseract(tess.clone(), image_path, options, image_id, img_w, img_h)?
                } else {
                    self.mock_recognize(img_w as i32, img_h as i32, image_id)
                }
            }
            #[cfg(not(feature = "tesseract"))]
            {
                self.mock_recognize(img_w as i32, img_h as i32, image_id)
            }
        };

        for block in &mut blocks {
            block.x = normalize(block.x, img_w_f);
            block.y = normalize(block.y, img_h_f);
            block.width = normalize(block.width, img_w_f);
            block.height = normalize(block.height, img_h_f);
            block.text_tokenized = tokenize_chinese(&block.text);
        }

        Ok(blocks)
    }

    #[cfg(feature = "tesseract")]
    fn run_tesseract(
        &self,
        mut tess: leptess::LepTess,
        image_path: &str,
        options: Option<&OcrProcessOptions>,
        image_id: i64,
        _img_w: u32,
        _img_h: u32,
    ) -> AppResult<Vec<OcrTextBlock>> {
        if let Some(psm) = options.and_then(|o| o.psm) {
            use leptess::tesseract::PageSegMode;
            let mode = match psm {
                0 => PageSegMode::PsmOsdOnly,
                1 => PageSegMode::PsmAutoOsd,
                3 => PageSegMode::PsmAuto,
                4 => PageSegMode::PsmSingleColumn,
                5 => PageSegMode::PsmSingleBlockVertText,
                6 => PageSegMode::PsmSingleBlock,
                7 => PageSegMode::PsmSingleLine,
                8 => PageSegMode::PsmSingleWord,
                9 => PageSegMode::PsmCircleWord,
                10 => PageSegMode::PsmSingleChar,
                11 => PageSegMode::PsmSparseText,
                12 => PageSegMode::PsmSparseTextOsd,
                _ => PageSegMode::PsmAuto,
            };
            tess.set_page_seg_mode(mode);
        }

        tess.set_image(image_path)
            .map_err(|e| AppError::Ocr(format!("Tesseract set_image失败: {}", e)))?;

        let _ = tess.get_text();

        let boxes = tess
            .get_component_images(leptess::tesseract::PageIteratorLevel::Textline, 0)
            .map_err(|e| AppError::Ocr(format!("获取文字框失败: {}", e)))?;

        let now = now_str();
        let mut blocks: Vec<OcrTextBlock> = Vec::with_capacity(boxes.len());

        let mut paragraph_map: std::collections::BTreeMap<i32, Vec<usize>> =
            std::collections::BTreeMap::new();
        let mut sorted_indices: Vec<usize> = (0..boxes.len()).collect();
        sorted_indices.sort_by_key(|&i| {
            let b = &boxes[i];
            (b.1 / 30, b.0)
        });

        let mut current_para = 0i32;
        let mut last_y = -1i32;
        for &idx in &sorted_indices {
            let (x, y, w, h) = boxes[idx];
            if last_y >= 0 && (y - last_y).abs() > 40 {
                current_para += 1;
            }
            last_y = y;
            paragraph_map.entry(current_para).or_default().push(idx);
        }

        for (para_idx, indices) in &paragraph_map {
            let para_id = format!("p{}", para_idx);
            for (block_in_para, &orig_idx) in indices.iter().enumerate() {
                let (x, y, w, h) = boxes[orig_idx];
                tess.set_rectangle(
                    leptess::tesseract::Rect {
                        left: x as i32,
                        top: y as i32,
                        width: w as i32,
                        height: h as i32,
                    },
                );
                let text = tess.get_text().unwrap_or_default();
                let confidence = tess.mean_text_conf() as f64;
                let text = text.trim().to_string();
                if text.is_empty() {
                    continue;
                }
                blocks.push(OcrTextBlock {
                    id: 0,
                    image_id,
                    text,
                    text_tokenized: String::new(),
                    paragraph: para_id.clone(),
                    paragraph_idx: *para_idx * 100 + block_in_para as i32,
                    x: x as f64,
                    y: y as f64,
                    width: w as f64,
                    height: h as f64,
                    confidence,
                    lang: "tesseract".to_string(),
                    created_at: now.clone(),
                });
            }
        }

        Ok(blocks)
    }

    fn mock_recognize(
        &self,
        img_w: i32,
        img_h: i32,
        image_id: i64,
    ) -> Vec<OcrTextBlock> {
        let now = now_str();
        let sample_lines = vec![
            (
                "The quick brown fox jumps over the lazy dog.",
                "敏捷的棕色狐狸跳过了懒狗。",
            ),
            (
                "This is a sample OCR recognition result.",
                "这是一个示例OCR识别结果。",
            ),
            (
                "Hello world! Welcome to OCR Translate Desktop.",
                "你好世界！欢迎使用OCR翻译桌面版。",
            ),
            (
                "Artificial intelligence makes life easier.",
                "人工智能让生活更美好。",
            ),
            (
                "Tesseract OCR engine fallback mode.",
                "Tesseract OCR引擎回退模拟模式。",
            ),
        ];

        let n = sample_lines.len() as i32;
        let margin_x = (img_w as f64 * 0.08) as i32;
        let margin_y = (img_h as f64 * 0.1) as i32;
        let usable_w = img_w - margin_x * 2;
        let usable_h = img_h - margin_y * 2;
        let line_h = (usable_h as f64 / n as f64 * 0.85) as i32;
        let gap = (usable_h as f64 / n as f64 * 0.15) as i32;

        let mut blocks = Vec::with_capacity(sample_lines.len() * 2);

        for (i, (en, zh)) in sample_lines.iter().enumerate() {
            let i_i32 = i as i32;
            let y = margin_y + i_i32 * (line_h + gap);
            let half_w = usable_w / 2;

            blocks.push(OcrTextBlock {
                id: 0,
                image_id,
                text: en.to_string(),
                text_tokenized: String::new(),
                paragraph: format!("p{}", i),
                paragraph_idx: i_i32 * 100,
                x: margin_x as f64,
                y: y as f64,
                width: (half_w - 10) as f64,
                height: line_h as f64,
                confidence: 92.5 + (i as f64) * 0.5,
                lang: "eng".to_string(),
                created_at: now.clone(),
            });

            blocks.push(OcrTextBlock {
                id: 0,
                image_id,
                text: zh.to_string(),
                text_tokenized: String::new(),
                paragraph: format!("p{}", i),
                paragraph_idx: i_i32 * 100 + 1,
                x: (margin_x + half_w + 10) as f64,
                y: y as f64,
                width: (half_w - 10) as f64,
                height: line_h as f64,
                confidence: 90.0 + (i as f64) * 0.3,
                lang: "chi_sim".to_string(),
                created_at: now.clone(),
            });
        }

        blocks
    }
}

impl Default for OcrEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn normalize(px: f64, dim: f64) -> f64 {
    if dim <= 0.0 {
        0.0
    } else {
        (px / dim).clamp(0.0, 1.0)
    }
}

pub static OCR: once_cell::sync::Lazy<OcrEngine> = once_cell::sync::Lazy::new(OcrEngine::new);
