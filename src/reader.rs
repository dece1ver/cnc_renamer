use std::ffi::OsStr;
use std::fs::{self, File};
use std::io;
use std::io::{BufRead, Read};
use std::path::Path;

use crate::{print_status, Status};

const _NAMELESS: &str = "Без названия";
const MAZATROL_EXTENSIONS: [&str; 2] = ["pbg", "pbd"];
const HEIDENHAIN_EXTENSIONS: [&str; 1] = ["h"];
const SINUMERIK_EXTENSIONS: [&str; 2] = ["mpf", "spf"];
const BAD_SYMBOLS: [char; 9] = ['<', '>', ':', '\"', '/', '\\', '|', '?', '*'];

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

fn get_fanuc_name(file_path: &str) -> Option<(String, &str)> {
    if let Ok(lines) = read_lines(file_path) {
        for (i, line) in lines.take(2).flatten().enumerate() {
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

fn get_mazatrol_name<'a>(file_path: &str, extension: &'a str) -> Option<(String, &'a str)> {
    println!("Mazatrol");
    if let Ok(mut f) = File::open(file_path) {
        let mut buffer = Vec::new();
        if f.read_to_end(&mut buffer).is_ok() {
            let mut name = String::new();
            for char in String::from_utf8_lossy(buffer.as_ref())
                .chars()
                .skip(80)
                .take(32)
            {
                name.push(char)
            }
            return Some((
                remove_bad_symbols(name.trim().trim_matches('\0')),
                extension,
            ));
        }
    }
    None
}

fn get_sinumerik_name<'a>(file_path: &str, extension: &'a str) -> Option<(String, &'a str)> {
    if let Ok(lines) = read_lines(file_path) {
        if let Some(line) = lines.flatten().next() {
            if line.starts_with("MSG") && line.contains('(') && line.contains(')') {
                if let Some(name) = line.split('(').nth(1) {
                    if let Some(name) = name.split(')').next() {
                        return Some((remove_bad_symbols(name.trim_matches('"')), extension));
                    }
                }
            }
        }
    }
    None
}

fn get_heidenhain_name<'a>(file_path: &str, extension: &'a str) -> Option<(String, &'a str)> {
    if let Ok(lines) = read_lines(file_path) {
        if let Some(line) = lines.take(1).flatten().next() {
            return if line.starts_with("BEGIN PGM") {
                Some((
                    remove_bad_symbols(
                        line.replace("BEGIN PGM ", "")
                            .trim_start_matches('0')
                            .trim(),
                    ),
                    extension,
                ))
            } else {
                None
            };
        }
    }
    None
}

fn get_extension(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn remove_bad_symbols(text: &str) -> String {
    let mut text = text.to_string();
    for bad_symbol in BAD_SYMBOLS {
        text = text.replace(bad_symbol, "-");
    }
    text
}

pub fn try_rename(file_path: &str) {
    if let Some((name, ext)) = get_cnc_name(file_path) {
        println!("    Найдена программа ЧПУ: {}", name);
        let old_name = Path::new(file_path);

        if let Some(dir) = old_name.parent() {
            let mut new_name = match ext.len() {
                0 => dir.join(&name),
                _ => dir.join(format!("{}.{}", name, ext)),
            };
            if new_name == old_name {
                println!("    Переименовывание не требуется.");
                return;
            };
            println!("    Новое имя: {:#?}", new_name);
            let mut copy: u32 = 0;
            while new_name.exists() {
                if new_name == old_name {
                    break;
                };
                copy += 1;
                new_name = match ext.len() {
                    0 => dir.join(format!("{} ({})", name, copy)),
                    _ => dir.join(format!("{} ({}).{}", name, copy, ext)),
                };
                println!(
                    "    Новый файл уже существует...\n    Проверка {:?}",
                    new_name
                );
            }
            print!("    Переименовывание...");
            if fs::rename(old_name, new_name).is_ok() {
                print_status(Status::Ok);
            } else {
                print_status(Status::Bad);
            }
        }
    }
    println!("\n")
}
