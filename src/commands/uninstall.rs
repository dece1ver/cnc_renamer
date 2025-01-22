use std::{
    fs,
    io::{self, stdout},
};

use crossterm::{execute, style::Print};
use registry::{Data, Hive, Security};

use cnc_renamer::{
    pause, DisplayStatus, Status, INSTALL_EXECUTABLE_PATH, INSTALL_PATH, REG_ARCHIVE_COMMAND_PATH,
    REG_BGDIR_PATH, REG_DIR_PATH, REG_FILE_PATH, REG_SYSTEM_ENV_PATH,
};

pub fn uninstall() -> io::Result<()> {
    clearscreen::clear().unwrap();
    execute!(stdout(), Print("Удаление из контекстного меню файлов "))?;
    match Hive::ClassesRoot.delete(REG_FILE_PATH, true) {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    }
    execute!(stdout(), Print("\nУдаление из контекстного меню папок "))?;
    match Hive::ClassesRoot.delete(REG_DIR_PATH, true) {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    }
    execute!(
        stdout(),
        Print("\nУдаление из контекстного меню папок (ф) ")
    )?;
    match Hive::ClassesRoot.delete(REG_BGDIR_PATH, true) {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    }
    execute!(stdout(), Print("\nУдаление расширенной команды "))?;
    match Hive::ClassesRoot.delete(REG_ARCHIVE_COMMAND_PATH, true) {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    }
    execute!(stdout(), Print("\nУдаление файла "))?;
    match fs::remove_file(INSTALL_EXECUTABLE_PATH) {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    }
    execute!(stdout(), Print("\nУдаление из PATH "))?;
    let key = Hive::LocalMachine.open(REG_SYSTEM_ENV_PATH, Security::AllAccess);
    match key {
        Ok(key) => {
            if let Ok(path) = key.value("Path") {
                let new_path = Data::String(
                    path.to_string()
                        .replace(format!(";{}", INSTALL_PATH).as_str(), "")
                        .parse()
                        .unwrap(),
                );
                match key.set_value("Path", &new_path) {
                    Ok(_) => Status::Ok.print_status(),
                    Err(_) => Status::Bad.print_status(),
                }
            }
        }
        Err(e) => {
            Status::Bad.print_status();
            println!("{:#?}", e)
        }
    }
    pause();
    Ok(())
}
