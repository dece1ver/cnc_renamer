use std::io::{Write, stdin, stdout};

use crossterm::{
    event::{Event, KeyCode, read},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use is_elevated::is_elevated;

use crate::{
    config::{Config, save_config},
    error::AppResult,
    registry,
};

/// Интерактивный экран настроек CNC Remedy.
pub fn show_settings(config: &mut Config) -> AppResult<()> {
    loop {
        clearscreen::clear()?;

        execute!(stdout(), Print("Настройки CNC Remedy\n\n"))?;

        if registry::is_installed() {
            execute!(
                stdout(),
                SetForegroundColor(Color::Yellow),
                Print("[1]"),
                ResetColor,
                Print(" Команды контекстного меню"),
            )?;
            execute!(
                stdout(),
                Print("\n"),
                SetForegroundColor(Color::Yellow),
                Print("[2]"),
                ResetColor,
                Print(" Параметры команд"),
            )?;
            execute!(stdout(), Print("\n"))?;
        }

        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("[d]"),
            ResetColor,
            Print(" Сбросить настройки по умолчанию"),
        )?;

        execute!(
            stdout(),
            Print("\n\n"),
            SetForegroundColor(Color::Yellow),
            Print("[0]"),
            ResetColor,
            Print(" Назад"),
        )?;

        terminal::enable_raw_mode()?;
        let key = loop {
            if let Event::Key(event) = read()?
                && event.kind == crossterm::event::KeyEventKind::Press
            {
                break event.code;
            }
        };
        terminal::disable_raw_mode()?;

        match key {
            KeyCode::Char('1') if registry::is_installed() => {
                show_commands(config)?;
            }
            KeyCode::Char('2') if registry::is_installed() => {
                show_params(config)?;
            }
            KeyCode::Char('d') => {
                clearscreen::clear()?;
                *config = Config::default();
                if let Err(e) = save_config(config) {
                    execute!(
                        stdout(),
                        SetForegroundColor(Color::Red),
                        Print(format!("Ошибка сохранения настроек: {e}")),
                        ResetColor,
                    )?;
                } else {
                    execute!(
                        stdout(),
                        SetForegroundColor(Color::Green),
                        Print("Настройки сброшены на умолчания."),
                        ResetColor,
                    )?;
                }
                crate::ui::pause()?;
            }
            KeyCode::Esc | KeyCode::Char('0') => break Ok(()),
            _ => {}
        }
    }
}

/// Интерактивный подэкран включения/отключения команд.
fn show_commands(config: &mut Config) -> AppResult<()> {
    loop {
        clearscreen::clear()?;

        execute!(stdout(), Print("Команды контекстного меню\n\n"))?;

        let mut keys: Vec<&String> = config.commands.keys().collect();
        keys.sort();

        for (i, name) in keys.iter().enumerate() {
            let cmd = &config.commands[*name];
            let num = i + 1;
            let status = if cmd.enabled { "[x]" } else { "[ ]" };
            execute!(
                stdout(),
                SetForegroundColor(Color::Yellow),
                Print(format!("[{num}] ")),
                ResetColor,
                Print(format!("{status} ")),
                SetForegroundColor(if cmd.enabled {
                    Color::Green
                } else {
                    Color::DarkGrey
                }),
                Print(format!("{} ({})", cmd.label, name)),
                ResetColor,
                Print("\n"),
            )?;
        }

        let menu_name = config.context_menu_name.as_deref().unwrap_or("CNC Remedy");
        execute!(
            stdout(),
            Print("\nПодменю: "),
            SetForegroundColor(Color::Cyan),
            Print(menu_name),
            ResetColor,
            Print("\n"),
        )?;

        execute!(
            stdout(),
            Print("\n"),
            SetForegroundColor(Color::Yellow),
            Print("[r]"),
            ResetColor,
            Print(" Применить и переустановить контекстное меню"),
        )?;
        if !is_elevated() {
            execute!(
                stdout(),
                SetForegroundColor(Color::Red),
                Print(" (недоступно)"),
                ResetColor,
            )?;
        }

        execute!(
            stdout(),
            Print("\n"),
            SetForegroundColor(Color::Yellow),
            Print("[0]"),
            ResetColor,
            Print(" Назад"),
        )?;

        terminal::enable_raw_mode()?;
        let key = loop {
            if let Event::Key(event) = read()?
                && event.kind == crossterm::event::KeyEventKind::Press
            {
                break event.code;
            }
        };
        terminal::disable_raw_mode()?;

        match key {
            KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                let idx = (c as u8 - b'1') as usize;
                let mut keys: Vec<&String> = config.commands.keys().collect();
                keys.sort();
                if idx < keys.len() {
                    let name = keys[idx].clone();
                    if let Some(cmd) = config.commands.get_mut(&name) {
                        cmd.enabled = !cmd.enabled;
                        if let Err(e) = save_config(config) {
                            eprintln!("Ошибка сохранения настроек: {e}");
                        }
                    }
                }
            }
            KeyCode::Char('r') => {
                if is_elevated() {
                    clearscreen::clear()?;
                    match registry::install_all(config) {
                        Ok(_) => {
                            execute!(
                                stdout(),
                                SetForegroundColor(Color::Green),
                                Print("Контекстное меню обновлено."),
                                ResetColor,
                            )?;
                        }
                        Err(e) => {
                            execute!(
                                stdout(),
                                SetForegroundColor(Color::Red),
                                Print(format!("Ошибка: {e}")),
                                ResetColor,
                            )?;
                        }
                    }
                    crate::ui::pause()?;
                }
            }
            KeyCode::Esc | KeyCode::Char('0') => break Ok(()),
            _ => {}
        }
    }
}

/// Подэкран редактирования extra-параметров команд.
fn show_params(config: &mut Config) -> AppResult<()> {
    loop {
        clearscreen::clear()?;

        execute!(stdout(), Print("Параметры команд\n\n"))?;

        let mut keys: Vec<&String> = config.commands.keys().collect();
        keys.sort();

        for (i, name) in keys.iter().enumerate() {
            let num = i + 1;
            let cmd = &config.commands[*name];

            execute!(
                stdout(),
                SetForegroundColor(Color::Yellow),
                Print(format!("[{num}]")),
                ResetColor,
                Print(format!(" {} ({})", cmd.label, name)),
                Print("\n"),
            )?;

            let mut extra: Vec<&String> = cmd.extra.keys().collect();
            extra.sort();
            for key in &extra {
                let val = &cmd.extra[*key];
                execute!(
                    stdout(),
                    Print(format!("    {key} = ")),
                    SetForegroundColor(Color::Cyan),
                    Print(val),
                    ResetColor,
                    Print("\n"),
                )?;
            }
        }

        execute!(
            stdout(),
            Print("\n"),
            SetForegroundColor(Color::Yellow),
            Print("[0]"),
            ResetColor,
            Print(" Назад"),
        )?;

        terminal::enable_raw_mode()?;
        let key = loop {
            if let Event::Key(event) = read()?
                && event.kind == crossterm::event::KeyEventKind::Press
            {
                break event.code;
            }
        };
        terminal::disable_raw_mode()?;

        match key {
            KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                let idx = (c as u8 - b'1') as usize;
                let mut cmd_names: Vec<&String> = config.commands.keys().collect();
                cmd_names.sort();
                if idx < cmd_names.len() {
                    let name = cmd_names[idx].clone();
                    edit_command_params(config, &name)?;
                }
            }
            KeyCode::Esc | KeyCode::Char('0') => break Ok(()),
            _ => {}
        }
    }
}

/// Редактирование extra-параметров одной команды.
fn edit_command_params(config: &mut Config, cmd_name: &str) -> AppResult<()> {
    loop {
        clearscreen::clear()?;

        execute!(
            stdout(),
            Print("Параметры: "),
            SetForegroundColor(Color::Cyan),
            Print(cmd_name),
            ResetColor,
            Print("\n\n"),
        )?;

        let extra: Vec<String> = {
            let cmd = &config.commands[cmd_name];
            let mut k: Vec<String> = cmd.extra.keys().cloned().collect();
            k.sort();
            k
        };

        for (i, key) in extra.iter().enumerate() {
            let num = i + 1;
            let val = &config.commands[cmd_name].extra[key];
            execute!(
                stdout(),
                SetForegroundColor(Color::Yellow),
                Print(format!("[{num}]")),
                ResetColor,
                Print(format!(" {key} = ")),
                SetForegroundColor(Color::Cyan),
                Print(val),
                ResetColor,
                Print("\n"),
            )?;
        }

        execute!(
            stdout(),
            Print("\n"),
            SetForegroundColor(Color::Yellow),
            Print("[+]"),
            ResetColor,
            Print(" Добавить параметр"),
        )?;

        if !extra.is_empty() {
            execute!(
                stdout(),
                Print("\n"),
                SetForegroundColor(Color::Yellow),
                Print("[-]"),
                ResetColor,
                Print(" Удалить параметр"),
            )?;
        }

        execute!(
            stdout(),
            Print("\n\n"),
            SetForegroundColor(Color::Yellow),
            Print("[0]"),
            ResetColor,
            Print(" Назад"),
        )?;

        terminal::enable_raw_mode()?;
        let key = loop {
            if let Event::Key(event) = read()?
                && event.kind == crossterm::event::KeyEventKind::Press
            {
                break event.code;
            }
        };
        terminal::disable_raw_mode()?;

        match key {
            KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                let idx = (c as u8 - b'1') as usize;
                if idx < extra.len() {
                    let key = extra[idx].clone();
                    let cmd = &config.commands[cmd_name];
                    let val = cmd.extra.get(&key).cloned().unwrap_or_default();

                    let new_val = match val.to_ascii_lowercase().as_str() {
                        "true" => "false".to_string(),
                        "false" => "true".to_string(),
                        "now" => "modified".to_string(),
                        "modified" => "now".to_string(),
                        "starts-with" => "contains".to_string(),
                        "contains" => "starts-with".to_string(),
                        _ => {
                            execute!(stdout(), Print("\nНовое значение: "),)?;
                            terminal::disable_raw_mode()?;
                            stdout().flush()?;
                            let mut input = String::new();
                            stdin().read_line(&mut input)?;
                            terminal::enable_raw_mode()?;
                            input.trim().to_string()
                        }
                    };

                    if let Some(cmd) = config.commands.get_mut(cmd_name) {
                        cmd.extra.insert(key, new_val);
                    }
                    if let Err(e) = save_config(config) {
                        eprintln!("Ошибка сохранения: {e}");
                    }
                }
            }
            KeyCode::Char('+') => {
                execute!(stdout(), Print("\nИмя параметра: "),)?;
                terminal::disable_raw_mode()?;
                stdout().flush()?;
                let mut key_input = String::new();
                stdin().read_line(&mut key_input)?;
                terminal::enable_raw_mode()?;
                let key_name = key_input.trim().to_string();
                if key_name.is_empty() {
                    continue;
                }

                execute!(stdout(), Print("Значение: "),)?;
                terminal::disable_raw_mode()?;
                stdout().flush()?;
                let mut val_input = String::new();
                stdin().read_line(&mut val_input)?;
                terminal::enable_raw_mode()?;
                let val = val_input.trim().to_string();

                if let Some(cmd) = config.commands.get_mut(cmd_name) {
                    cmd.extra.insert(key_name, val);
                }
                if let Err(e) = save_config(config) {
                    eprintln!("Ошибка сохранения: {e}");
                }
            }
            KeyCode::Char('-') if !extra.is_empty() => {
                execute!(stdout(), Print("\nНомер параметра для удаления: "),)?;
                terminal::disable_raw_mode()?;
                stdout().flush()?;
                let mut del_input = String::new();
                stdin().read_line(&mut del_input)?;
                terminal::enable_raw_mode()?;
                let idx: usize = match del_input.trim().parse::<usize>() {
                    Ok(n) if n >= 1 && n <= extra.len() => n - 1,
                    _ => continue,
                };
                let key = extra[idx].clone();
                if let Some(cmd) = config.commands.get_mut(cmd_name) {
                    cmd.extra.remove(&key);
                }
                if let Err(e) = save_config(config) {
                    eprintln!("Ошибка сохранения: {e}");
                }
            }
            KeyCode::Esc | KeyCode::Char('0') => break Ok(()),
            _ => {}
        }
    }
}
