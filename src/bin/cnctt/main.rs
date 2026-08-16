mod clipboard;
mod windows;

use std::env;
use std::fs;

use cnc_remedy::text::decode_bytes;
use cnc_remedy::tool_list::{detect_variant, format_tool_table, parse_tools};

const NO_NUMBERS_FLAG: &str = "--no-numbers";

enum Source {
    Path(String),
    WindowTitle(String),
}

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let args: Vec<String> = env::args().collect();

    if args.iter().skip(1).any(|a| a == "--help" || a == "-h") {
        print_usage();
        return 0;
    }

    let print_number = !args.iter().any(|a| a == NO_NUMBERS_FLAG);
    let positional: Vec<&str> = args
        .iter()
        .skip(1)
        .map(String::as_str)
        .filter(|a| !a.starts_with('-'))
        .collect();

    let source = match positional.as_slice() {
        [] => match windows::find_cimco_windows().into_iter().next() {
            Some(title) => Source::WindowTitle(title),
            None => {
                eprintln!("Окна CIMCO Edit не найдены.");
                return 1;
            }
        },
        [file] => Source::Path(file.to_string()),
        _ => {
            print_usage();
            return 0;
        }
    };

    let (path, modified) = match source {
        Source::Path(p) => (p, false),
        Source::WindowTitle(title) => windows::file_path_from_title(&title),
    };

    if modified {
        println!(
            "Внимание: файл был изменен, но не сохранен. Будет прочитана последняя сохраненная версия."
        );
    }

    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Не удалось прочитать файл '{}': {e}", path);
            return 1;
        }
    };
    let content = decode_bytes(&bytes);
    let lines: Vec<String> = content.lines().map(String::from).collect();

    let variant = match detect_variant(&path, &lines) {
        Some(v) => v,
        None => {
            eprintln!("Mazatrol не поддерживается.");
            return 1;
        }
    };

    let tools = parse_tools(&lines, &variant);
    if tools.is_empty() {
        println!("Инструмента не найдено.");
        return 1;
    }

    let table = format_tool_table(&tools, &variant, print_number);
    let joined = table.join("\r\n");

    if let Err(e) = clipboard::set_text(&joined) {
        eprintln!("Ошибка копирования в буфер обмена: {e}");
        return 1;
    }

    println!("Таблица сформирована и скопирована в буфер обмена:");
    for row in &table {
        println!("{row}");
    }

    0
}

fn print_usage() {
    println!("NC Tool Table — таблица инструментов из УП ЧПУ в буфер обмена.");
    println!();
    println!("Использование:");
    println!("  cnctt                   первое окно CIMCO Edit -> таблица в буфер обмена");
    println!("  cnctt <файл>            таблица из указанного файла");
    println!("  cnctt --no-numbers      только комментарии, без номеров инструментов");
    println!("  cnctt --help, -h        показать справку");
}
