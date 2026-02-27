use crate::{
    config::Config, registry::{
        INSTALL_EXECUTABLE_PATH, INSTALL_PATH, REG_ARCHIVE_COMMAND_PATH, REG_ARCHIVE_PATH, REG_BGDIR_COMMAND_PATH, REG_BGDIR_PATH, REG_DIR_COMMAND_PATH, REG_DIR_PATH, REG_FILE_COMMAND_PATH, REG_FILE_PATH, REG_SYSTEM_ENV_PATH, install_key
    }, ui::{DisplayStatus, Status, pause}
};
use crossterm::{execute, style::Print};
use registry::{Data, Hive, Security};
use std::{
    fs,
    io::{self, stdout}, path::Path,
};

pub fn install(executable_path: &String) -> io::Result<()> {
    clearscreen::clear().unwrap();

    execute!(stdout(), Print("Создание директории "))?;
    match fs::create_dir_all(INSTALL_PATH) {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    }

    execute!(stdout(), Print("\nКопирование программы "))?;
    match fs::copy(executable_path, INSTALL_EXECUTABLE_PATH) {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    }

    let config_src = Path::new(executable_path).parent().unwrap().join("cncr.toml");
    let config_dst = Path::new(INSTALL_PATH).join("cncr.toml");
    execute!(stdout(), Print("\nКопирование конфига "))?;
    if config_dst.exists() {
        " [ уже существует ]".print_status();
    } else if config_src.exists() {
        match fs::copy(&config_src, &config_dst) {
            Ok(_) => Status::Ok.print_status(),
            Err(_) => Status::Bad.print_status(),
        }
    } else {
        // Создаём дефолтный если рядом с exe тоже нет
    match crate::config::save_config_to(&Config::default(), &config_dst) {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    }
}

    execute!(stdout(), Print("\nСоздание ключа реестра для файлов "))?;
    match install_key(REG_FILE_PATH, REG_FILE_COMMAND_PATH, &["%1"], "Переименовать УП") {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    };

    execute!(stdout(), Print("\nСоздание ключа реестра для папок "))?;
    match install_key(REG_DIR_PATH, REG_DIR_COMMAND_PATH, &["%1"], "Переименовать все УП в директории") {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    };

    execute!(stdout(), Print("\nСоздание ключа реестра для папок (ф) "))?;
    match install_key(REG_BGDIR_PATH, REG_BGDIR_COMMAND_PATH, &["%V"], "Переименовать все УП в директории") {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    };

    execute!(stdout(), Print("\nСоздание ключа реестра для файлов (архив)"))?;
    match install_key(REG_ARCHIVE_PATH, REG_ARCHIVE_COMMAND_PATH, &["%1", "-arc"], "Архивировать УП") {
        Ok(_) => Status::Ok.print_status(),
        Err(_) => Status::Bad.print_status(),
    };

    execute!(stdout(), Print("\nУстановка в PATH "))?;
    match Hive::LocalMachine.open(REG_SYSTEM_ENV_PATH, Security::AllAccess) {
        Ok(key) => {
            if let Ok(path) = key.value("Path") {
                let new_path = Data::ExpandString(format!("{};{}", path, INSTALL_PATH).parse().unwrap());
                if key.set_value("Path", &new_path).is_ok() {
                    Status::Ok.print_status();
                } else {
                    Status::Bad.print_status();
                }
            }
        }
        Err(e) => {
            Status::Bad.print_status();
            println!("{:#?}", e);
        }
    }

    pause();
    Ok(())
}