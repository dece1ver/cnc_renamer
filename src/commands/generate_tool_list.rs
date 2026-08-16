use std::collections::HashMap;
use std::fs;
use std::path::Path;

use cnc_remedy::tool_list::{CncVariant, detect_variant, format_tool_table, parse_tools};
use crossterm::style::Color;

use crate::config::Config;
use crate::error::AppResult;
use crate::ui::OutputWriter;

/// Генерирует таблицу инструментов и вставляет её в файл УП.
///
/// Анализирует файл на предмет использованных инструментов (Fanuc,
/// Sinumerik, Heidenhain) и вставляет форматированную таблицу
/// со списком инструментов, их H/D кодами и комментариями.
///
/// ### `.extra`
/// - `fanuc_milling_line`, `fanuc_lathe_line`, `sinumerik_line`,
///   `heidenhain_line` — строка вставки таблицы (умолч. 6 для Fanuc/Sinumerik,
///   3 для Heidenhain).
/// - `fanuc_milling_print_tool_number`, `fanuc_lathe_print_tool_number`,
///   `sinumerik_print_tool_number`, `heidenhain_print_tool_number` — если
///   `"true"` (умолч.), выводится номер инструмента, H и D коды; если
///   `"false"` — только комментарий.
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
        return Ok(());
    }

    let content = cnc_remedy::text::read_program(path)?;
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    if lines.is_empty() {
        out.status_info(" [ пустой файл ]")?;
        return Ok(());
    }

    let variant = match detect_variant(file, &lines) {
        Some(v) => v,
        None => {
            out.status_info(" [ Mazatrol не поддерживается ]")?;
            return Ok(());
        }
    };

    let insert_line = get_insert_line(&variant, extra);
    let tools = parse_tools(&lines, &variant);

    if tools.is_empty() {
        out.status_info(" [ инструменты не найдены ]")?;
        return Ok(());
    }

    let print_tool_number = get_print_tool_number(&variant, extra);
    let table = format_tool_table(&tools, &variant, print_tool_number);

    let insert_at = insert_line.min(lines.len());
    for (i, line) in table.iter().enumerate() {
        lines.insert(insert_at + i, line.clone());
    }

    fs::write(file, lines.join("\r\n"))?;

    out.print_colored(
        Color::DarkGrey,
        &format!("-> добавлено {} инструментов ", tools.len()),
    )?;
    out.status_ok()?;

    Ok(())
}

fn get_insert_line(variant: &CncVariant, extra: &HashMap<String, String>) -> usize {
    let key = match variant {
        CncVariant::FanucMilling => "fanuc_milling_line",
        CncVariant::FanucLathe => "fanuc_lathe_line",
        CncVariant::Sinumerik => "sinumerik_line",
        CncVariant::Heidenhain => "heidenhain_line",
    };
    extra
        .get(key)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(50)
}

fn get_print_tool_number(variant: &CncVariant, extra: &HashMap<String, String>) -> bool {
    let key = match variant {
        CncVariant::FanucMilling => "fanuc_milling_print_tool_number",
        CncVariant::FanucLathe => "fanuc_lathe_print_tool_number",
        CncVariant::Sinumerik => "sinumerik_print_tool_number",
        CncVariant::Heidenhain => "heidenhain_print_tool_number",
    };
    extra.get(key).map(|s| s == "true").unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::ui::TestWriter;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // ── Integration: execute ───────────────────────────────

    #[test]
    fn execute_fanuc_milling_inserts_table() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.nc");
        fs::write(
            &file_path,
            "%\r\nT1 M6 (10MM ENDMILL)\r\nG43 H1\r\nG41 D1\r\nM30\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("fanuc_milling_line".into(), "5".into());
        extra.insert("fanuc_milling_print_tool_number".into(), "true".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("(T01 H01 D01 - 10MM ENDMILL)"));
    }

    #[test]
    fn execute_fanuc_lathe_inserts_table() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.nc");
        fs::write(&file_path, "%\r\nT0101 (ROUGH TURN)\r\nM30\r\n").unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("fanuc_lathe_line".into(), "3".into());
        extra.insert("fanuc_lathe_print_tool_number".into(), "true".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("(T0101 - ROUGH TURN)"));
    }

    #[test]
    fn execute_sinumerik_inserts_table() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(&file_path, "MSG(\"TEST\")\r\nT25 M6;FR 6\r\nM30\r\n").unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("sinumerik_line".into(), "3".into());
        extra.insert("sinumerik_print_tool_number".into(), "true".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains(";T25 - FR 6"));
    }

    #[test]
    fn execute_heidenhain_inserts_table() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.h");
        fs::write(
            &file_path,
            "BEGIN PGM TEST MM\r\n; 10MM ENDMILL\r\nTOOL CALL 1 Z S1000\r\nM30\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("heidenhain_line".into(), "4".into());
        extra.insert("heidenhain_print_tool_number".into(), "true".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains(";T01 - 10MM ENDMILL"));
    }

    #[test]
    fn execute_no_tools_found() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.nc");
        fs::write(&file_path, "%\r\nG00 X0 Y0\r\nM30\r\n").unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let extra = HashMap::new();

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let last = writer.output.last().unwrap();
        assert_eq!(last.0, Some(Color::Yellow));
        assert!(last.1.contains("инструменты не найдены"));
    }

    #[test]
    fn execute_file_not_found() {
        let mut writer = TestWriter::new();
        let config = Config::default();
        let extra = HashMap::new();

        execute("/nonexistent/file.nc", &config, &extra, &mut writer).unwrap();
    }

    #[test]
    fn execute_mazatrol_not_supported() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.pbg");
        fs::write(&file_path, [0u8; 100]).unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let extra = HashMap::new();

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let last = writer.output.last().unwrap();
        assert_eq!(last.0, Some(Color::Yellow));
        assert!(last.1.contains("Mazatrol"));
    }
}
