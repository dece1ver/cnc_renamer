use std::{
    fs,
    io::{self, stdout},
};

use crossterm::{execute, style::Print};
use registry::{Data, Hive, Security};

use cnc_renamer::{
    pause, print_status, Status, INSTALL_EXECUTABLE_PATH, INSTALL_PATH, REG_BGDIR_PATH,
    REG_DIR_PATH, REG_FILE_PATH, REG_SYSTEM_ENV_PATH,
};

pub fn uninstall() -> io::Result<()> {
    clearscreen::clear().unwrap();
    execute!(
        stdout(),
        Print("Удаление из контекстного меню файлов.......")
    )?;
    match Hive::ClassesRoot.delete(REG_FILE_PATH, true) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    }
    execute!(
        stdout(),
        Print("\nУдаление из контекстного меню папок........")
    )?;
    match Hive::ClassesRoot.delete(REG_DIR_PATH, true) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    }
    execute!(
        stdout(),
        Print("\nУдаление из контекстного меню папок (ф)....")
    )?;
    match Hive::ClassesRoot.delete(REG_BGDIR_PATH, true) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    }
    execute!(
        stdout(),
        Print("\nУдаление файла.............................")
    )?;
    match fs::remove_file(INSTALL_EXECUTABLE_PATH) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    }
    execute!(
        stdout(),
        Print("\nУдаление из PATH...........................")
    )?;
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
                    Ok(_) => print_status(Status::Ok),
                    Err(_) => print_status(Status::Bad),
                }
            }
        }
        Err(e) => {
            print_status(Status::Bad);
            println!("{:#?}", e)
        }
    }
    pause();
    Ok(())
}
