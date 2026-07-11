use std::collections::HashMap;
use std::io::stdout;

use crate::config::Config;
use crate::error::AppResult;
use crate::registry::is_installed;
use crate::ui::OutputWriter;
use crossterm::{
    event::{Event, KeyCode, read},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::disable_raw_mode,
};
use is_elevated::is_elevated;

/// Сигнатура зарегистрированного обработчика команды.
pub type CommandFn =
    fn(&str, &Config, &HashMap<String, String>, &mut dyn OutputWriter) -> AppResult<()>;

pub mod archive;
pub mod generate_tool_list;
pub mod install;
pub mod rename;
pub mod show_about;
pub mod show_settings;
pub mod strip_comments;
pub mod uninstall;

/// Возвращает словарь всех зарегистрированных CLI-команд.
pub fn registered_commands() -> HashMap<&'static str, CommandFn> {
    let mut map: HashMap<&'static str, CommandFn> = HashMap::new();
    map.insert("rename", rename::execute);
    map.insert("archive", archive::execute);
    map.insert("strip-comments", strip_comments::execute);
    map.insert("generate-tool-list", generate_tool_list::execute);
    map
}

/// Доступные команды TUI-меню.
pub enum Command {
    Install,
    Uninstall,
    ShowAbout,
    ShowSettings,
    Exit,
}

/// Ожидает выбора пользователя в интерактивном TUI-меню.
///
/// Отображает главное меню и возвращает выбранную [`Command`].
pub fn wait_command() -> AppResult<Command> {
    clearscreen::clear()?;
    if is_elevated() {
        execute!(
            stdout(),
            Print("Программа запущена с правами "),
            SetForegroundColor(Color::Green),
            Print("администратора"),
            ResetColor,
            Print(".\n"),
        )?;
    } else {
        execute!(
            stdout(),
            Print("Программа запущена c "),
            SetForegroundColor(Color::Red),
            Print("ограниченными"),
            ResetColor,
            Print(" правами.\n"),
        )?;
    }

    if is_elevated() && !is_installed() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Установить CNC Remedy и добавить в контекстное меню"),
        )?;
    } else if !is_elevated() && !is_installed() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Установить CNC Remedy и добавить в контекстное меню "),
            SetForegroundColor(Color::Red),
            Print("(недоступно)"),
            ResetColor,
        )?;
    } else if is_elevated() && is_installed() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Удалить CNC Remedy и убрать из контекстного меню"),
        )?;
    } else {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[1]"),
            ResetColor,
            Print(" Удалить CNC Remedy и убрать из контекстного меню "),
            SetForegroundColor(Color::Red),
            Print("(недоступно)"),
            ResetColor,
        )?;
    }

    execute!(
        stdout(),
        SetForegroundColor(Color::Yellow),
        Print("\n[2]"),
        ResetColor,
        Print(" О программе"),
    )?;

    if is_installed() {
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("\n[3]"),
            ResetColor,
            Print(" Настройки"),
        )?;
    }

    execute!(
        stdout(),
        SetForegroundColor(Color::Yellow),
        Print("\n\n[0]"),
        ResetColor,
        Print(" Закрыть программу"),
    )?;

    crossterm::terminal::enable_raw_mode()?;
    let command;
    loop {
        if let Event::Key(event) = read()?
            && event.kind == crossterm::event::KeyEventKind::Press
        {
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
                KeyCode::Char('3') if is_installed() => {
                    command = Command::ShowSettings;
                    break;
                }
                _ => (),
            }
        }
    }
    disable_raw_mode()?;
    Ok(command)
}
