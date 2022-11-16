use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::SetTitle,
    Result,
};
use is_elevated::is_elevated;
use nc_renamer::wait_command;
use std::io::stdout;

fn main() -> Result<()> {
    execute!(
        stdout(),
        SetTitle("NC Renamer"),
        Print("Программа запущена с ")
    )?;
    if is_elevated() {
        execute!(
            stdout(),
            Print("правами "),
            SetForegroundColor(Color::Green),
            Print("администратора"),
            ResetColor,
            Print(".\n"),
        )?;
    } else {
        execute!(
            stdout(),
            SetForegroundColor(Color::Red),
            Print("ограниченными"),
            ResetColor,
            Print(" правами.\n")
        )?;
    }

    match wait_command() {
        nc_renamer::Command::Exit => {}
    }

    Ok(())
}
