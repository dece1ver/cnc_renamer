use cnc_renamer::{install, pause, show_about, uninstall, wait_command, Command};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{Clear, ClearType, SetTitle},
    Result,
};
use std::env;
use std::io::stdout;

mod reader;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
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
                    Command::ShowAbout => show_about(),
                    Command::Install => install(&args[0])?,
                    Command::Uninstall => uninstall()?,
                }
            }
            execute!(stdout(), Show)?;
        }
        _ => {
            for arg in args.iter().skip(1) {
                println!("{}", arg);
                if let Some(name) = reader::get_cnc_name(arg) {
                    println!("{}", name);
                }
            }
            pause()
        }
    }

    Ok(())
}
