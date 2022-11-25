use cnc_renamer::{
    install, pause, print_status, show_about, uninstall, wait_command, Command, Status,
};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{Clear, ClearType, SetTitle},
    Result,
};
use std::io::stdout;
use std::path::Path;
use std::{env, fs};

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
                if let Some(name) = reader::get_cnc_name(arg) {
                    let old_name = Path::new(arg);
                    if let Some(dir) = old_name.parent() {
                        let new_name = dir.join(name);
                        if fs::rename(old_name, new_name).is_ok() {
                            print_status(Status::Ok)
                        } else {
                            print_status(Status::Bad)
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
