use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Расширения файлов, определяющие формат Mazatrol.
const MAZATROL_EXTENSIONS: [&str; 2] = ["pbg", "pbd"];

/// Расширения файлов, определяющие формат Heidenhain.
const HEIDENHAIN_EXTENSIONS: [&str; 1] = ["h"];

/// Расширения файлов, определяющие формат Sinumerik.
const SINUMERIK_EXTENSIONS: [&str; 2] = ["mpf", "spf"];

/// Символы, запрещённые в именах файлов Windows (заменяются на `-`).
const BAD_SYMBOLS: [char; 9] = ['<', '>', ':', '\"', '/', '\\', '|', '?', '*'];

/// Смещение в байтах, где начинается имя программы Mazatrol.
const MAZATROL_NAME_OFFSET: usize = 80;

/// Максимальная длина имени программы Mazatrol в байтах.
const MAZATROL_NAME_LENGTH: usize = 32;

/// Определяет систему ЧПУ по расширению файла и извлекает имя программы.
///
/// Возвращает `(имя, расширение)`, где расширение — оригинальное
/// расширение файла или пустая строка для Fanuc.
pub fn get_cnc_name(file_path: &str) -> Option<(String, &str)> {
    match get_extension(file_path) {
        None => get_fanuc_name(file_path),
        Some(ext) => {
            if MAZATROL_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                get_mazatrol_name(file_path, ext)
            } else if HEIDENHAIN_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                get_heidenhain_name(file_path, ext)
            } else if SINUMERIK_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                get_sinumerik_name(file_path, ext)
            } else {
                get_fanuc_name(file_path)
            }
        }
    }
}

/// Парсит имя программы Fanuc из первых двух строк.
///
/// Поддерживаемые форматы:
/// * `O0001(ИМЯ)` — O-номер с именем в скобках
/// * `<ИМЯ>` — имя в угловых скобках на второй строке после `%`
fn get_fanuc_name(file_path: &str) -> Option<(String, &str)> {
    if let Ok(lines) = read_lines(file_path) {
        for (i, line) in lines.iter().take(2).enumerate() {
            if i == 0 && line.starts_with('%') {
                continue;
            } else if i == 1 && line.starts_with('<') {
                return line.split('<').nth(1).and_then(|name| {
                    name.split('>')
                        .next()
                        .map(|name| (remove_bad_symbols(name), ""))
                });
            } else if i == 1 && line.starts_with('O') {
                return line.split('(').nth(1).and_then(|name| {
                    name.split(')')
                        .next()
                        .map(|name| (remove_bad_symbols(name), ""))
                });
            } else {
                return None;
            }
        }
    }
    None
}

/// Извлекает имя программы из файла Mazatrol.
///
/// Имя начинается со смещения 80 и имеет длину до 32 байт.
fn get_mazatrol_name<'a>(file_path: &str, extension: &'a str) -> Option<(String, &'a str)> {
    if let Ok(mut f) = File::open(file_path) {
        let mut buffer = Vec::new();
        if f.read_to_end(&mut buffer).is_ok() {
            let name: String = String::from_utf8_lossy(&buffer)
                .chars()
                .skip(MAZATROL_NAME_OFFSET)
                .take(MAZATROL_NAME_LENGTH)
                .collect();
            return Some((
                remove_bad_symbols(name.trim().trim_matches('\0')),
                extension,
            ));
        }
    }
    None
}

/// Извлекает имя программы из файла Sinumerik.
///
/// Ищет паттерн `MSG("имя")` на первой строке.
fn get_sinumerik_name<'a>(file_path: &str, extension: &'a str) -> Option<(String, &'a str)> {
    if let Ok(lines) = read_lines(file_path)
        && let Some(line) = lines.iter().next()
        && line.starts_with("MSG")
        && line.contains('(')
        && line.contains(')')
        && let Some(name) = line.split('(').nth(1)
        && let Some(name) = name.split(')').next()
    {
        return Some((remove_bad_symbols(name.trim_matches('"')), extension));
    }
    None
}

/// Извлекает имя программы из файла Heidenhain.
///
/// Ищет `BEGIN PGM ИМЯ` на первой строке.
fn get_heidenhain_name<'a>(file_path: &str, extension: &'a str) -> Option<(String, &'a str)> {
    if let Ok(lines) = read_lines(file_path)
        && let Some(line) = lines.iter().next()
        && line.starts_with("BEGIN PGM")
    {
        return Some((
            remove_bad_symbols(
                line.replace("BEGIN PGM ", "")
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('0')
                    .trim(),
            ),
            extension,
        ));
    }
    None
}

/// Возвращает расширение файла (без точки) из строки пути.
fn get_extension(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

/// Открывает файл и возвращает строки с lossy UTF-8 декодингом.
pub fn read_lines<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let bytes = std::fs::read(filename)?;
    let content = String::from_utf8_lossy(&bytes);
    Ok(content.lines().map(|s| s.to_string()).collect())
}

/// Заменяет символы, запрещённые в именах файлов Windows, на `-`.
pub fn remove_bad_symbols(text: &str) -> String {
    let mut text = text.to_string();
    for bad_symbol in BAD_SYMBOLS {
        text = text.replace(bad_symbol, "-");
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp(content: &str) -> (NamedTempFile, String) {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "{content}").unwrap();
        let path = f.path().to_str().unwrap().to_string();
        (f, path)
    }

    // ── Fanuc ──────────────────────────────────────────────

    #[test]
    fn fanuc_percent_then_o_with_parens() {
        let (_f, path) = write_temp("%\nO0001(МОЯ ДЕТАЛЬ)");
        let (name, ext) = get_cnc_name(&path).unwrap();
        assert_eq!(name, "МОЯ ДЕТАЛЬ");
        assert_eq!(ext, "");
    }

    #[test]
    fn fanuc_percent_then_angle() {
        let (_f, path) = write_temp("%\n<MY-PART>");
        let (name, ext) = get_cnc_name(&path).unwrap();
        assert_eq!(name, "MY-PART");
        assert_eq!(ext, "");
    }

    #[test]
    fn fanuc_no_name_returns_none() {
        let (_f, path) = write_temp("N10 G0 X0\nN20 G1 Z-1");
        assert!(get_cnc_name(&path).is_none());
    }

    #[test]
    fn fanuc_bad_symbols_replaced() {
        let (_f, path) = write_temp("%\nO0001(file<name>)");
        let (name, _) = get_cnc_name(&path).unwrap();
        assert_eq!(name, "file-name-");
    }

    // ── Mazatrol ────────────────────────────────────────────

    #[test]
    fn mazatrol_name_from_offset() {
        let mut f = NamedTempFile::new().unwrap();
        let mut content = vec![b' '; 120];
        let prog_name = b"MY-PROGRAM";
        content[MAZATROL_NAME_OFFSET..MAZATROL_NAME_OFFSET + prog_name.len()]
            .copy_from_slice(prog_name);
        f.write_all(&content).unwrap();
        let path = f.path().to_str().unwrap().to_string();

        let pbg_path = format!("{path}.pbg");
        std::fs::copy(&path, &pbg_path).unwrap();
        let (name, ext) = get_cnc_name(&pbg_path).unwrap();
        assert_eq!(name, "MY-PROGRAM");
        assert_eq!(ext, "pbg");
        let _ = std::fs::remove_file(&pbg_path);
    }

    // ── Sinumerik ──────────────────────────────────────────

    #[test]
    fn sinumerik_msg_format() {
        let (_f, path) = write_temp("MSG(\"TestPart\")\nN10 G0 X0");
        let mpf_path = format!("{path}.mpf");
        std::fs::copy(&path, &mpf_path).unwrap();
        let (name, ext) = get_cnc_name(&mpf_path).unwrap();
        assert_eq!(name, "TestPart");
        assert_eq!(ext, "mpf");
        let _ = std::fs::remove_file(&mpf_path);
    }

    #[test]
    fn sinumerik_no_msg_returns_none() {
        let (_f, path) = write_temp("N10 G0 X0\n");
        let mpf_path = format!("{path}.mpf");
        std::fs::copy(&path, &mpf_path).unwrap();
        assert!(get_cnc_name(&mpf_path).is_none());
        let _ = std::fs::remove_file(&mpf_path);
    }

    // ── Heidenhain ──────────────────────────────────────────

    #[test]
    fn heidenhain_begin_pgm() {
        let (_f, path) = write_temp("BEGIN PGM 0123 MM\n");
        let h_path = format!("{path}.h");
        std::fs::copy(&path, &h_path).unwrap();
        let (name, ext) = get_cnc_name(&h_path).unwrap();
        assert_eq!(name, "123");
        assert_eq!(ext, "h");
        let _ = std::fs::remove_file(&h_path);
    }

    #[test]
    fn heidenhain_no_begin_returns_none() {
        let (_f, path) = write_temp("N10 G0 X0\n");
        let h_path = format!("{path}.h");
        std::fs::copy(&path, &h_path).unwrap();
        assert!(get_cnc_name(&h_path).is_none());
        let _ = std::fs::remove_file(&h_path);
    }

    // ── remove_bad_symbols ──────────────────────────────────

    #[test]
    fn removes_bad_symbols() {
        assert_eq!(
            remove_bad_symbols("a<b>c:d\"e/f\\g|h?i*j"),
            "a-b-c-d-e-f-g-h-i-j"
        );
    }

    #[test]
    fn leaves_good_symbols_alone() {
        assert_eq!(remove_bad_symbols("hello world 123"), "hello world 123");
    }
}
