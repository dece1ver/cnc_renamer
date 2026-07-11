use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crossterm::style::Color;

use crate::config::Config;
use crate::error::AppResult;
use crate::ui::OutputWriter;

const UNNAMED_TOOL: &str = "---";

#[derive(Clone, Debug, PartialEq)]
struct ToolInfo {
    number: u32,
    name: String,
    h: u32,
    d: u32,
}

enum CncVariant {
    FanucMilling,
    FanucLathe,
    Sinumerik,
    Heidenhain,
}

pub fn execute(
    file: &str,
    _config: &Config,
    extra: &HashMap<String, String>,
    out: &mut dyn OutputWriter,
) -> AppResult<()> {
    let path = Path::new(file);
    if !path.exists() {
        out.print_colored(Color::Red, &format!("Ошибка: файл '{}' не найден", path.display()))?;
        return Ok(());
    }

    let bytes = fs::read(file)?;
    let content = String::from_utf8_lossy(&bytes).to_string();
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
    let tools = match &variant {
        CncVariant::FanucMilling => parse_fanuc_milling(&lines),
        CncVariant::FanucLathe => parse_fanuc_lathe(&lines),
        CncVariant::Sinumerik => parse_sinumerik(&lines),
        CncVariant::Heidenhain => parse_heidenhain(&lines),
    };

    if tools.is_empty() {
        out.status_info(" [ инструменты не найдены ]")?;
        return Ok(());
    }

    let table = format_tool_table(&tools, &variant);

    let insert_at = insert_line.min(lines.len());
    for (i, line) in table.iter().enumerate() {
        lines.insert(insert_at + i, line.clone());
    }

    fs::write(file, lines.join("\r\n"))?;

    out.print_colored(Color::DarkGrey, &format!("-> добавлено {} инструментов ", tools.len()))?;
    out.status_ok()?;

    Ok(())
}

fn detect_variant(file: &str, lines: &[String]) -> Option<CncVariant> {
    let ext = get_file_extension(file);
    match ext.as_deref() {
        Some("h") => Some(CncVariant::Heidenhain),
        Some("mpf") | Some("spf") => Some(CncVariant::Sinumerik),
        Some("pbg") | Some("pbd") => None,
        _ => {
            if is_fanuc_milling(lines) {
                Some(CncVariant::FanucMilling)
            } else {
                Some(CncVariant::FanucLathe)
            }
        }
    }
}

fn get_file_extension(file: &str) -> Option<String> {
    Path::new(file)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
}

fn is_fanuc_milling(lines: &[String]) -> bool {
    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with('(') || trimmed.starts_with('<') {
            continue;
        }
        if let Some(t_pos) = trimmed.find('T') {
            let after_t = &trimmed[t_pos + 1..];
            let _digits: String = after_t.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !_digits.is_empty() {
                let rest = &after_t[_digits.len()..];
                if rest.contains("M06") || rest.contains("M6") {
                    return true;
                }
            }
        }
    }
    false
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

fn extract_tool_number(line: &str) -> Option<u32> {
    let trimmed = line.trim();
    if trimmed.starts_with('(') {
        return None;
    }
    let t_pos = trimmed.find('T')?;
    let after_t = &trimmed[t_pos + 1..];
    let digits: String = after_t.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

fn extract_comment(line: &str) -> Option<String> {
    let open = line.find('(')?;
    let after_open = &line[open + 1..];
    let close = after_open.find(')')?;
    Some(after_open[..close].trim().to_string())
}

fn extract_h_number(line: &str) -> Option<u32> {
    let h_pos = line.find('H')?;
    let after_h = &line[h_pos + 1..];
    let digits: String = after_h.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

fn extract_d_number(line: &str) -> Option<u32> {
    let d_pos = line.find('D')?;
    let after_d = &line[d_pos + 1..];
    let digits: String = after_d.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

fn parse_fanuc_milling(lines: &[String]) -> Vec<ToolInfo> {
    let mut tools: HashMap<u32, (String, u32, u32)> = HashMap::new();
    let mut current_tool = 0u32;
    let mut current_comment = String::new();

    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with('(') || trimmed.starts_with('<') {
            continue;
        }

        if trimmed.contains("M30") || trimmed.contains("M99") {
            if current_tool != 0 {
                    let entry = tools.entry(current_tool).or_insert_with(|| {
                        (current_comment.clone(), 0, 0)
                    });
                    if entry.0.is_empty() {
                        entry.0 = current_comment.clone();
                    }
                }
                break;
        }

        if line_has_m6(trimmed) {
            if let Some(tool_num) = extract_tool_number(trimmed) {
                if current_tool != 0 {
                    let entry = tools.entry(current_tool).or_insert_with(|| {
                        (current_comment.clone(), 0, 0)
                    });
                    if entry.0.is_empty() {
                        entry.0 = current_comment.clone();
                    }
                }
                current_tool = tool_num;
                let comment = extract_comment(trimmed).unwrap_or_default();
                if !comment.is_empty() {
                    current_comment = comment;
                } else {
                    current_comment.clear();
                }
                continue;
            }
        }

        if trimmed.contains("G43") && trimmed.contains('H') {
            if let Some(h) = extract_h_number(trimmed) {
                if current_tool != 0 {
                    tools.entry(current_tool)
                        .and_modify(|e| e.1 = h)
                        .or_insert_with(|| (current_comment.clone(), h, 0));
                }
            }
        }

        if (trimmed.contains("G41") || trimmed.contains("G42")) && trimmed.contains('D') {
            if let Some(d) = extract_d_number(trimmed) {
                if current_tool != 0 {
                    tools.entry(current_tool)
                        .and_modify(|e| e.2 = d)
                        .or_insert_with(|| (current_comment.clone(), 0, d));
                }
            }
        }
    }

    let mut result: Vec<ToolInfo> = tools
        .into_iter()
        .map(|(num, (name, h, d))| ToolInfo {
            number: num,
            name: if name.is_empty() {
                UNNAMED_TOOL.to_string()
            } else {
                name
            },
            h,
            d,
        })
        .collect();
    result.sort_by_key(|t| t.number);
    result
}

fn line_has_m6(line: &str) -> bool {
    line.contains("M06") || {
        if let Some(pos) = line.find("M6") {
            let rest = &line[pos..];
            !rest.starts_with("M06")
        } else {
            false
        }
    }
}

fn parse_fanuc_lathe(lines: &[String]) -> Vec<ToolInfo> {
    let mut tools: HashMap<u32, (String, u32, u32)> = HashMap::new();
    let mut current_tool = 0u32;
    let mut current_comment = String::new();

    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with('(') || trimmed.starts_with('<') {
            continue;
        }

        if trimmed.contains("M30") || trimmed.contains("M99") {
            if current_tool != 0 {
                tools.entry(current_tool)
                    .or_insert_with(|| (current_comment.clone(), 0, 0));
            }
            break;
        }

        if line_has_m6(trimmed) {
            continue;
        }

        if let Some(tool_num) = extract_tool_number(trimmed) {
            if current_tool != 0 {
                tools.entry(current_tool)
                    .or_insert_with(|| (current_comment.clone(), 0, 0));
            }
            current_tool = tool_num;
            let comment = extract_comment(trimmed).unwrap_or_default();
            if !comment.is_empty() {
                current_comment = comment;
            } else {
                current_comment.clear();
            }
        }
    }

    let mut result: Vec<ToolInfo> = tools
        .into_iter()
        .map(|(num, (name, _, _))| ToolInfo {
            number: num,
            name: if name.is_empty() {
                UNNAMED_TOOL.to_string()
            } else {
                name
            },
            h: 0,
            d: 0,
        })
        .collect();
    result.sort_by_key(|t| t.number);
    result
}

fn parse_sinumerik(lines: &[String]) -> Vec<ToolInfo> {
    let mut tools: HashMap<u32, String> = HashMap::new();

    for line in lines {
        if line_has_m6(line) {
            if let Some(tool_num) = extract_tool_number(line) {
                let comment = line
                    .split(';')
                    .nth(1)
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| UNNAMED_TOOL.to_string());
                tools.entry(tool_num).or_insert(comment);
            }
        }
    }

    let mut result: Vec<ToolInfo> = tools
        .into_iter()
        .map(|(num, name)| ToolInfo {
            number: num,
            name,
            h: 0,
            d: 0,
        })
        .collect();
    result.sort_by_key(|t| t.number);
    result
}

fn parse_heidenhain(lines: &[String]) -> Vec<ToolInfo> {
    let mut tools: HashMap<u32, String> = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        if lower.contains("tool call") {
            let after = lower.split("tool call").nth(1).unwrap_or("").trim();
            let first_word = after.split_whitespace().next().unwrap_or("");
            if let Ok(pos) = first_word.parse::<u32>() {
                let comment = if i > 0 {
                    let prev = &lines[i - 1];
                    if let Some(c_pos) = prev.find(';') {
                        prev[c_pos + 1..].trim().to_string()
                    } else if let Some(c_pos) = prev.find('*') {
                        prev[c_pos + 1..].trim().to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let name = if comment.is_empty() {
                    UNNAMED_TOOL.to_string()
                } else {
                    comment
                };
                tools.entry(pos).or_insert(name);
            }
        }
    }

    let mut result: Vec<ToolInfo> = tools
        .into_iter()
        .map(|(num, name)| ToolInfo {
            number: num,
            name,
            h: 0,
            d: 0,
        })
        .collect();
    result.sort_by_key(|t| t.number);
    result
}

fn format_tool_table(tools: &[ToolInfo], variant: &CncVariant) -> Vec<String> {
    match variant {
        CncVariant::FanucMilling => tools
            .iter()
            .map(|t| {
                format!(
                    "(T{:02} H{:02} D{:02} - {})",
                    t.number, t.h, t.d, t.name
                )
            })
            .collect(),
        CncVariant::FanucLathe => tools
            .iter()
            .map(|t| format!("(T{:04} - {})", t.number, t.name))
            .collect(),
        CncVariant::Sinumerik | CncVariant::Heidenhain => tools
            .iter()
            .map(|t| format!(";T{:02} - {}", t.number, t.name))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::ui::TestWriter;
    use std::collections::HashMap;
    use tempfile::TempDir;

    // ── Fanuc Milling ──────────────────────────────────────

    #[test]
    fn detects_fanuc_milling() {
        let lines = vec![
            "%".to_string(),
            "T1 M6 (10MM ENDMILL)".to_string(),
            "G43 H1".to_string(),
            "G41 D1".to_string(),
            "M30".to_string(),
        ];
        assert!(is_fanuc_milling(&lines));
    }

    #[test]
    fn detects_fanuc_milling_with_m06() {
        let lines = vec![
            "%".to_string(),
            "T01 M06 (10MM ENDMILL)".to_string(),
            "G43 H1".to_string(),
            "G41 D1".to_string(),
            "M30".to_string(),
        ];
        assert!(is_fanuc_milling(&lines));
    }

    #[test]
    fn detects_fanuc_lathe() {
        let lines = vec![
            "%".to_string(),
            "T0101 (ROUGH TURN)".to_string(),
            "G00 X50 Z5".to_string(),
            "M30".to_string(),
        ];
        assert!(!is_fanuc_milling(&lines));
    }

    #[test]
    fn parse_fanuc_milling_basic() {
        let lines = vec![
            "%".to_string(),
            "T1 M6 (10MM ENDMILL)".to_string(),
            "G43 H1".to_string(),
            "G41 D1".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_fanuc_milling(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].number, 1);
        assert_eq!(tools[0].name, "10MM ENDMILL");
        assert_eq!(tools[0].h, 1);
        assert_eq!(tools[0].d, 1);
    }

    #[test]
    fn parse_fanuc_milling_multiple_tools() {
        let lines = vec![
            "%".to_string(),
            "T1 M6 (FACE MILL)".to_string(),
            "G43 H1".to_string(),
            "G41 D1".to_string(),
            "M30".to_string(),
            "T2 M6 (ENDMILL)".to_string(),
            "G43 H2".to_string(),
            "G41 D2".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_fanuc_milling(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].number, 1);
    }

    #[test]
    fn parse_fanuc_milling_no_comment() {
        let lines = vec![
            "%".to_string(),
            "T1 M6".to_string(),
            "G43 H1".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_fanuc_milling(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, UNNAMED_TOOL);
    }

    #[test]
    fn parse_fanuc_milling_h_only() {
        let lines = vec![
            "%".to_string(),
            "T5 M6 (DRILL)".to_string(),
            "G43 H5".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_fanuc_milling(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].h, 5);
        assert_eq!(tools[0].d, 0);
    }

    #[test]
    fn parse_fanuc_milling_d_only() {
        let lines = vec![
            "%".to_string(),
            "T3 M6 (REAMER)".to_string(),
            "G41 D3".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_fanuc_milling(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].h, 0);
        assert_eq!(tools[0].d, 3);
    }

    // ── Fanuc Lathe ────────────────────────────────────────

    #[test]
    fn parse_fanuc_lathe_basic() {
        let lines = vec![
            "%".to_string(),
            "T0101 (ROUGH TURN)".to_string(),
            "G00 X50 Z5".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_fanuc_lathe(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].number, 101);
        assert_eq!(tools[0].name, "ROUGH TURN");
    }

    #[test]
    fn parse_fanuc_lathe_no_comment() {
        let lines = vec![
            "%".to_string(),
            "T0101".to_string(),
            "G00 X50 Z5".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_fanuc_lathe(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, UNNAMED_TOOL);
    }

    #[test]
    fn parse_fanuc_lathe_multiple() {
        let lines = vec![
            "%".to_string(),
            "T0101 (ROUGH)".to_string(),
            "G00 X50 Z5".to_string(),
            "T0202 (FINISH)".to_string(),
            "G00 X30 Z2".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_fanuc_lathe(&lines);
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].number, 101);
        assert_eq!(tools[1].number, 202);
    }

    #[test]
    fn parse_fanuc_lathe_skips_m6_lines() {
        let lines = vec![
            "%".to_string(),
            "T0101 (TURN)".to_string(),
            "T1 M6 (MILL)".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_fanuc_lathe(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].number, 101);
        assert_eq!(tools[0].name, "TURN");
    }

    // ── Sinumerik ──────────────────────────────────────────

    #[test]
    fn parse_sinumerik_basic() {
        let lines = vec![
            "MSG(\"TEST\")".to_string(),
            "T25 M6;FR 6".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_sinumerik(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].number, 25);
        assert_eq!(tools[0].name, "FR 6");
    }

    #[test]
    fn parse_sinumerik_no_comment() {
        let lines = vec![
            "MSG(\"TEST\")".to_string(),
            "T25 M6".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_sinumerik(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, UNNAMED_TOOL);
    }

    // ── Heidenhain ─────────────────────────────────────────

    #[test]
    fn parse_heidenhain_basic() {
        let lines = vec![
            "BEGIN PGM TEST MM".to_string(),
            "; 10MM ENDMILL".to_string(),
            "TOOL CALL 1 Z S1000".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_heidenhain(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].number, 1);
        assert_eq!(tools[0].name, "10MM ENDMILL");
    }

    #[test]
    fn parse_heidenhain_no_comment() {
        let lines = vec![
            "BEGIN PGM TEST MM".to_string(),
            "TOOL CALL 1 Z S1000".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_heidenhain(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, UNNAMED_TOOL);
    }

    #[test]
    fn parse_heidenhain_multiple() {
        let lines = vec![
            "BEGIN PGM TEST MM".to_string(),
            "; FIRST TOOL".to_string(),
            "TOOL CALL 1 Z S1000".to_string(),
            "; SECOND TOOL".to_string(),
            "TOOL CALL 2 Z S2000".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_heidenhain(&lines);
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].number, 1);
        assert_eq!(tools[0].name, "FIRST TOOL");
        assert_eq!(tools[1].number, 2);
        assert_eq!(tools[1].name, "SECOND TOOL");
    }

    // ── Format ─────────────────────────────────────────────

    #[test]
    fn format_fanuc_milling_table() {
        let tools = vec![ToolInfo {
            number: 1,
            name: "10MM ENDMILL".to_string(),
            h: 1,
            d: 1,
        }];
        let table = format_tool_table(&tools, &CncVariant::FanucMilling);
        assert_eq!(table, vec!["(T01 H01 D01 - 10MM ENDMILL)"]);
    }

    #[test]
    fn format_fanuc_lathe_table() {
        let tools = vec![ToolInfo {
            number: 101,
            name: "ROUGH TURN".to_string(),
            h: 0,
            d: 0,
        }];
        let table = format_tool_table(&tools, &CncVariant::FanucLathe);
        assert_eq!(table, vec!["(T0101 - ROUGH TURN)"]);
    }

    #[test]
    fn format_sinumerik_table() {
        let tools = vec![ToolInfo {
            number: 25,
            name: "FR 6".to_string(),
            h: 0,
            d: 0,
        }];
        let table = format_tool_table(&tools, &CncVariant::Sinumerik);
        assert_eq!(table, vec![";T25 - FR 6"]);
    }

    #[test]
    fn format_heidenhain_table() {
        let tools = vec![ToolInfo {
            number: 1,
            name: "10MM ENDMILL".to_string(),
            h: 0,
            d: 0,
        }];
        let table = format_tool_table(&tools, &CncVariant::Heidenhain);
        assert_eq!(table, vec![";T01 - 10MM ENDMILL"]);
    }

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

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("(T01 H01 D01 - 10MM ENDMILL)"));
    }

    #[test]
    fn execute_fanuc_lathe_inserts_table() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.nc");
        fs::write(
            &file_path,
            "%\r\nT0101 (ROUGH TURN)\r\nM30\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("fanuc_lathe_line".into(), "3".into());

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("(T0101 - ROUGH TURN)"));
    }

    #[test]
    fn execute_sinumerik_inserts_table() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.mpf");
        fs::write(
            &file_path,
            "MSG(\"TEST\")\r\nT25 M6;FR 6\r\nM30\r\n",
        )
        .unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let mut extra = HashMap::new();
        extra.insert("sinumerik_line".into(), "3".into());

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
        fs::write(&file_path, &[0u8; 100]).unwrap();

        let mut writer = TestWriter::new();
        let config = Config::default();
        let extra = HashMap::new();

        execute(file_path.to_str().unwrap(), &config, &extra, &mut writer).unwrap();

        let last = writer.output.last().unwrap();
        assert_eq!(last.0, Some(Color::Yellow));
        assert!(last.1.contains("Mazatrol"));
    }
}
