use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::{stdout, Write};
use unicode_segmentation::UnicodeSegmentation;

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

pub fn pause() {
    execute!(stdout(), Print("\n\nНажмите любую клавишу для продолжения...")).unwrap();
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
                KeyCode::Esc | KeyCode::Char('0') => break,
                _ => (),
            }
        }
    }
}