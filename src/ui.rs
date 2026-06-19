use crossterm::{
    cursor,
    event::{Event, KeyCode, read},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use std::io::{self, Write, stdout};
use unicode_segmentation::UnicodeSegmentation;

/// Абстракция над устройством вывода для результатов команд.
///
/// Каждая команда получает `&mut dyn OutputWriter` и сообщает о прогрессе
/// без привязки к реальному терминалу.
///
/// ## Реализации
/// * [`TerminalWriter`] — вывод на консоль через crossterm.
/// * [`TestWriter`] — захват вывода в память (для тестов).
pub trait OutputWriter {
    /// Печать обычного текста без цветов.
    fn print(&mut self, text: &str) -> io::Result<()>;

    /// Печать текста указанным цветом.
    fn print_colored(&mut self, color: Color, text: &str) -> io::Result<()>;

    /// Печать маркера успеха (`[ Ok ]`) с точками до конца строки.
    fn status_ok(&mut self) -> io::Result<()>;

    /// Печать маркера неудачи (`[ Неудача ]`) с точками до конца строки.
    fn status_bad(&mut self) -> io::Result<()>;

    /// Печать информационного текста (жёлтый, с точками до конца строки).
    fn status_info(&mut self, text: &str) -> io::Result<()>;
}

/// Вывод результатов команд в реальный терминал через crossterm.
pub struct TerminalWriter;

impl OutputWriter for TerminalWriter {
    fn print(&mut self, text: &str) -> io::Result<()> {
        execute!(stdout(), Print(text))
    }

    fn print_colored(&mut self, color: Color, text: &str) -> io::Result<()> {
        execute!(stdout(), SetForegroundColor(color), Print(text), ResetColor)
    }

    fn status_ok(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        stdout().flush()?;
        let (width, _) = terminal::size()?;
        let (used, _) = cursor::position()?;
        let free = (width - used) as usize - 7;
        let fill = String::from_utf8(vec![b'.'; free]).map_err(io::Error::other)?;
        execute!(
            stdout(),
            SetForegroundColor(Color::DarkGrey),
            Print(fill),
            SetForegroundColor(Color::Green),
            Print(" [ Ok ]"),
            ResetColor
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn status_bad(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        stdout().flush()?;
        let (width, _) = terminal::size()?;
        let (used, _) = cursor::position()?;
        let free = (width - used) as usize - 12;
        let fill = String::from_utf8(vec![b'.'; free]).map_err(io::Error::other)?;
        execute!(
            stdout(),
            SetForegroundColor(Color::DarkGrey),
            Print(fill),
            SetForegroundColor(Color::Red),
            Print(" [ Неудача ]"),
            ResetColor
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    fn status_info(&mut self, text: &str) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        stdout().flush()?;
        let (width, _) = terminal::size()?;
        let (used, _) = cursor::position()?;
        let text_width = text.graphemes(true).count();
        let free = (width - used) as usize;
        let fill = String::from_utf8(vec![b'.'; free.saturating_sub(text_width)])
            .map_err(io::Error::other)?;
        execute!(
            stdout(),
            SetForegroundColor(Color::DarkGrey),
            Print(fill),
            SetForegroundColor(Color::Yellow),
            Print(text),
            ResetColor
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

/// Захватывает вывод команд в память (используется в тестах).
#[cfg(test)]
pub struct TestWriter {
    /// Захваченные записи: `(опциональный цвет, текст)`.
    pub output: Vec<(Option<Color>, String)>,
}

#[cfg(test)]
impl TestWriter {
    pub fn new() -> Self {
        Self { output: Vec::new() }
    }
}

#[cfg(test)]
impl OutputWriter for TestWriter {
    fn print(&mut self, text: &str) -> io::Result<()> {
        self.output.push((None, text.to_string()));
        Ok(())
    }

    fn print_colored(&mut self, color: Color, text: &str) -> io::Result<()> {
        self.output.push((Some(color), text.to_string()));
        Ok(())
    }

    fn status_ok(&mut self) -> io::Result<()> {
        self.output.push((Some(Color::Green), "[ Ok ]".to_string()));
        Ok(())
    }

    fn status_bad(&mut self) -> io::Result<()> {
        self.output
            .push((Some(Color::Red), "[ Неудача ]".to_string()));
        Ok(())
    }

    fn status_info(&mut self, text: &str) -> io::Result<()> {
        self.output.push((Some(Color::Yellow), text.to_string()));
        Ok(())
    }
}

/// Ожидание нажатия любой клавиши.
pub fn pause() -> io::Result<()> {
    execute!(
        stdout(),
        Print("\n\nНажмите любую клавишу для продолжения...")
    )?;
    loop {
        if let Event::Key(event) = read()?
            && event.kind == crossterm::event::KeyEventKind::Press
        {
            return Ok(());
        }
    }
}

/// Приглашение "Нажмите 0 или Esc для возврата".
pub fn return_back() -> io::Result<()> {
    execute!(
        stdout(),
        SetForegroundColor(Color::Yellow),
        Print("\n\n[0]"),
        ResetColor,
        Print(" Назад"),
    )?;
    loop {
        if let Event::Key(event) = read()? {
            match event.code {
                KeyCode::Esc | KeyCode::Char('0') => return Ok(()),
                _ => (),
            }
        }
    }
}
