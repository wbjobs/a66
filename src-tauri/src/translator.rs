use std::collections::HashMap;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use crate::models::*;
use crate::AppResult;

pub struct TranslateEngine {
    fallback_mode: bool,
    dict: Mutex<HashMap<String, String>>,
}

impl TranslateEngine {
    pub fn new() -> Self {
        let mut dict = HashMap::new();

        dict.insert("the".to_string(), "这个".to_string());
        dict.insert("quick".to_string(), "快速的".to_string());
        dict.insert("brown".to_string(), "棕色的".to_string());
        dict.insert("fox".to_string(), "狐狸".to_string());
        dict.insert("jumps".to_string(), "跳跃".to_string());
        dict.insert("over".to_string(), "在...上面".to_string());
        dict.insert("lazy".to_string(), "懒惰的".to_string());
        dict.insert("dog".to_string(), "狗".to_string());
        dict.insert("this".to_string(), "这个".to_string());
        dict.insert("is".to_string(), "是".to_string());
        dict.insert("a".to_string(), "一个".to_string());
        dict.insert("sample".to_string(), "示例".to_string());
        dict.insert("ocr".to_string(), "OCR".to_string());
        dict.insert("recognition".to_string(), "识别".to_string());
        dict.insert("result".to_string(), "结果".to_string());
        dict.insert("hello".to_string(), "你好".to_string());
        dict.insert("world".to_string(), "世界".to_string());
        dict.insert("welcome".to_string(), "欢迎".to_string());
        dict.insert("to".to_string(), "到".to_string());
        dict.insert("translate".to_string(), "翻译".to_string());
        dict.insert("desktop".to_string(), "桌面版".to_string());
        dict.insert("artificial".to_string(), "人工".to_string());
        dict.insert("intelligence".to_string(), "智能".to_string());
        dict.insert("makes".to_string(), "让".to_string());
        dict.insert("life".to_string(), "生活".to_string());
        dict.insert("easier".to_string(), "更美好".to_string());
        dict.insert("tesseract".to_string(), "Tesseract".to_string());
        dict.insert("engine".to_string(), "引擎".to_string());
        dict.insert("fallback".to_string(), "回退".to_string());
        dict.insert("mode".to_string(), "模式".to_string());
        dict.insert("computer".to_string(), "电脑".to_string());
        dict.insert("vision".to_string(), "视觉".to_string());
        dict.insert("machine".to_string(), "机器".to_string());
        dict.insert("learning".to_string(), "学习".to_string());
        dict.insert("deep".to_string(), "深度".to_string());
        dict.insert("neural".to_string(), "神经".to_string());
        dict.insert("network".to_string(), "网络".to_string());
        dict.insert("image".to_string(), "图片".to_string());
        dict.insert("processing".to_string(), "处理".to_string());
        dict.insert("text".to_string(), "文字".to_string());
        dict.insert("extraction".to_string(), "提取".to_string());
        dict.insert("document".to_string(), "文档".to_string());
        dict.insert("analysis".to_string(), "分析".to_string());
        dict.insert("natural".to_string(), "自然".to_string());
        dict.insert("language".to_string(), "语言".to_string());

        Self {
            fallback_mode: true,
            dict: Mutex::new(dict),
        }
    }

    pub fn is_fallback(&self) -> bool {
        self.fallback_mode
    }

    pub fn translate(
        &self,
        source_text: &str,
        options: Option<&TranslateOptions>,
    ) -> AppResult<(String, String, String)> {
        let source_lang = options
            .and_then(|o| o.source_lang.clone())
            .unwrap_or_else(|| detect_lang(source_text));
        let target_lang = options
            .and_then(|o| o.target_lang.clone())
            .unwrap_or_else(|| "zh-CN".to_string());

        let translated = if source_lang.starts_with("zh") && !target_lang.starts_with("zh") {
            self.translate_zh_to_en(source_text)
        } else if target_lang.starts_with("zh") {
            self.translate_to_zh(source_text, &source_lang)
        } else {
            self.translate_passthrough(source_text)
        };

        Ok((translated, source_lang, target_lang))
    }

    fn translate_to_zh(&self, text: &str, _source_lang: &str) -> String {
        let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?' || c == '。' || c == '！' || c == '？').collect();
        let mut result_parts = Vec::new();

        for sent in sentences {
            let sent = sent.trim();
            if sent.is_empty() {
                continue;
            }
            let translated = self.translate_sentence_to_zh(sent);
            result_parts.push(translated);
        }

        if result_parts.is_empty() {
            return format!("[模拟翻译] {}", text);
        }

        result_parts.join("。") + "。"
    }

    fn translate_sentence_to_zh(&self, sentence: &str) -> String {
        let special = self.lookup_special(sentence);
        if let Some(s) = special {
            return s;
        }

        let dict = self.dict.lock();
        let words: Vec<&str> = sentence.split_whitespace().collect();
        let mut translated_words = Vec::with_capacity(words.len());

        for w in &words {
            let clean = w
                .trim_matches(|c: char| c.is_ascii_punctuation())
                .to_lowercase();
            if let Some(t) = dict.get(&clean) {
                translated_words.push(t.clone());
            } else if !clean.is_empty() {
                translated_words.push(w.to_string());
            }
        }

        drop(dict);

        if translated_words.is_empty() {
            return sentence.to_string();
        }

        translated_words.join("")
    }

    fn translate_zh_to_en(&self, text: &str) -> String {
        let dict = self.dict.lock();
        let reverse: HashMap<&str, &str> = dict
            .iter()
            .map(|(k, v)| (v.as_str(), k.as_str()))
            .collect();
        drop(dict);

        let mut result = String::with_capacity(text.len());
        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let mut matched = false;
            for len in (2..=6).rev() {
                if i + len <= chars.len() {
                    let seg: String = chars[i..i + len].iter().collect();
                    if let Some(t) = reverse.get(seg.as_str()) {
                        if !result.is_empty() {
                            result.push(' ');
                        }
                        result.push_str(t);
                        i += len;
                        matched = true;
                        break;
                    }
                }
            }
            if !matched {
                if chars[i].is_ascii_alphanumeric() {
                    if !result.is_empty() && !result.ends_with(' ') {
                        result.push(' ');
                    }
                    result.push(chars[i]);
                } else {
                    result.push(chars[i]);
                }
                i += 1;
            }
        }

        if result.trim().is_empty() {
            return format!("[Mock Translation] {}", text);
        }

        result.trim().to_string()
    }

    fn translate_passthrough(&self, text: &str) -> String {
        format!("[模拟翻译] {}", text)
    }

    fn lookup_special(&self, text: &str) -> Option<String> {
        let lower = text.to_lowercase();
        let special: &[(&str, &str)] = &[
            (
                "the quick brown fox jumps over the lazy dog",
                "敏捷的棕色狐狸跳过了懒狗",
            ),
            (
                "this is a sample ocr recognition result",
                "这是一个示例OCR识别结果",
            ),
            ("hello world", "你好世界"),
            (
                "hello world welcome to ocr translate desktop",
                "你好世界，欢迎使用OCR翻译桌面版",
            ),
            (
                "artificial intelligence makes life easier",
                "人工智能让生活更美好",
            ),
            (
                "tesseract ocr engine fallback mode",
                "Tesseract OCR引擎回退模拟模式",
            ),
            ("computer vision", "计算机视觉"),
            ("machine learning", "机器学习"),
            ("deep neural network", "深度神经网络"),
            ("image processing", "图像处理"),
            ("text extraction", "文字提取"),
            ("document analysis", "文档分析"),
            ("natural language processing", "自然语言处理"),
        ];
        for (k, v) in special {
            if lower.contains(k) {
                if lower.trim() == *k {
                    return Some(v.to_string());
                }
                let replaced = lower.replace(k, v);
                return Some(replaced);
            }
        }
        None
    }
}

impl Default for TranslateEngine {
    fn default() -> Self {
        Self::new()
    }
}

fn detect_lang(text: &str) -> String {
    let chinese_count = text.chars().filter(|c| {
        matches!(c as u32,
            0x4E00..=0x9FFF |
            0x3400..=0x4DBF |
            0x20000..=0x2A6DF
        )
    }).count();
    let total_chars = text.chars().filter(|c| !c.is_whitespace() && !c.is_ascii_punctuation()).count();
    if total_chars == 0 {
        return "unknown".to_string();
    }
    let ratio = chinese_count as f64 / total_chars as f64;
    if ratio > 0.3 {
        "zh-CN".to_string()
    } else {
        "en".to_string()
    }
}

pub static TRANSLATOR: Lazy<TranslateEngine> = Lazy::new(TranslateEngine::new);
