use std::io::stdout;

use cnc_renamer::is_installed;
use crossterm::{
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::disable_raw_mode,
};
use is_elevated::is_elevated;

pub mod install;
pub mod show_about;
pub mod uninstall;

pub enum Command {
    Install,
    Uninstall,
    ShowAbout,
    Exit,
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
            Print(" Установить CNC Renamer и добавить в контекстное меню"),
        )
        .unwrap();
    } else if !is_elevated() && !is_installed() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Установить CNC Renamer и добавить в контекстное меню "),
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
            Print(" Удалить CNC Renamer и убрать из контекстного меню"),
        )
        .unwrap();
    } else {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Удалить CNC Renamer и убрать из контекстного меню "),
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
            if event.kind == crossterm::event::KeyEventKind::Press {
                match event.code {
                    KeyCode::Esc | KeyCode::Char('0') => {
                        command = Command::Exit;
                        break;
                    }
                    KeyCode::Char('1') => {
                        if is_elevated() {
                            if is_installed() {
                                command = Command::Uninstall;
                            } else {
                                command = Command::Install;
                            }

                            break;
                        }
                    }
                    KeyCode::Char('2') => {
                        command = Command::ShowAbout;
                        break;
                    }
                    //_ => println!("{:?}", event),
                    _ => (),
                }
            }
        }
    }
    disable_raw_mode().unwrap();
    command
}
