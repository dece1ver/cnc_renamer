mod commands;
mod config;
mod reader;
mod registry;
mod ui;

use commands::{
    install::install, show_about::show_about, show_settings::show_settings, uninstall::uninstall,
    wait_command, Command,
};
use config::load_config;
use crossterm::{
    cursor::{Hide, Show},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType, SetTitle},
};
use reader::{archive_program, try_rename};
use std::io::{self, stdout};
use std::path::Path;
use std::{env, fs};
use ui::{pause, Status};

use crate::ui::DisplayStatus;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut config = load_config();

    match args.len() {
        1 => {
            clearscreen::clear().unwrap();
            execute!(stdout(), SetTitle("CNC Remedy"), Hide, Clear(ClearType::All))?;
            loop {
                match wait_command() {
                    Command::Exit => break,
                    Command::ShowAbout => show_about(),
                    Command::ShowSettings => show_settings(&mut config),
                    Command::Install => install(&args[0])?,
                    Command::Uninstall => uninstall()?,
                }
            }
            execute!(stdout(), Show)?;
        }
        _ => {
            for arg in args.iter().skip(1) {
                if arg.starts_with('-') {
                    continue;
                }
                print!("{arg}");
                let path = Path::new(arg);
                if path.is_file() {
                    println!(" - файл.\n");
                    if args.contains(&"-arc".to_string()) {
                        match archive_program(arg, &config) {
                            Ok(_) => {
                                if config.use_queue {
                                    execute!(
                                        stdout(),
                                        SetForegroundColor(Color::Cyan),
                                        Print("Заявка на архивирование добавлена в очередь."),
                                        ResetColor,
                                        Print("\n"),
                                    )
                                    .unwrap();
                                } else {
                                    Status::Ok.print_status();
                                }
                            }
                            Err(e) => {
                                execute!(
                                    stdout(),
                                    SetForegroundColor(Color::Red),
                                    Print(format!("Ошибка: {e}")),
                                    ResetColor,
                                    Print("\n"),
                                )
                                .unwrap();
                            }
                        }
                    } else {
                        try_rename(arg);
                    }
                } else if path.is_dir() {
                    println!(" - директория.\n");
                    if let Ok(entries) = fs::read_dir(arg) {
                        entries
                            .filter_map(|entry| entry.ok())
                            .filter(|entry| entry.path().is_file())
                            .for_each(|entry| {
                                if let Ok(rel) = entry.path().strip_prefix(Path::new(arg)) {
                                    if let Some(s) = rel.to_str() {
                                        execute!(
                                            stdout(),
                                            SetForegroundColor(Color::DarkGrey),
                                            Print("└──"),
                                            ResetColor,
                                            Print(format!(" {s} ")),
                                        )
                                        .unwrap();
                                    }
                                }
                                if let Some(abs) = entry.path().to_str() {
                                    try_rename(abs);
                                }
                            });
                        pause();
                    } else {
                        println!("Не удалось прочитать содержимое папки.");
                    }
                }
            }
        }
    }
    Ok(())
}