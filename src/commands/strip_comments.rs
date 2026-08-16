use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crossterm::style::Color;

use crate::config::Config;
use crate::error::AppResult;
use crate::ui::OutputWriter;

/// Удаляет строки комментариев (начинающиеся с заданных символов) из файла.
///
/// Строки, содержащие любой из фрагментов из `keep_string`, сохраняются.
/// Если `keep_string` не задан или пуст — удаляются все строки комментариев.
/// Использует `\r\n` для оконной совместимости.
///
/// ### `.extra`
/// - `keep_string` — список фрагментов через запятую. Строки-комментарии,
///   содержащие хотя бы один фрагмент, не удаляются.
///   Пример: `keep_string = "MSG,INFO"` сохраняет `;MSG("...")` и `;INFO("...")`.
/// - `comment_chars` — строка символов, которыми начинаются комментарии
///   (умолч. `";"`). Пример: `comment_chars = ";,/"`.
/// - `strip_mode` — режим поиска символа комментария:
///   - `"starts-with"` (по умолч.) — строка считается комментарием, если
///     начинается с одного из символов.
///   - `"contains"` — строка считается комментарием, если символ встречается
///     в любом месте строки.
pub fn execute(
    file: &str,
    _config: &Config,
    extra: &HashMap<String, String>,
    out: &mut dyn OutputWriter,
) -> AppResult<()> {
    let path = Path::new(file);

    if !path.exists() {
        out.print_colored(
            Color::Red,
            &format!("Ошибка: файл '{}' не найден", path.display()),
        )?;
        out.print("\n")?;
        return Ok(());
    }

    let keep_list: Vec<&str> = extra
        .get("keep_string")
        .map(|s| {
            s.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let comment_chars: Vec<char> = extra
        .get("comment_chars")
        .map(|s| s.chars().collect())
        .unwrap_or_else(|| vec![';']);

    let strip_mode: &str = extra
        .get("strip_mode")
        .map(|s| s.as_str())
        .unwrap_or("starts-with");

    let bytes = fs::read(file)?;
    let content = String::from_utf8_lossy(&bytes).to_string();
    let lines: Vec<&str> = content.lines().collect();
    let original_count = lines.len();

    let is_comment = |line: &str| -> bool {
        match strip_mode {
            "contains" => comment_chars.iter().any(|c| line.contains(*c)),
            _ => {
                let trimmed = line.trim_start();
                comment_chars.iter().any(|c| trimmed.starts_with(*c))
            }
        }
    };

    let filtered: Vec<&str> = lines
        .into_iter()
        .filter(|line| {
            if !is_comment(line) {
                return true;
            }
            if keep_list.iter().any(|k| line.contains(k)) {
                return true;
            }
            false
        })
        .collect();

    let removed = original_count - filtered.len();

    if removed == 0 {
        out.status_info(" [ нет строк для удаления ]")?;
        return Ok(());
    }

    let output = filtered.join("\r\n");
    fs::write(file, output)?;

    out.print_colored(Color::DarkGrey, &format!("-> удалено {removed} строк "))?;
    out.status_ok()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::ui::TestWriter;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn strips_comment_lines() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(
            &file_path,
            "MSG(\"Test\")\r\n; this is a comment\r\nN10 G0 X0\r\n; another comment\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("keep_string".into(), String::new());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("MSG(\"Test\")"));
        assert!(content.contains("N10 G0 X0"));
        assert!(!content.contains("; this is a comment"));
        assert!(!content.contains("; another comment"));
    }

    #[test]
    fn keeps_lines_containing_keep_string() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(
            &file_path,
            ";MSG(\"Test\")\r\n; strip this\r\nN10 G0 X0\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("keep_string".into(), "MSG".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("MSG"));
        assert!(content.contains("N10 G0 X0"));
        assert!(!content.contains("; strip this"));
    }

    #[test]
    fn keeps_lines_with_multiple_fragments() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(
            &file_path,
            ";MSG(\"Test\")\r\n;INFO(\"note\")\r\n; strip this\r\nN10 G0 X0\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("keep_string".into(), "MSG,INFO".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("MSG"));
        assert!(content.contains("INFO"));
        assert!(content.contains("N10 G0 X0"));
        assert!(!content.contains("; strip this"));
    }

    #[test]
    fn no_comments_to_strip() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(&file_path, "N10 G0 X0\r\nN20 G1 Z-1\r\n").unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let extra = HashMap::new();

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let last = writer.output.last().unwrap();
        assert_eq!(last.0, Some(Color::Yellow));
        assert!(last.1.contains("нет строк"));
    }

    #[test]
    fn strips_with_custom_comment_char() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(
            &file_path,
            "MSG(\"Test\")\r\n( comment)\r\nN10 G0 X0\r\n( another)\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("comment_chars".into(), "(".into());
        extra.insert("keep_string".into(), String::new());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("MSG(\"Test\")"));
        assert!(content.contains("N10 G0 X0"));
        assert!(!content.contains("( comment)"));
        assert!(!content.contains("( another)"));
    }

    #[test]
    fn strips_with_multiple_comment_chars() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(
            &file_path,
            "; semicolon comment\r\n% percent comment\r\n/ slash comment\r\nN10 G0 X0\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("comment_chars".into(), ";%/".into());
        extra.insert("keep_string".into(), String::new());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "N10 G0 X0");
    }

    #[test]
    fn custom_comment_char_with_keep_string() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(
            &file_path,
            "(%KEEP=\"value\")\r\n(% strip this)\r\nN10 G0 X0\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("comment_chars".into(), "(".into());
        extra.insert("keep_string".into(), "KEEP".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("KEEP"));
        assert!(content.contains("N10 G0 X0"));
        assert!(!content.contains("strip this"));
    }

    #[test]
    fn strips_with_contains_mode() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(
            &file_path,
            "N10 G0 X0 ; inline comment\r\n; full line comment\r\nN20 G1 Z-1\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("comment_chars".into(), ";".into());
        extra.insert("strip_mode".into(), "contains".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "N20 G1 Z-1");
    }

    #[test]
    fn keeps_keep_string_in_contains_mode() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(&file_path, "N10 G0 X0 ;MSG(\"test\")\r\nN20 G1 Z-1\r\n").unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("comment_chars".into(), ";".into());
        extra.insert("strip_mode".into(), "contains".into());
        extra.insert("keep_string".into(), "MSG".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("MSG"));
        assert!(content.contains("N20 G1 Z-1"));
    }
}
