use crossterm::{
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, Clear, ClearType},
};
use is_elevated::is_elevated;
use std::io::stdout;

pub enum Command {
    ShowAbout,
    Exit,
}

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
            Print(" Удалить NC Renamer и убрать из контекстного меню. "),
        )
        .unwrap();
    } else if !is_elevated() && !is_installed() {
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
    } else if is_elevated() && is_installed() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Удалить NC Renamer и убрать из контекстного меню. "),
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
    execute!(stdout(), Clear(ClearType::All),).unwrap();
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
