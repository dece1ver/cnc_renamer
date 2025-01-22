pub mod reader;

use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use unicode_segmentation::UnicodeSegmentation;

use registry::{Data, Hive, Security};
use std::io::{self, stdout, Write};
use std::path::Path;
use std::string::ToString;

pub trait DisplayStatus {
    fn print_status(&self);
}

pub enum Status {
    Ok,
    Bad,
}

impl DisplayStatus for Status {
    fn print_status(&self) {
        terminal::enable_raw_mode().unwrap();
        stdout().flush().unwrap();
        let (width, _) = terminal::size().unwrap();
        let (used, _) = cursor::position().unwrap();
        match self {
            Status::Ok => {
                let free = (width - used) as usize - 7;
                let fill = String::from_utf8(vec![b'.'; free]).expect("Invalid UTF-8");
                execute!(
                    stdout(),
                    SetForegroundColor(Color::DarkGrey),
                    Print(fill),
                    SetForegroundColor(Color::Green),
                    Print(" [ Ok ]"),
                    ResetColor
                )
                .unwrap();
            }
            Status::Bad => {
                let free = (width - used) as usize - 12;
                let fill = String::from_utf8(vec![b'.'; free]).expect("Invalid UTF-8");
                execute!(
                    stdout(),
                    SetForegroundColor(Color::DarkGrey),
                    Print(fill),
                    SetForegroundColor(Color::Red),
                    Print(" [ Неудача ]"),
                    ResetColor
                )
                .unwrap();
            }
        }
        terminal::disable_raw_mode().unwrap();
    }
}

impl DisplayStatus for &str {
    fn print_status(&self) {
        terminal::enable_raw_mode().unwrap();
        stdout().flush().unwrap();
        let (width, _) = terminal::size().unwrap();
        let (used, _) = cursor::position().unwrap();
        let free = (width - used) as usize;
        let fill = String::from_utf8(vec![b'.'; free - self.graphemes(true).count()])
            .expect("Invalid UTF-8");
        execute!(
            stdout(),
            SetForegroundColor(Color::DarkGrey),
            Print(fill),
            SetForegroundColor(Color::Yellow),
            Print(self),
            ResetColor
        )
        .unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}

pub const INSTALL_PATH: &str = r"C:\Program Files\dece1ver\CNC Renamer";
pub const INSTALL_EXECUTABLE_PATH: &str = r"C:\Program Files\dece1ver\CNC Renamer\cncr.exe";
pub const REG_FILE_PATH: &str = r"*\shell\cnc_renamer";
pub const REG_DIR_PATH: &str = r"Directory\shell\cnc_renamer";
pub const REG_BGDIR_PATH: &str = r"Directory\Background\shell\cnc_renamer";
pub const REG_FILE_COMMAND_PATH: &str = r"*\shell\cnc_renamer\command";
pub const REG_DIR_COMMAND_PATH: &str = r"Directory\shell\cnc_renamer\command";
pub const REG_BGDIR_COMMAND_PATH: &str = r"Directory\Background\shell\cnc_renamer\command";
pub const REG_ARCHIVE_PATH: &str = r"*\shell\cnc_renamer_archive";
pub const REG_ARCHIVE_COMMAND_PATH: &str = r"*\shell\cnc_renamer_archive\command";
pub const REG_SYSTEM_ENV_PATH: &str =
    r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

pub fn pause() {
    execute!(
        stdout(),
        Print("\n\nНажмите любую клавишу для продолжения..."),
    )
    .unwrap();
    loop {
        if let Event::Key(event) = read().unwrap() {
            if event.kind == crossterm::event::KeyEventKind::Press {
                break;
            }
        }
    }
}

pub fn return_back() {
    execute!(
        stdout(),
        SetForegroundColor(Color::Yellow),
        Print("\n\n[0]"),
        ResetColor,
        Print(" Назад"),
    )
    .unwrap();
    loop {
        if let Event::Key(event) = read().unwrap() {
            match event.code {
                KeyCode::Esc | KeyCode::Char('0') => {
                    break;
                }
                //_ => println!("{:?}", event.code),
                _ => (),
            }
        }
    }
}

pub fn is_installed() -> bool {
    if !Path::new(INSTALL_EXECUTABLE_PATH).exists()
        || Hive::ClassesRoot
            .open(REG_FILE_PATH, Security::Read)
            .is_err()
        || Hive::ClassesRoot
            .open(REG_DIR_PATH, Security::Read)
            .is_err()
        || Hive::ClassesRoot
            .open(REG_BGDIR_PATH, Security::Read)
            .is_err()
        || Hive::ClassesRoot
            .open(REG_FILE_COMMAND_PATH, Security::Read)
            .is_err()
        || Hive::ClassesRoot
            .open(REG_DIR_COMMAND_PATH, Security::Read)
            .is_err()
        || Hive::ClassesRoot
            .open(REG_BGDIR_COMMAND_PATH, Security::Read)
            .is_err()
        || Hive::ClassesRoot
            .open(REG_ARCHIVE_COMMAND_PATH, Security::Read)
            .is_err()
    {
        return false;
    }
    if let Ok(key) = Hive::ClassesRoot.open(REG_SYSTEM_ENV_PATH, Security::Read) {
        if let Ok(path) = key.value("Path") {
            if !path.to_string().contains(INSTALL_PATH) {
                return false;
            }
        }
    }
    true
}

pub fn install_key<T: AsRef<str>>(
    base_key: &str,
    command_key: &str,
    args: &[T],
    command_name: &str,
) -> io::Result<()> {
    let key = Hive::ClassesRoot.create(base_key, Security::Write);
    if let Ok(key) = key {
        if let (Ok(_), Ok(_)) = (
            key.set_value("", &Data::String(command_name.parse().unwrap())),
            key.set_value(
                "Icon",
                &Data::String(
                    format!("\"{}\",2", INSTALL_EXECUTABLE_PATH)
                        .parse()
                        .unwrap(),
                ),
            ),
        ) {
            let key = Hive::ClassesRoot.create(command_key, Security::Write);
            if let Ok(key) = key {
                if let Err(err) = key.set_value(
                    "",
                    &Data::String(
                        format!(
                            "\"{}\" {}",
                            INSTALL_EXECUTABLE_PATH,
                            args.iter()
                                .map(|arg| format!("\"{}\"", arg.as_ref()))
                                .collect::<Vec<String>>()
                                .join(" ")
                        )
                        .parse()
                        .unwrap(),
                    ),
                ) {
                    return Err(io::Error::new(io::ErrorKind::Other, err.to_string()));
                }
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Ошибка создания ключа реестра.",
            ));
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Ошибка создания ключа реестра.",
        ));
    }
    Ok(())
}
