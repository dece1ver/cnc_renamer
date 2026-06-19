use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use crossterm::style::Color;

use crate::config::Config;
use crate::error::AppResult;
use crate::reader;
use crate::ui::OutputWriter;

/// Переименовывает файл УП по названию, найденному внутри файла.
///
/// Читает имя программы из содержимого файла (Fanuc, Mazatrol, Sinumerik
/// или Heidenhain) и переименовывает файл. Если целевое имя занято,
/// добавляет числовой суффикс.
///
/// ### `.extra`
/// - `prefix` — добавляется в начало имени (перед названием УП).
/// - `suffix` — добавляется в конец имени (перед расширением).
/// - `overwrite = "true"` — при конфликте имён сравнивает дату изменения
///   и оставляет новейший файл (вместо создания копий с номером).
pub fn execute(
    file: &str,
    _config: &Config,
    extra: &HashMap<String, String>,
    out: &mut dyn OutputWriter,
) -> AppResult<()> {
    if let Some((mut name, ext)) = reader::get_cnc_name(file) {
        let prefix = extra.get("prefix").map(|s| s.as_str()).unwrap_or("");
        let suffix = extra.get("suffix").map(|s| s.as_str()).unwrap_or("");
        let overwrite = extra.get("overwrite").map(|s| s == "true").unwrap_or(false);

        if !prefix.is_empty() || !suffix.is_empty() {
            name = format!("{prefix}{name}{suffix}");
        }

        let old_path = Path::new(file);

        if let Some(dir) = old_path.parent() {
            let (new_path, new_name, already_matches) = if overwrite {
                let new_name = if ext.is_empty() {
                    name.clone()
                } else {
                    format!("{name}.{ext}")
                };
                let new_path = dir.join(&new_name);
                (new_path.clone(), new_name, new_path == old_path)
            } else {
                dedup_path(old_path, &name, ext, dir)?
            };

            if already_matches {
                out.status_info(" [ не требуется ]")?;
                return Ok(());
            }

            if overwrite && new_path.exists() && new_path != old_path {
                let src_mtime = old_path.metadata()?.modified()?;
                let dst_mtime = new_path.metadata()?.modified()?;
                if dst_mtime > src_mtime {
                    out.status_info(" [ новее существует ]")?;
                    return Ok(());
                }
                fs::remove_file(&new_path)?;
            }

            out.print_colored(Color::DarkGrey, "-> ")?;
            out.print_colored(Color::Cyan, &format!("{new_name} "))?;

            if fs::rename(old_path, &new_path).is_ok() {
                out.status_ok()?;
            } else {
                out.status_bad()?;
            }
        }
    } else {
        out.status_info(" [ не программа или отсутствует имя ]")?;
    }
    Ok(())
}

/// Вычисляет путь без конфликтов имён для переименованного файла.
///
/// Возвращает `(конечный_путь, конечное_имя, уже_совпадает)`.
/// Если `already_matches == true` — исходный и целевой пути совпадают.
fn dedup_path(
    old_path: &Path,
    name: &str,
    ext: &str,
    dir: &Path,
) -> io::Result<(std::path::PathBuf, String, bool)> {
    let mut new_name = if ext.is_empty() {
        name.to_string()
    } else {
        format!("{name}.{ext}")
    };
    let mut new_path = dir.join(&new_name);
    let mut copy: u32 = 0;

    while new_path.exists() {
        if new_path == old_path {
            return Ok((new_path, new_name, true));
        }
        copy += 1;
        new_name = if ext.is_empty() {
            format!("{name} ({copy})")
        } else {
            format!("{name} ({copy}).{ext}")
        };
        new_path = dir.join(&new_name);
    }
    Ok((new_path, new_name, false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::ui::TestWriter;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn dedup_no_conflict() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("O0001.existing");
        fs::write(&file_path, b"test").unwrap();

        let (_path, name, matched) = dedup_path(&file_path, "O0001", "mpf", dir.path()).unwrap();
        assert!(!matched);
        assert_eq!(name, "O0001.mpf");
    }

    #[test]
    fn dedup_same_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("O0001.mpf");
        fs::write(&file_path, b"test").unwrap();

        let (_path, name, matched) = dedup_path(&file_path, "O0001", "mpf", dir.path()).unwrap();
        assert!(matched);
        assert_eq!(name, "O0001.mpf");
    }

    #[test]
    fn dedup_with_collision() {
        let dir = TempDir::new().unwrap();
        let original = dir.path().join("other.mpf");
        fs::write(&original, b"test").unwrap();
        let clash = dir.path().join("O0001.mpf");
        fs::write(&clash, b"test").unwrap();

        let (_path, name, matched) = dedup_path(&original, "O0001", "mpf", dir.path()).unwrap();
        assert!(!matched);
        assert_eq!(name, "O0001 (1).mpf");
    }

    #[test]
    fn execute_unknown_file_returns_info() {
        let mut writer = TestWriter::new();
        let config = Config::default();
        let extra = HashMap::new();

        execute("/nonexistent/file.nc", &config, &extra, &mut writer).unwrap();

        let last = writer.output.last().unwrap();
        assert_eq!(last.0, Some(Color::Yellow));
        assert!(last.1.contains("не программа"));
    }

    fn create_fanuc_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(name);
        // Fanuc format: first line %, second line O0001(NAME)
        fs::write(&file_path, format!("%\nO0001({content})")).unwrap();
        file_path
    }

    #[test]
    fn prefix_applied_to_name() {
        let dir = TempDir::new().unwrap();
        let file_path = create_fanuc_file(&dir, "source.nc", "MYPROG");

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("prefix".into(), "NC_".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let expected = dir.path().join("NC_MYPROG");
        assert!(expected.exists());
        assert!(!file_path.exists());
    }

    #[test]
    fn suffix_applied_to_name() {
        let dir = TempDir::new().unwrap();
        let file_path = create_fanuc_file(&dir, "source.nc", "MYPROG");

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("suffix".into(), "_v2".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let expected = dir.path().join("MYPROG_v2");
        assert!(expected.exists());
        assert!(!file_path.exists());
    }

    #[test]
    fn prefix_and_suffix_both_applied() {
        let dir = TempDir::new().unwrap();
        let file_path = create_fanuc_file(&dir, "source.nc", "MYPROG");

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("prefix".into(), "NC_".into());
        extra.insert("suffix".into(), "_v2".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let expected = dir.path().join("NC_MYPROG_v2");
        assert!(expected.exists());
        assert!(!file_path.exists());
    }

    #[test]
    fn overwrite_replaces_when_source_newer() {
        let dir = TempDir::new().unwrap();
        // Create destination first (older)
        let dest_path = dir.path().join("MYPROG");
        fs::write(&dest_path, b"old content").unwrap();
        let old_time = filetime::FileTime::from_unix_time(1000000, 0);
        filetime::set_file_mtime(&dest_path, old_time).unwrap();

        // Create source second (newer)
        let file_path = create_fanuc_file(&dir, "source.nc", "MYPROG");

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("overwrite".into(), "true".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        assert!(dest_path.exists());
        assert!(!file_path.exists());
        let content = fs::read_to_string(&dest_path).unwrap();
        // Content should be the source content (MYPROG was inside source.nc,
        // but Fanuc parse reads the name in parens -> "MYPROG", file content is "%\nO0001(MYPROG)")
        assert_eq!(content, "%\nO0001(MYPROG)");
    }

    #[test]
    fn overwrite_skips_when_dest_newer() {
        let dir = TempDir::new().unwrap();
        // Create destination second (newer)
        let dest_path = dir.path().join("MYPROG");

        // Create source first (older)
        let file_path = create_fanuc_file(&dir, "source.nc", "MYPROG");
        let old_time = filetime::FileTime::from_unix_time(1000000, 0);
        filetime::set_file_mtime(&file_path, old_time).unwrap();

        fs::write(&dest_path, b"newer content").unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("overwrite".into(), "true".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        assert!(file_path.exists());
        assert_eq!(fs::read_to_string(&dest_path).unwrap(), "newer content");
    }

    #[test]
    fn overwrite_no_conflict_renames_normally() {
        let dir = TempDir::new().unwrap();
        let file_path = create_fanuc_file(&dir, "source.nc", "MYPROG");

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("overwrite".into(), "true".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let expected = dir.path().join("MYPROG");
        assert!(expected.exists());
        assert!(!file_path.exists());
    }
}
