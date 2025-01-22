use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, stdout, BufRead, Read};
use std::path::Path;

use chrono::Local;
use crossterm::execute;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};

use crate::{DisplayStatus, Status};

const _NAMELESS: &str = "Без названия";
const MAZATROL_EXTENSIONS: [&str; 2] = ["pbg", "pbd"];
const HEIDENHAIN_EXTENSIONS: [&str; 1] = ["h"];
const SINUMERIK_EXTENSIONS: [&str; 2] = ["mpf", "spf"];
const BAD_SYMBOLS: [char; 9] = ['<', '>', ':', '\"', '/', '\\', '|', '?', '*'];
const ARCHIVE_DIR_NAME: &str = "_";

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
        let old_path = Path::new(file_path);

        if let Some(dir) = old_path.parent() {
            let mut new_name = match ext.len() {
                0 => String::from(&name),
                _ => {
                    format!("{name}.{ext}")
                }
            };
            let mut new_path = dir.join(&new_name);
            let mut copy: u32 = 0;
            while new_path.exists() {
                if new_path == old_path {
                    " [ не требуется ]".print_status();
                    return;
                };
                copy += 1;
                new_name = match ext.len() {
                    0 => format!("{name} ({copy})"),
                    _ => format!("{name} ({copy}).{ext}"),
                };
                new_path = dir.join(&new_name);
            }
            execute!(
                stdout(),
                SetForegroundColor(Color::DarkGrey),
                Print("-> "),
                SetForegroundColor(Color::Cyan),
                Print(format!("{new_name} ")),
                ResetColor,
            )
            .unwrap();
            if fs::rename(old_path, new_path).is_ok() {
                Status::Ok.print_status();
            } else {
                Status::Bad.print_status();
            }
        }
    } else {
        " [ не программа или отсутствует имя ]".print_status();
    }
}

pub fn archive_program(file_path: impl AsRef<Path>) -> io::Result<()> {
    let path = file_path.as_ref();

    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Файл '{}' не найден", path.display()),
        ));
    }

    let parent_dir = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Невозможно получить родительскую директорию",
        )
    })?;

    let archive_base = parent_dir.join(ARCHIVE_DIR_NAME);
    fs::create_dir_all(&archive_base)?;

    let timestamp = Local::now().format("%d%m%y.%H%M").to_string();
    let archive_dir = archive_base.join(timestamp);

    fs::create_dir_all(&archive_dir)?;

    let file_name = path.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "Невозможно получить имя файла")
    })?;

    let dest_path = archive_dir.join(file_name);

    if dest_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("Файл '{}' уже существует в архиве", dest_path.display()),
        ));
    }
    fs::rename(path, &dest_path)?;
    Ok(())
}
