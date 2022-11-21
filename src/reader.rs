use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

const NAMELESS: &str = "Без названия";
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

fn get_extension(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

fn get_fanuc_name(file_path: &str) -> Option<String> {
    if let Ok(lines) = read_lines(file_path) {
        let mut index = 0;
        for line in lines.take(2).flatten() {
            if index == 0 && line.starts_with('%') {
                index += 1;
                print!("Fanuc ");
            } else if index == 1 && line.starts_with('<') {
                let name = line.split('<').nth(1).unwrap().split('>').next().unwrap();
                println!("0i-TF");
                return Some(remove_bad_symbols(name));
            } else if index == 1 && line.starts_with('O') {
                println!("0i");
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
    // так не читает
    if let Ok(mut lines) = read_lines(file_path) {
        if let Some(Ok(name)) = lines.nth(1) {
            return Some(remove_bad_symbols(name.trim().trim_matches('\0')));
        }
    }
    None
}

fn get_sinumerik_name(_file_path: &str) -> Option<String> {
    todo!()
}

fn get_heidenhain_name(_file_path: &str) -> Option<String> {
    todo!()
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
