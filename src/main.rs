use cnc_renamer::{install, print_status, show_about, uninstall, wait_command, Command, Status};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{Clear, ClearType, SetTitle},
    Result,
};
use std::io::stdout;
use std::path::Path;
use std::{env, fs};

mod reader;

fn main() -> Result<()> {
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
                println!("Аргумент: {}", arg);
                if let Some((name, ext)) = reader::get_cnc_name(arg) {
                    println!("Найдена программа ЧПУ: {}", name);
                    let old_name = Path::new(arg);

                    if let Some(dir) = old_name.parent() {
                        let mut new_name = match ext.len() {
                            0 => dir.join(&name),
                            _ => dir.join(format!("{}.{}", name, ext)),
                        };
                        if new_name == old_name {
                            break;
                        };
                        println!("Новый файл: {:#?}", new_name);
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
                            println!("Новый файл уже существует...\nПроверка {:?}", new_name);
                        }
                        print!("Переименовывание...");
                        if fs::rename(old_name, new_name).is_ok() {
                            print_status(Status::Ok)
                        } else {
                            print_status(Status::Bad)
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
