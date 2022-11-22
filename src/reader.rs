use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, Read};
use std::path::Path;
use std::{io};

const _NAMELESS: &str = "Без названия";
const MAZATROL_EXTENSIONS: [&str; 2] = ["pbg", "pbd"];
const HEIDENHAIN_EXTENSIONS: [&str; 1] = ["h"];
const SINUMERIK_EXTENSIONS: [&str; 2] = ["mpf", "spf"];
const BAD_SYMBOLS: [char; 9] = ['<', '>', ':', '\"', '/', '\\', '|', '?', '*'];

pub fn get_cnc_name(file_path: &str) -> Option<String> {
    match get_extension(file_path) {
        None => get_fanuc_name(file_path),
        Some(ext) => {
            if MAZATROL_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                get_mazatrol_name(file_path)
            } else if HEIDENHAIN_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                get_heidenhain_name(file_path)
            } else if SINUMERIK_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                get_sinumerik_name(file_path)
            } else {
                get_fanuc_name(file_path)
            }
        }
    }
}

fn get_fanuc_name(file_path: &str) -> Option<String> {
    if let Ok(lines) = read_lines(file_path) {
        for (i, line) in lines.take(2).flatten().enumerate() {
            if i == 0 && line.starts_with('%') {
            } else if i == 1 && line.starts_with('<') {
                let name = line.split('<').nth(1).unwrap().split('>').next().unwrap();
                return Some(remove_bad_symbols(name));
            } else if i == 1 && line.starts_with('O') {
                let name = line.split('(').nth(1).unwrap().split(')').next().unwrap();
                return Some(remove_bad_symbols(name));
            } else {
                return None;
            }
        }
    }
    None
}

fn get_mazatrol_name(file_path: &str) -> Option<String> {
    println!("Mazatrol");
    if let Ok(mut f) = File::open(file_path) {
        let mut buffer = Vec::new();
        if f.read_to_end(&mut buffer).is_ok() {
            let text = String::from_utf8_lossy(buffer.as_ref());
            return Some(remove_bad_symbols(text.trim()[80..132].trim_matches('\0')));
        }
    }
    None
}

fn get_sinumerik_name(file_path: &str) -> Option<String> {
    if let Ok(lines) = read_lines(file_path) {
        if let Some(line)= lines.flatten().next() {
            return if line.starts_with("MSG") && line.contains('(') && line.contains(')') {
                let name = line.split('(').nth(1).unwrap().split(')').next().unwrap();
                Some(remove_bad_symbols(name))
            } else {
                None
            };
        }
    }
    None
}

fn get_heidenhain_name(_file_path: &str) -> Option<String> {
    todo!()
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
