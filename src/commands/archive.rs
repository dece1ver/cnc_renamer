use std::collections::HashMap;
use std::fs;
use std::path::Path;

use chrono::{DateTime, Local};
use crossterm::style::Color;

use crate::config::Config;
use crate::error::AppResult;
use crate::ui::OutputWriter;

/// Имя директории архива, создаваемой рядом с исходным файлом.
const ARCHIVE_DIR_NAME: &str = "_";

/// Архивирует (перемещает) файл УП в поддиректорию с меткой времени.
///
/// Создаёт папку `_` рядом с файлом, внутри — подпапку с именем
/// `ДДММГГ.ЧЧММ` (текущее локальное время) и перемещает файл туда.
///
/// ### `.extra`
/// - `date_format` — формат даты подпапки (умолч. `"%d%m%y.%H%M"`).
/// - `timestamp_source` — `"now"` (текущее время, умолч.) или
///   `"modified"` (дата изменения файла).
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

    let parent_dir = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Невозможно получить родительскую директорию",
        )
    })?;

    let archive_base = parent_dir.join(ARCHIVE_DIR_NAME);
    fs::create_dir_all(&archive_base)?;

    let date_format = extra
        .get("date_format")
        .map(|s| s.as_str())
        .unwrap_or("%d%m%y.%H%M");
    let timestamp_source = extra
        .get("timestamp_source")
        .map(|s| s.as_str())
        .unwrap_or("now");

    let dt = if timestamp_source == "modified" {
        let metadata = path.metadata()?;
        let modified = metadata.modified()?;
        let duration = modified
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs() as i64;
        let nsecs = duration.subsec_nanos();
        DateTime::from_timestamp(secs, nsecs)
            .map(|utc| utc.with_timezone(&Local))
            .unwrap_or(Local::now())
    } else {
        Local::now()
    };

    let timestamp = dt.format(date_format).to_string();
    let archive_dir = archive_base.join(timestamp);
    fs::create_dir_all(&archive_dir)?;

    let file_name = path.file_name().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Невозможно получить имя файла",
        )
    })?;

    let dest_path = archive_dir.join(file_name);

    if dest_path.exists() {
        out.print_colored(
            Color::Red,
            &format!(
                "Ошибка: файл '{}' уже существует в архиве",
                dest_path.display()
            ),
        )?;
        out.print("\n")?;
        return Ok(());
    }

    fs::rename(path, &dest_path)?;
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
    fn archive_nonexistent_file_shows_error() {
        let mut writer = TestWriter::new();
        let config = Config::default();
        let extra = HashMap::new();

        execute("/tmp/nonexistent.nc", &config, &extra, &mut writer).unwrap();

        let first = &writer.output[0];
        assert_eq!(first.0, Some(Color::Red));
        assert!(first.1.contains("не найден"));
    }

    #[test]
    fn archive_creates_timestamped_dir() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.nc");
        fs::write(&file_path, b"content").unwrap();
        let file_str = file_path.to_str().unwrap().to_string();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let extra = HashMap::new();

        execute(&file_str, &config, &extra, &mut writer).unwrap();

        assert!(dir.path().join("_").exists());
        let entries: Vec<_> = fs::read_dir(dir.path().join("_"))
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(!entries.is_empty());
        assert!(!file_path.exists());
    }

    #[test]
    fn archive_with_custom_date_format() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.nc");
        fs::write(&file_path, b"content").unwrap();
        let file_str = file_path.to_str().unwrap().to_string();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("date_format".into(), "%Y-%m-%d".into());

        execute(&file_str, &config, &extra, &mut writer).unwrap();

        let archive_dir = dir.path().join("_");
        assert!(archive_dir.exists());
        let entries: Vec<_> = fs::read_dir(&archive_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1);
        let name = entries[0].file_name();
        let name_str = name.to_str().unwrap();
        // Should match YYYY-MM-DD format
        assert_eq!(name_str.len(), 10, "expected YYYY-MM-DD, got {name_str}");
        assert!(name_str.chars().nth(4) == Some('-'));
        assert!(name_str.chars().nth(7) == Some('-'));
    }

    #[test]
    fn archive_with_modified_timestamp() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.nc");
        fs::write(&file_path, b"content").unwrap();
        let file_str = file_path.to_str().unwrap().to_string();

        // Use filetime to set a known mtime
        let known_time = filetime::FileTime::from_unix_time(1700000000, 0);
        filetime::set_file_mtime(&file_path, known_time).unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("timestamp_source".into(), "modified".into());
        extra.insert("date_format".into(), "%s".into());

        execute(&file_str, &config, &extra, &mut writer).unwrap();

        let archive_dir = dir.path().join("_");
        assert!(archive_dir.exists());
        let entries: Vec<_> = fs::read_dir(&archive_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1);
        let name = entries[0].file_name();
        let name_str = name.to_str().unwrap();
        // Unix timestamp 1700000000
        assert_eq!(name_str, "1700000000");
    }
}
