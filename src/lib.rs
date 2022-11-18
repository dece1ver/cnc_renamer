use crossterm::{
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::disable_raw_mode,
};
use is_elevated::is_elevated;
use registry::{Data, Hive, Security};
use std::io::stdout;
use std::path::Path;
use std::{env, fs, io};

pub enum Command {
    Install,
    ShowAbout,
    Exit,
}

enum Status {
    Ok,
    Bad,
}

const INSTALL_PATH: &str = r"C:\ProgramData\dece1ver\nc_renamer";
const REG_BASE_PATH: &str = r"*\shell\nc_renamer";
const REG_COMMAND_PATH: &str = r"*\shell\nc_renamer\command";

pub fn pause() {
    execute!(stdout(), Print("Нажмите любую клавишу для продолжения..."),).unwrap();
    loop {
        if let Event::Key(event) = read().unwrap() {
            if let KeyCode::Char(_) = event.code {
                break;
            }
        }
    }
}

fn return_back() {
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

fn is_installed() -> bool {
    false
}

pub fn wait_command() -> Command {
    clearscreen::clear().unwrap();
    if is_elevated() {
        execute!(
            stdout(),
            Print("Программа запущена с правами "),
            SetForegroundColor(Color::Green),
            Print("администратора"),
            ResetColor,
            Print(".\n"),
        )
        .unwrap();
    } else {
        execute!(
            stdout(),
            Print("Программа запущена c "),
            SetForegroundColor(Color::Red),
            Print("ограниченными"),
            ResetColor,
            Print(" правами.\n")
        )
        .unwrap();
    }

    if is_elevated() && !is_installed() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Установить NC Renamer и добавить в контекстное меню."),
        )
        .unwrap();
    } else if !is_elevated() && !is_installed() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Установить NC Renamer и добавить в контекстное меню. "),
            SetForegroundColor(Color::Red),
            Print("(недоступно)"),
            ResetColor
        )
        .unwrap();
    } else if is_elevated() && is_installed() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Удалить NC Renamer и убрать из контекстного меню."),
        )
        .unwrap();
    } else {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Удалить NC Renamer и убрать из контекстного меню. "),
            SetForegroundColor(Color::Red),
            Print("(недоступно)"),
            ResetColor
        )
        .unwrap();
    }

    execute!(
        stdout(),
        SetForegroundColor(Color::Yellow),
        Print("\n[2]"),
        ResetColor,
        Print(" О программе "),
    )
    .unwrap();
    execute!(
        stdout(),
        SetForegroundColor(Color::Yellow),
        Print("\n\n[0]"),
        ResetColor,
        Print(" Закрыть программу"),
    )
    .unwrap();

    crossterm::terminal::enable_raw_mode().unwrap();
    let command;
    loop {
        if let Event::Key(event) = read().unwrap() {
            match event.code {
                KeyCode::Esc | KeyCode::Char('0') => {
                    command = Command::Exit;
                    break;
                }
                KeyCode::Char('1') => {
                    if is_elevated() {
                        command = Command::Install;
                        break;
                    }
                }
                KeyCode::Char('2') => {
                    command = Command::ShowAbout;
                    break;
                }
                //_ => println!("{:?}", event.code),
                _ => (),
            }
        }
    }
    disable_raw_mode().unwrap();
    command
}

pub fn show_about() {
    clearscreen::clear().unwrap();
    execute!(stdout(), Print("Программа переименовывает файлы управляющих программ по названию самой управляющей программы.\n
Поддерживаемые СЧПУ:
* Fanuc 0i           [ O0001(НАЗВАНИЕ) ]
* Fanuc 0i-*F        [ <НАЗВАНИЕ> ]
* Mazatrol Smart     [ .PBG | .PBD ]
* Sinumerik 840D sl  [ MSG (\"НАЗВАНИЕ\") ]
* Hiedenhain         [ BEGIN PGM НАЗВАНИЕ MM ]

Использование программы подразумевается вызовом через контекстное меню (ПКМ) по переменовываемым файлам.
Работа программы происходит без каких-либо уведомлений.
Если программа определяет файл как УП, то происходит переименование.
Если файл уже существует, он не перезаписывается, а создается копия с добавлением номера."),).unwrap();

    return_back()
}

pub fn install() -> io::Result<()> {
    clearscreen::clear().unwrap();
    let args: Vec<String> = env::args().collect();
    let executable_path = Path::new(INSTALL_PATH).join("nc_renamer.exe");
    let reg_executable_path = executable_path.to_str().unwrap();

    execute!(stdout(), Print("Создание директории..."))?;
    match fs::create_dir_all(INSTALL_PATH) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    }
    execute!(stdout(), Print("\nКопирование программы..."))?;
    match fs::copy(&args[0], &executable_path) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    }

    execute!(stdout(), Print("\nСоздание ключа реестра..."))?;
    let key = Hive::ClassesRoot.create(REG_BASE_PATH, Security::Write);
    match key {
        Ok(key) => {
            print_status(Status::Ok);
            execute!(stdout(), Print("\nВнесение параметров..."))?;

            match (
                key.set_value("", &Data::String("Переименовать УП".parse().unwrap())),
                key.set_value(
                    "Icon",
                    &Data::String(format!("\"{}\",0", reg_executable_path).parse().unwrap()),
                ),
            ) {
                (Ok(_), Ok(_)) => {
                    print_status(Status::Ok);
                    execute!(stdout(), Print("\nУстановка комманды..."))?;
                    let key = Hive::ClassesRoot.create(REG_COMMAND_PATH, Security::Write);
                    match key {
                        Ok(key) => {
                            if key
                                .set_value(
                                    "",
                                    &Data::String(
                                        format!("\"{}\" \"%1\"", reg_executable_path)
                                            .parse()
                                            .unwrap(),
                                    ),
                                )
                                .is_ok()
                            {
                                print_status(Status::Ok);
                            }
                        }
                        Err(_) => {
                            print_status(Status::Bad);
                        }
                    }
                }
                _ => {
                    print_status(Status::Bad);
                }
            }
        }
        Err(_) => {
            print_status(Status::Bad);
        }
    }
    return_back();
    Ok(())
}

fn print_status(status: Status) {
    match status {
        Status::Ok => {
            execute!(
                stdout(),
                SetForegroundColor(Color::Green),
                Print("Ok"),
                ResetColor
            )
            .unwrap();
        }
        Status::Bad => {
            execute!(
                stdout(),
                SetForegroundColor(Color::Red),
                Print("Неудача"),
                ResetColor
            )
            .unwrap();
        }
    }
}
