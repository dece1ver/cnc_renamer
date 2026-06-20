mod commands;
mod config;
mod error;
mod reader;
mod registry;
mod ui;

use commands::{
    Command, CommandFn, install::install, registered_commands, show_about::show_about,
    show_settings::show_settings, uninstall::uninstall, wait_command,
};
use config::{Config, load_config, save_config};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType, SetTitle},
};
use std::collections::HashMap;
use std::io::{self, stdout};
use std::path::Path;
use std::{env, fs};
use ui::{OutputWriter, TerminalWriter, pause};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 && args[1] == "--reset-config" {
        save_config(&Config::default())?;
        println!("Конфигурация сброшена на умолчания.");
        return Ok(());
    }

    let mut config = load_config();

    match args.len() {
        1 => {
            clearscreen::clear().map_err(|e| io::Error::other(e.to_string()))?;
            execute!(
                stdout(),
                SetTitle("CNC Remedy"),
                Hide,
                Clear(ClearType::All)
            )?;
            loop {
                match wait_command()? {
                    Command::Exit => break,
                    Command::ShowAbout => show_about()?,
                    Command::ShowSettings => show_settings(&mut config)?,
                    Command::Install => install(&args[0])?,
                    Command::Uninstall => uninstall()?,
                }
            }
            execute!(stdout(), Show)?;
        }
        _ => {
            if args[1] == "install" {
                return Ok(install(&args[0])?);
            }
            if args[1] == "uninstall" {
                return Ok(uninstall()?);
            }

            let cmds = registered_commands();
            let cmd_name = args[1].as_str();

            if !cmds.contains_key(cmd_name) {
                let names: Vec<&str> = cmds.keys().copied().collect();
                eprintln!("Неизвестная команда '{}'.", args[1]);
                eprintln!("Использование: cncr <команда> <файлы...>");
                eprintln!("Команды: {}", names.join(", "));
                return Ok(());
            }

            let cmd = cmds[cmd_name];
            let extra = config
                .commands
                .get(cmd_name)
                .map(|c| c.extra.clone())
                .unwrap_or_default();

            let mut out = TerminalWriter;

            for arg in args.iter().skip(2).filter(|a| !a.starts_with('-')) {
                let path = Path::new(arg);
                if path.is_dir() {
                    let recurse = extra.get("recurse").map(|s| s == "true").unwrap_or(false);
                    process_dir(path, recurse, &config, &extra, &mut out, &cmd)?;
                } else if path.is_file() {
                    print!("{arg}");
                    println!(" - файл.\n");
                    cmd(arg, &config, &extra, &mut out)?;
                }
            }
        }
    }
    Ok(())
}

fn process_dir(
    dir: &Path,
    recurse: bool,
    config: &Config,
    extra: &HashMap<String, String>,
    out: &mut TerminalWriter,
    cmd: &CommandFn,
) -> io::Result<()> {
    println!(" {} - директория.\n", dir.display());
    if recurse {
        process_dir_recursive(dir, config, extra, out, cmd)?;
    } else if let Ok(entries) = fs::read_dir(dir) {
        let files: Vec<_> = entries.flatten().filter(|e| e.path().is_file()).collect();
        if files.is_empty() {
            out.status_info(" [ нет файлов для обработки ]")?;
            pause()?;
            return Ok(());
        }
        for entry in &files {
            if let Some(name) = entry.file_name().to_str() {
                let _ = execute!(
                    stdout(),
                    SetForegroundColor(Color::DarkGrey),
                    Print("└──"),
                    ResetColor,
                    Print(format!(" {name} ")),
                );
            }
            if let Some(abs) = entry.path().to_str() {
                let _ = cmd(abs, config, extra, out);
            }
        }
    }
    pause()?;
    Ok(())
}

fn process_dir_recursive(
    dir: &Path,
    config: &Config,
    extra: &HashMap<String, String>,
    out: &mut TerminalWriter,
    cmd: &CommandFn,
) -> io::Result<()> {
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(current) = dirs.pop() {
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        let _ = execute!(
                            stdout(),
                            SetForegroundColor(Color::DarkGrey),
                            Print("└──"),
                            ResetColor,
                            Print(format!(" {name} ")),
                        );
                    }
                    if let Some(abs) = path.to_str() {
                        let _ = cmd(abs, config, extra, out);
                    }
                }
            }
        }
    }
    Ok(())
}
