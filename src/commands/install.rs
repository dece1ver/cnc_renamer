use cnc_renamer::{
    install_key, pause, print_status, Status, INSTALL_EXECUTABLE_PATH, INSTALL_PATH,
    REG_BGDIR_COMMAND_PATH, REG_BGDIR_PATH, REG_DIR_COMMAND_PATH, REG_DIR_PATH,
    REG_FILE_COMMAND_PATH, REG_FILE_PATH, REG_SYSTEM_ENV_PATH,
};
use crossterm::{execute, style::Print};
use registry::{Data, Hive, Security};
use std::{
    fs,
    io::{self, stdout},
};

pub fn install(executable_path: &String) -> io::Result<()> {
    clearscreen::clear().unwrap();

    execute!(
        stdout(),
        Print("Создание директории........................")
    )?;
    match fs::create_dir_all(INSTALL_PATH) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    }

    execute!(
        stdout(),
        Print("\nКопирование программы......................")
    )?;
    match fs::copy(executable_path, INSTALL_EXECUTABLE_PATH) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    }

    execute!(
        stdout(),
        Print("\nСоздание ключа реестра для файлов..........")
    )?;
    match install_key(
        REG_FILE_PATH,
        REG_FILE_COMMAND_PATH,
        "%1",
        "Переименовать УП",
    ) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    };

    execute!(
        stdout(),
        Print("\nСоздание ключа реестра для папок...........")
    )?;
    match install_key(
        REG_DIR_PATH,
        REG_DIR_COMMAND_PATH,
        "%1",
        "Переименовать все УП в директории",
    ) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    };

    execute!(
        stdout(),
        Print("\nСоздание ключа реестра для папок (ф).......")
    )?;
    match install_key(
        REG_BGDIR_PATH,
        REG_BGDIR_COMMAND_PATH,
        "%V",
        "Переименовать все УП в директории",
    ) {
        Ok(_) => print_status(Status::Ok),
        Err(_) => print_status(Status::Bad),
    };

    execute!(
        stdout(),
        Print("\nУстановка в PATH...........................")
    )?;
    let key = Hive::LocalMachine.open(REG_SYSTEM_ENV_PATH, Security::AllAccess);
    match key {
        Ok(key) => {
            if let Ok(path) = key.value("Path") {
                let new_path = Data::String(format!("{};{}", path, INSTALL_PATH).parse().unwrap());
                if key.set_value("Path", &new_path).is_ok() {
                    print_status(Status::Ok);
                } else {
                    print_status(Status::Bad);
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
