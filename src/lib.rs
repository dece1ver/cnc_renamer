use crossterm::{
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::disable_raw_mode,
};
use is_elevated::is_elevated;
use std::io::{stdout, Read};

pub enum Command {
    Exit,
}

pub fn pause() {
    let _ = std::io::stdin().read(&mut [0]).unwrap();
}

fn is_installed() -> bool {
    false
}

pub fn wait_command() -> Command {
    if is_elevated() && !is_installed() {
        execute!(
            stdout(),
            Print("\n[1] Установить NC Renamer и добавить в контекстное меню."),
        )
        .unwrap();
    } else if !is_elevated() && !is_installed() {
        execute!(
            stdout(),
            Print("\n[1] Установить NC Renamer и добавить в контекстное меню. "),
            SetForegroundColor(Color::Red),
            Print("(недоступно)"),
            ResetColor
        )
        .unwrap();
    } else if is_elevated() && is_installed() {
        execute!(
            stdout(),
            Print("\n[1] Удалить NC Renamer и убрать из контекстного меню. "),
        )
        .unwrap();
    } else {
        execute!(
            stdout(),
            Print("\n[1] Удалить NC Renamer и убрать из контекстного меню. "),
            SetForegroundColor(Color::Red),
            Print("(недоступно)"),
            ResetColor
        )
        .unwrap();
    }

    execute!(stdout(), Print("\n\n[0] Закрыть программу. "),).unwrap();

    crossterm::terminal::enable_raw_mode().unwrap();
    let command;
    loop {
        if let Event::Key(event) = read().unwrap() {
            match event.code {
                KeyCode::Esc | KeyCode::Char('0') => {
                    command = Command::Exit;
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
