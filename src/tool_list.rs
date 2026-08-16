use std::collections::HashMap;
use std::path::Path;

/// Имя инструмента, когда комментарий в УП отсутствует.
pub const UNNAMED_TOOL: &str = "---";

/// Информация об инструменте, извлечённая из УП.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolInfo {
    /// Номер инструмента.
    pub number: u32,
    /// Комментарий/наименование инструмента.
    pub name: String,
    /// H-код (Fanuc фрезер).
    pub h: u32,
    /// D-код (Fanuc фрезер).
    pub d: u32,
}

/// Вариант системы ЧПУ, по которому разбирается УП.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CncVariant {
    FanucMilling,
    FanucLathe,
    Sinumerik,
    Heidenhain,
}

/// Определяет систему ЧПУ по расширению файла и содержимому.
///
/// Возвращает `None` для Mazatrol (не поддерживается).
pub fn detect_variant(file: &str, lines: &[String]) -> Option<CncVariant> {
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

/// Разбирает УП на список инструментов в зависимости от системы ЧПУ.
pub fn parse_tools(lines: &[String], variant: &CncVariant) -> Vec<ToolInfo> {
    match variant {
        CncVariant::FanucMilling => parse_fanuc_milling(lines),
        CncVariant::FanucLathe => parse_fanuc_lathe(lines),
        CncVariant::Sinumerik => parse_sinumerik(lines),
        CncVariant::Heidenhain => parse_heidenhain(lines),
    }
}

/// Формирует строки таблицы инструментов для вставки/вывода.
///
/// При `print_number = false` выводится только комментарий.
pub fn format_tool_table(
    tools: &[ToolInfo],
    variant: &CncVariant,
    print_number: bool,
) -> Vec<String> {
    match variant {
        CncVariant::FanucMilling => tools
            .iter()
            .map(|t| {
                if print_number {
                    format!("(T{:02} H{:02} D{:02} - {})", t.number, t.h, t.d, t.name)
                } else {
                    format!("({})", t.name)
                }
            })
            .collect(),
        CncVariant::FanucLathe => tools
            .iter()
            .map(|t| {
                if print_number {
                    format!("(T{:04} - {})", t.number, t.name)
                } else {
                    format!("({})", t.name)
                }
            })
            .collect(),
        CncVariant::Sinumerik | CncVariant::Heidenhain => tools
            .iter()
            .map(|t| {
                if print_number {
                    format!(";T{:02} - {}", t.number, t.name)
                } else {
                    format!(";{}", t.name)
                }
            })
            .collect(),
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
            let digits = after_t.bytes().take_while(u8::is_ascii_digit).count();
            if digits > 0 && line_has_m6(&after_t[digits..]) {
                return true;
            }
        }
    }
    false
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
                let entry = tools
                    .entry(current_tool)
                    .or_insert_with(|| (current_comment.clone(), 0, 0));
                if entry.0.is_empty() {
                    entry.0 = current_comment.clone();
                }
            }
            break;
        }

        if line_has_m6(trimmed)
            && let Some(tool_num) = extract_tool_number(trimmed)
        {
            if current_tool != 0 {
                let entry = tools
                    .entry(current_tool)
                    .or_insert_with(|| (current_comment.clone(), 0, 0));
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

        if trimmed.contains("G43")
            && trimmed.contains('H')
            && let Some(h) = extract_h_number(trimmed)
            && current_tool != 0
        {
            tools
                .entry(current_tool)
                .and_modify(|e| e.1 = h)
                .or_insert_with(|| (current_comment.clone(), h, 0));
        }

        if (trimmed.contains("G41") || trimmed.contains("G42"))
            && trimmed.contains('D')
            && let Some(d) = extract_d_number(trimmed)
            && current_tool != 0
        {
            tools
                .entry(current_tool)
                .and_modify(|e| e.2 = d)
                .or_insert_with(|| (current_comment.clone(), 0, d));
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
    line.contains("M06") || line.contains("M6")
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
                insert_lathe_tool(&mut tools, current_tool, &current_comment);
            }
            break;
        }

        if line_has_m6(trimmed) {
            continue;
        }

        if let Some(tool_num) = extract_tool_number(trimmed) {
            if current_tool != 0 {
                insert_lathe_tool(&mut tools, current_tool, &current_comment);
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

/// Вставляет инструмент токарного парсера; если имя уже пустое — заполняет комментарием.
fn insert_lathe_tool(tools: &mut HashMap<u32, (String, u32, u32)>, num: u32, comment: &str) {
    let entry = tools
        .entry(num)
        .or_insert_with(|| (comment.to_string(), 0, 0));
    if entry.0.is_empty() {
        entry.0 = comment.to_string();
    }
}

fn parse_sinumerik(lines: &[String]) -> Vec<ToolInfo> {
    let mut tools: HashMap<u32, String> = HashMap::new();

    for line in lines {
        if line_has_m6(line)
            && let Some(tool_num) = extract_tool_number(line)
        {
            let comment = line
                .split(';')
                .nth(1)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| UNNAMED_TOOL.to_string());
            tools.entry(tool_num).or_insert(comment);
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn parse_fanuc_lathe_fills_comment_on_repeat() {
        let lines = vec![
            "T0828G55 ".to_string(),
            "T0828G55(RASKOTNIK M8*1.25)".to_string(),
            "M30".to_string(),
        ];
        let tools = parse_fanuc_lathe(&lines);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].number, 828);
        assert_eq!(tools[0].name, "RASKOTNIK M8*1.25");
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
    fn format_with_numbers() {
        let cases = [
            (
                CncVariant::FanucMilling,
                ToolInfo {
                    number: 1,
                    name: "10MM ENDMILL".into(),
                    h: 1,
                    d: 1,
                },
                "(T01 H01 D01 - 10MM ENDMILL)",
            ),
            (
                CncVariant::FanucLathe,
                ToolInfo {
                    number: 101,
                    name: "ROUGH TURN".into(),
                    h: 0,
                    d: 0,
                },
                "(T0101 - ROUGH TURN)",
            ),
            (
                CncVariant::Sinumerik,
                ToolInfo {
                    number: 25,
                    name: "FR 6".into(),
                    h: 0,
                    d: 0,
                },
                ";T25 - FR 6",
            ),
            (
                CncVariant::Heidenhain,
                ToolInfo {
                    number: 1,
                    name: "10MM ENDMILL".into(),
                    h: 0,
                    d: 0,
                },
                ";T01 - 10MM ENDMILL",
            ),
        ];
        for (variant, tool, expected) in cases {
            assert_eq!(format_tool_table(&[tool], &variant, true), vec![expected]);
        }
    }

    #[test]
    fn format_without_numbers() {
        let cases = [
            (
                CncVariant::FanucMilling,
                ToolInfo {
                    number: 1,
                    name: "10MM ENDMILL".into(),
                    h: 1,
                    d: 1,
                },
                "(10MM ENDMILL)",
            ),
            (
                CncVariant::FanucLathe,
                ToolInfo {
                    number: 101,
                    name: "ROUGH TURN".into(),
                    h: 0,
                    d: 0,
                },
                "(ROUGH TURN)",
            ),
            (
                CncVariant::Sinumerik,
                ToolInfo {
                    number: 25,
                    name: "FR 6".into(),
                    h: 0,
                    d: 0,
                },
                ";FR 6",
            ),
            (
                CncVariant::Heidenhain,
                ToolInfo {
                    number: 1,
                    name: "10MM ENDMILL".into(),
                    h: 0,
                    d: 0,
                },
                ";10MM ENDMILL",
            ),
        ];
        for (variant, tool, expected) in cases {
            assert_eq!(format_tool_table(&[tool], &variant, false), vec![expected]);
        }
    }
}
