use std::io::{stdout, Write};

use crossterm::{
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};

use crate::{
    config::{save_config, Config},
    ui::return_back,
};

pub fn show_settings(config: &mut Config) {
    loop {
        clearscreen::clear().unwrap();

        execute!(
            stdout(),
            Print("Настройки CNC Remedy\n\n"),
        )
        .unwrap();

        // Режим очереди
        execute!(
            stdout(),
            SetForegroundColor(Color::Yellow),
            Print("[1]"),
            ResetColor,
            Print(" Режим архивирования:   "),
        )
        .unwrap();

        if config.use_queue {
            execute!(
                stdout(),
                SetForegroundColor(Color::Cyan),
                Print("через очередь (служба)"),
                ResetColor,
            )
            .unwrap();
        } else {
            execute!(
                stdout(),
                SetForegroundColor(Color::Green),
                Print("прямое перемещение"),
                ResetColor,
            )
            .unwrap();
        }

        // Путь к очереди
        execute!(
            stdout(),
            Print("\n"),
            SetForegroundColor(Color::Yellow),
            Print("[2]"),
            ResetColor,
            Print(" Папка очереди:         "),
        )
        .unwrap();

        if config.queue_path.is_empty() {
            execute!(
                stdout(),
                SetForegroundColor(Color::DarkGrey),
                Print("не задана"),
                ResetColor,
            )
            .unwrap();
        } else {
            execute!(
                stdout(),
                SetForegroundColor(Color::Cyan),
                Print(&config.queue_path),
                ResetColor,
            )
            .unwrap();
        }

        execute!(
            stdout(),
            Print("\n\n"),
            SetForegroundColor(Color::Yellow),
            Print("[0]"),
            ResetColor,
            Print(" Назад"),
        )
        .unwrap();

        terminal::enable_raw_mode().unwrap();
        let key = loop {
            if let Event::Key(event) = read().unwrap() {
                if event.kind == crossterm::event::KeyEventKind::Press {
                    break event.code;
                }
            }
        };
        terminal::disable_raw_mode().unwrap();

        match key {
            KeyCode::Char('1') => {
                config.use_queue = !config.use_queue;
                if let Err(e) = save_config(config) {
                    eprintln!("Ошибка сохранения настроек: {e}");
                }
            }
            KeyCode::Char('2') => {
                edit_queue_path(config);
            }
            KeyCode::Esc | KeyCode::Char('0') => break,
            _ => {}
        }
    }
}

fn edit_queue_path(config: &mut Config) {
    clearscreen::clear().unwrap();

    execute!(
        stdout(),
        Print("Введите путь к папке очереди и нажмите Enter:\n"),
        SetForegroundColor(Color::DarkGrey),
        Print("Пример: \\\\serv\\share\\_queue\n\n"),
        ResetColor,
    )
    .unwrap();

    // Показываем текущее значение как подсказку
    if !config.queue_path.is_empty() {
        execute!(
            stdout(),
            Print("Текущее значение: "),
            SetForegroundColor(Color::Cyan),
            Print(&config.queue_path),
            ResetColor,
            Print("\n\n"),
        )
        .unwrap();
    }

    terminal::enable_raw_mode().unwrap();

    let mut input = String::new();
    execute!(stdout(), Print("> ")).unwrap();

    loop {
        if let Event::Key(event) = read().unwrap() {
            if event.kind == crossterm::event::KeyEventKind::Press {
                match event.code {
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        terminal::disable_raw_mode().unwrap();
                        return;
                    }
                    KeyCode::Backspace => {
                        if !input.is_empty() {
                            input.pop();
                            // Перерисовываем строку ввода
                            execute!(
                                stdout(),
                                crossterm::cursor::MoveToColumn(2),
                                terminal::Clear(terminal::ClearType::UntilNewLine),
                                Print(&input),
                            )
                            .unwrap();
                        }
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        execute!(stdout(), Print(c)).unwrap();
                    }
                    _ => {}
                }
                stdout().flush().unwrap();
            }
        }
    }

    terminal::disable_raw_mode().unwrap();

    let trimmed = input.trim().to_string();
    if !trimmed.is_empty() {
        config.queue_path = trimmed;
        if let Err(e) = save_config(config) {
            eprintln!("\nОшибка сохранения настроек: {e}");
            return_back();
        }
    }
}