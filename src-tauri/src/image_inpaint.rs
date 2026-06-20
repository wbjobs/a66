use std::path::Path;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{GenericImageView, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use crate::models::{AppResult, AppError, BlockWithTranslation, InpaintOptions};

#[cfg(feature = "opencv")]
use opencv::{
    core::{self, Mat, Scalar, Vec3b},
    imgcodecs,
    imgproc,
    photo,
    prelude::*,
};

pub struct ImageInpainter {
    font: Vec<u8>,
}

impl ImageInpainter {
    pub fn new() -> Self {
        let font_data = if let Some(path) = find_default_font() {
            std::fs::read(&path).unwrap_or_default()
        } else {
            Vec::new()
        };
        
        Self { font: font_data }
    }

    pub fn try_new() -> AppResult<Self> {
        let font_data = if let Some(path) = find_default_font() {
            std::fs::read(&path)
                .map_err(|e| AppError::Image(format!("读取系统字体失败: {}", e)))?
        } else {
            return Err(AppError::Image(
                "未找到可用的系统字体，请指定字体路径".to_string()
            ));
        };

        FontRef::try_from_slice(&font_data)
            .map_err(|e| AppError::Image(format!("字体加载失败: {}", e)))?;
        
        Ok(Self { font: font_data })
    }

    pub fn with_custom_font(font_path: &str) -> AppResult<Self> {
        let font_data = std::fs::read(font_path)
            .map_err(|e| AppError::Image(format!("读取字体文件失败: {}", e)))?;
        Ok(Self { font: font_data })
    }

    pub fn process_image(
        &self,
        image_path: &str,
        translations: &[BlockWithTranslation],
        options: &InpaintOptions,
        output_path: &str,
    ) -> AppResult<String> {
        if self.font.is_empty() {
            return Err(AppError::Image(
                "字体未加载，请指定有效的字体路径".to_string()
            ));
        }

        let font = FontRef::try_from_slice(&self.font)
            .map_err(|e| AppError::Image(format!("字体无效: {}", e)))?;

        let img = image::open(image_path)
            .map_err(|e| AppError::Image(format!("读取图片失败: {}", e)))?;
        
        let (img_w, img_h) = img.dimensions();
        let mut rgba_img = img.to_rgba8();

        let blocks_with_pixels: Vec<_> = translations.iter()
            .map(|bt| {
                let block = &bt.block;
                let px = (block.x * img_w as f64) as i32;
                let py = (block.y * img_h as f64) as i32;
                let pw = (block.width * img_w as f64).max(1.0) as i32;
                let ph = (block.height * img_h as f64).max(1.0) as i32;
                (px, py, pw, ph, bt.translation.translated_text.clone())
            })
            .collect();

        #[cfg(feature = "opencv")]
        {
            self.inpaint_with_opencv(&mut rgba_img, &blocks_with_pixels, options)?;
        }
        
        #[cfg(not(feature = "opencv"))]
        {
            self.inpaint_fallback(&mut rgba_img, &blocks_with_pixels, options)?;
        }

        self.draw_translated_text(&font, &mut rgba_img, &blocks_with_pixels, options)?;

        let output_path_buf = std::path::PathBuf::from(output_path);
        let format = match options.export_format.to_lowercase().as_str() {
            "jpg" | "jpeg" => image::ImageFormat::Jpeg,
            "png" => image::ImageFormat::Png,
            _ => image::ImageFormat::Png,
        };
        
        rgba_img.save_with_format(&output_path_buf, format)
            .map_err(|e| AppError::Image(format!("保存图片失败: {}", e)))?;

        Ok(output_path_buf.to_string_lossy().to_string())
    }

    #[cfg(feature = "opencv")]
    fn inpaint_with_opencv(
        &self,
        img: &mut RgbaImage,
        blocks: &[(i32, i32, i32, i32, String)],
        options: &InpaintOptions,
    ) -> AppResult<()> {
        let (w, h) = img.dimensions();
        
        let mut mat = Self::rgba_image_to_mat(img)?;
        let mut mask = Mat::zeros(h as i32, w as i32, core::CV_8UC1)?.to_mat();

        for &(px, py, pw, ph, _) in blocks {
            let rect = core::Rect {
                x: px,
                y: py,
                width: pw,
                height: ph,
            };
            imgproc::rectangle(
                &mut mask,
                rect,
                Scalar::all(255.0),
                -1,
                imgproc::LINE_8,
                0,
            )?;
        }

        let mut inpainted = Mat::default();
        photo::inpaint(
            &mat,
            &mask,
            &mut inpainted,
            options.inpaint_radius as f64,
            photo::INPAINT_TELEA,
        )?;

        Self::mat_to_rgba_image(&inpainted, img)?;
        
        Ok(())
    }

    #[cfg(feature = "opencv")]
    fn rgba_image_to_mat(img: &RgbaImage) -> AppResult<Mat> {
        let (w, h) = img.dimensions();
        let mut mat = unsafe {
            Mat::new_rows_cols_with_data(
                h as i32,
                w as i32,
                core::CV_8UC4,
                img.as_ptr() as *mut _,
                core::Mat_AUTO_STEP,
            )?
        };
        imgproc::cvt_color(&mat.clone(), &mut mat, imgproc::COLOR_RGBA2BGR, 0)?;
        Ok(mat)
    }

    #[cfg(feature = "opencv")]
    fn mat_to_rgba_image(mat: &Mat, img: &mut RgbaImage) -> AppResult<()> {
        let (w, h) = img.dimensions();
        let mut rgba_mat = Mat::default();
        imgproc::cvt_color(mat, &mut rgba_mat, imgproc::COLOR_BGR2RGBA, 0)?;
        
        let data = rgba_mat.data_bytes()?;
        for y in 0..h {
            for x in 0..w {
                let idx = (y * w + x) as usize * 4;
                let pixel = Rgba([
                    data[idx],
                    data[idx + 1],
                    data[idx + 2],
                    data[idx + 3],
                ]);
                img.put_pixel(x, y, pixel);
            }
        }
        
        Ok(())
    }

    #[cfg(not(feature = "opencv"))]
    fn inpaint_fallback(
        &self,
        img: &mut RgbaImage,
        blocks: &[(i32, i32, i32, i32, String)],
        options: &InpaintOptions,
    ) -> AppResult<()> {
        let (w, h) = img.dimensions();
        let radius = options.inpaint_radius.max(1);
        
        for &(px, py, pw, ph, _) in blocks {
            let start_x = px.max(0) as u32;
            let start_y = py.max(0) as u32;
            let end_x = (px + pw).min(w as i32) as u32;
            let end_y = (py + ph).min(h as i32) as u32;

            for y in start_y..end_y {
                for x in start_x..end_x {
                    let neighbors = self.get_neighbor_color(img, x, y, radius, w, h);
                    img.put_pixel(x, y, neighbors);
                }
            }
        }
        
        Ok(())
    }

    #[cfg(not(feature = "opencv"))]
    fn get_neighbor_color(
        &self,
        img: &RgbaImage,
        cx: u32,
        cy: u32,
        radius: i32,
        w: u32,
        h: u32,
    ) -> Rgba<u8> {
        let mut r_sum = 0u32;
        let mut g_sum = 0u32;
        let mut b_sum = 0u32;
        let mut a_sum = 0u32;
        let mut count = 0u32;

        let r = radius as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy > r * r {
                    continue;
                }
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                    if dx.abs() == r || dy.abs() == r {
                        let pixel = img.get_pixel(nx as u32, ny as u32);
                        r_sum += pixel[0] as u32;
                        g_sum += pixel[1] as u32;
                        b_sum += pixel[2] as u32;
                        a_sum += pixel[3] as u32;
                        count += 1;
                    }
                }
            }
        }

        if count == 0 {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([
                (r_sum / count) as u8,
                (g_sum / count) as u8,
                (b_sum / count) as u8,
                (a_sum / count) as u8,
            ])
        }
    }

    fn draw_translated_text<F: Font>(
        &self,
        font: &F,
        img: &mut RgbaImage,
        blocks: &[(i32, i32, i32, i32, String)],
        options: &InpaintOptions,
    ) -> AppResult<()> {
        let style = &options.text_style;
        let base_font_size = style.font_size;
        let text_color = Rgba([style.color.0, style.color.1, style.color.2, 255]);

        for &(px, py, pw, ph, ref text) in blocks {
            let region_w = pw as u32;
            let region_h = ph as u32;
            
            let lines = self.wrap_text(&font, text, region_w, base_font_size);
            
            let total_text_h = lines.len() as f32 * base_font_size * 1.2;
            let start_y = py as f32 + (region_h as f32 - total_text_h) / 2.0;

            for (i, line) in lines.iter().enumerate() {
                let y = start_y + i as f32 * base_font_size * 1.2;
                
                if let Some(stroke_color) = style.stroke_color {
                    let stroke = Rgba([stroke_color.0, stroke_color.1, stroke_color.2, 255]);
                    let sw = style.stroke_width.max(0.5);
                    for dy in [-sw, 0.0, sw] {
                        for dx in [-sw, 0.0, sw] {
                            if dx.abs() < 0.1 && dy.abs() < 0.1 {
                                continue;
                            }
                            self.draw_text_line(
                                img, &font, line,
                                px as f32 + dx, y + dy,
                                region_w, base_font_size,
                                stroke,
                            );
                        }
                    }
                }

                self.draw_text_line(
                    img, &font, line,
                    px as f32, y,
                    region_w, base_font_size,
                    text_color,
                );
            }
        }

        Ok(())
    }

    fn draw_text_line<F: Font>(
        &self,
        img: &mut RgbaImage,
        font: &F,
        text: &str,
        x: f32,
        y: f32,
        max_w: u32,
        font_size: f32,
        color: Rgba<u8>,
    ) {
        let scale = PxScale::from(font_size);
        let scaled_font = font.as_scaled(scale);
        
        let text_w: f32 = text.chars()
            .map(|c| scaled_font.h_advance(scaled_font.glyph_id(c)))
            .sum();
        
        let centered_x = x + (max_w as f32 - text_w) / 2.0;

        draw_text_mut(
            img,
            color,
            centered_x as i32,
            y as i32,
            scale,
            font,
            text,
        );
    }

    fn wrap_text<F: Font>(
        &self,
        font: &F,
        text: &str,
        max_width: u32,
        font_size: f32,
    ) -> Vec<String> {
        let scale = PxScale::from(font_size);
        let scaled_font = font.as_scaled(scale);
        
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0.0;
        let max_w = max_width as f32;

        for ch in text.chars() {
            let glyph_id = scaled_font.glyph_id(ch);
            let advance = scaled_font.h_advance(glyph_id);
            
            if ch == '\n' || current_width + advance > max_w {
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                    current_width = 0.0;
                }
                if ch != '\n' {
                    current_line.push(ch);
                    current_width = advance;
                }
            } else {
                current_line.push(ch);
                current_width += advance;
            }
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        lines
    }
}

fn find_default_font() -> Option<std::path::PathBuf> {
    let candidates = if cfg!(windows) {
        vec![
            r"C:\Windows\Fonts\msyh.ttc",
            r"C:\Windows\Fonts\msyh.ttf",
            r"C:\Windows\Fonts\simhei.ttf",
            r"C:\Windows\Fonts\simsun.ttc",
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
        ]
    } else {
        vec![
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        ]
    };

    for path in candidates {
        let p = Path::new(path);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }
    
    None
}

fn include_default_font() -> Vec<u8> {
    Vec::new()
}

impl Default for ImageInpainter {
    fn default() -> Self {
        Self::new()
    }
}

pub static INPAINTER: once_cell::sync::Lazy<std::sync::Mutex<Option<ImageInpainter>>> = 
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

pub fn init_inpainter(font_path: Option<&str>) -> AppResult<()> {
    let inpainter = if let Some(path) = font_path {
        ImageInpainter::with_custom_font(path)?
    } else {
        ImageInpainter::try_new()?
    };
    
    *INPAINTER.lock().unwrap() = Some(inpainter);
    Ok(())
}

pub fn get_inpainter() -> AppResult<std::sync::MutexGuard<'static, Option<ImageInpainter>>> {
    let guard = INPAINTER.lock().unwrap();
    if guard.is_none() {
        drop(guard);
        init_inpainter(None)?;
        Ok(INPAINTER.lock().unwrap())
    } else {
        Ok(guard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text() {
        let inpainter = ImageInpainter::new();
        let font = FontRef::try_from_slice(&inpainter.font).unwrap();
        let text = "这是一段很长的测试文字，需要自动换行显示在指定宽度内";
        let lines = inpainter.wrap_text(&font, text, 200, 24.0);
        assert!(!lines.is_empty());
        println!("Wrapped into {} lines", lines.len());
        for line in &lines {
            println!("  {}", line);
        }
    }

    #[test]
    fn test_inpainter_creation() {
        let inpainter = ImageInpainter::new();
        assert!(!inpainter.font.is_empty());
    }
}
