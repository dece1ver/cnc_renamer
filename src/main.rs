use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{Clear, ClearType, SetTitle},
    Result,
};
use nc_renamer::{show_about, wait_command};
use std::io::stdout;

fn main() -> Result<()> {
    clearscreen::clear().unwrap();
    execute!(
        stdout(),
        SetTitle("NC Renamer"),
        Hide,
        Clear(ClearType::All)
    )?;

    loop {
        match wait_command() {
            nc_renamer::Command::Exit => break,
            nc_renamer::Command::ShowAbout => {
                show_about();
            }
        }
    }
    execute!(stdout(), Show,)?;
    Ok(())
}
