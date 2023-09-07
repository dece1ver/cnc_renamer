use cnc_renamer::{install, print_status, show_about, uninstall, wait_command, Command, Status};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{Clear, ClearType, SetTitle},
};
use std::io::stdout;
use std::path::Path;
use std::{env, fs};

use crate::reader::try_rename;

mod reader;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            clearscreen::clear().unwrap();
            execute!(
                stdout(),
                SetTitle("CNC Renamer"),
                Hide,
                Clear(ClearType::All)
            )?;

            loop {
                match wait_command() {
                    Command::Exit => break,
                    Command::ShowAbout => show_about(),
                    Command::Install => install(&args[0])?,
                    Command::Uninstall => uninstall()?,
                }
            }
            execute!(stdout(), Show)?;
        }
        _ => {
            for arg in args.iter().skip(1) {
                print!("{arg}");
                let path = Path::new(arg);
                if path.is_file() {
                    println!(" - файл.\n");
                    try_rename(arg);
                } else if path.is_dir() {
                    println!(" - директория.\n");
                    if let Ok(entries) = fs::read_dir(arg) {
                        entries
                            .filter_map(|entry| entry.ok())
                            .filter(|entry| entry.path().is_file())
                            .for_each(|entry| {
                                if let Ok(file_path) = entry.path().strip_prefix(Path::new(arg)) {
                                    if let Some(file_path_str) = file_path.to_str() {
                                        println!("└───{}", file_path_str);
                                        try_rename(file_path_str);
                                    }
                                }
                            });
                            cnc_renamer::pause();
                    } else {
                        println!("Не удалось прочитать содержимое папки.");
                    }
                }
            }
        }
    }
    Ok(())
}
