use cnc_renamer::{install, show_about, uninstall, wait_command, Command};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{Clear, ClearType, SetTitle},
    Result,
};
use std::io::stdout;

fn main() -> Result<()> {
    clearscreen::clear().unwrap();
    execute!(
        stdout(),
        SetTitle("CNC Renamer"),
        Hide,
        Clear(ClearType::All)
    )?;

    loop {
        match wait_command() {
            Command::Exit => break,
            Command::ShowAbout => {
                show_about();
            }
            Command::Install => install()?,
            Command::Uninstall => uninstall()?,
        }
    }
    execute!(stdout(), Show)?;
    Ok(())
}
