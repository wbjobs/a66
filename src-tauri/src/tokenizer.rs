use jieba_rs::Jieba;
use once_cell::sync::Lazy;
use std::sync::Mutex;

pub static JIEBA: Lazy<Mutex<Jieba>> = Lazy::new(|| {
    let jieba = Jieba::new();
    Mutex::new(jieba)
});

pub fn tokenize_chinese(text: &str) -> String {
    let jieba = JIEBA.lock().unwrap();

    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let total = chars.len();

    while i < total {
        let ch = chars[i];

        if is_cjk(ch) {
            let mut j = i;
            while j < total && is_cjk(chars[j]) {
                j += 1;
            }
            if j > i {
                let segment: String = chars[i..j].iter().collect();
                let tokens = jieba.cut(&segment, false);
                for token in tokens {
                    result.push_str(token);
                    result.push(' ');
                }
                i = j;
            }
        } else if ch.is_ascii_alphanumeric() {
            let mut j = i;
            while j < total
                && (chars[j].is_ascii_alphanumeric() || chars[j] == '_' || chars[j] == '\'')
            {
                j += 1;
            }
            let word: String = chars[i..j].iter().collect();
            result.push_str(&word);
            result.push(' ');
            i = j;
        } else if ch.is_whitespace() {
            i += 1;
        } else {
            result.push(ch);
            result.push(' ');
            i += 1;
        }
    }

    result.trim().to_string()
}

fn is_cjk(c: char) -> bool {
    let code = c as u32;
    matches!(code,
        0x4E00..=0x9FFF |
        0x3400..=0x4DBF |
        0x20000..=0x2A6DF |
        0x2A700..=0x2B73F |
        0x2B740..=0x2B81F |
        0x2B820..=0x2CEAF |
        0xF900..=0xFAFF |
        0x2F800..=0x2FA1F |
        0x3000..=0x303F |
        0xFF00..=0xFFEF
    )
}

pub fn tokenize_for_search(keyword: &str) -> String {
    let tokenized = tokenize_chinese(keyword);
    if tokenized.is_empty() {
        return keyword.to_string();
    }
    let parts: Vec<&str> = tokenized.split_whitespace().collect();
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

pub fn contains_chinese(text: &str) -> bool {
    text.chars().any(|c| {
        let code = c as u32;
        (0x4E00..=0x9FFF).contains(&code)
            || (0x3400..=0x4DBF).contains(&code)
            || (0x20000..=0x2A6DF).contains(&code)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_chinese() {
        let result = tokenize_chinese("你好世界");
        println!("{}", result);
        assert!(result.contains("你好"));
        assert!(result.contains("世界"));
    }

    #[test]
    fn test_tokenize_mixed() {
        let result = tokenize_chinese("Hello你好world世界123");
        println!("{}", result);
        assert!(result.contains("Hello"));
        assert!(result.contains("你好"));
        assert!(result.contains("world"));
        assert!(result.contains("世界"));
        assert!(result.contains("123"));
    }

    #[test]
    fn test_search_query() {
        let result = tokenize_for_search("你好");
        println!("{}", result);
        assert_eq!(result, "你好*");

        let result2 = tokenize_for_search("你好世界");
        println!("{}", result2);
        assert!(result2.contains("AND"));
    }

    #[test]
    fn test_contains_chinese() {
        assert!(contains_chinese("你好"));
        assert!(!contains_chinese("Hello"));
        assert!(contains_chinese("Hello你好"));
    }
}
