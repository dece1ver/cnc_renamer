use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::registry::{self, INSTALL_EXECUTABLE_PATH, INSTALL_PATH, REG_SYSTEM_ENV_PATH};
use crate::ui::{OutputWriter, TerminalWriter};
use ::registry::{Data, Hive, Security};
use std::fs;

/// Устанавливает CNC Remedy в систему.
///
/// Копирует исполняемый файл и конфиг в `C:\Program Files\dece1ver\CNC Remedy`,
/// регистрирует пункты контекстного меню и добавляет путь в системную
/// переменную `PATH`. Требует права администратора.
pub fn install(executable_path: &str) -> AppResult<()> {
    clearscreen::clear()?;
    let mut out = TerminalWriter;

    out.print("Создание директории ")?;
    match fs::create_dir_all(INSTALL_PATH) {
        Ok(_) => out.status_ok()?,
        Err(_) => out.status_bad()?,
    }

    out.print("\nКопирование программы ")?;
    match fs::copy(executable_path, INSTALL_EXECUTABLE_PATH) {
        Ok(_) => out.status_ok()?,
        Err(_) => out.status_bad()?,
    }

    let config_dir = crate::config::config_path()
        .parent()
        .ok_or_else(|| AppError::Msg("нет родительской директории у пути конфига".into()))?
        .to_path_buf();
    out.print("\nСоздание директории конфига ")?;
    if fs::create_dir_all(&config_dir).is_ok() || config_dir.exists() {
        out.status_ok()?;
    } else {
        out.status_bad()?;
    }

    let config_path = crate::config::config_path();
    out.print("\nПроверка конфига ")?;
    if config_path.exists() {
        out.status_info(" [ уже существует ]")?;
    } else {
        crate::config::save_config_to(&Config::default(), &config_path)?;
        out.status_ok()?;
    }

    out.print("\nДобавление контекстного меню ")?;
    let installed_config = crate::config::load_config();
    match registry::install_all(&installed_config) {
        Ok(_) => out.status_ok()?,
        Err(e) => {
            let _ = e;
            out.status_bad()?
        }
    }

    out.print("\nУстановка в PATH ")?;
    match Hive::LocalMachine.open(REG_SYSTEM_ENV_PATH, Security::AllAccess) {
        Ok(key) => {
            if let Ok(path_val) = key.value("Path") {
                let new_path =
                    Data::ExpandString(format!("{};{}", path_val, INSTALL_PATH).parse().map_err(
                        |e| AppError::Registry(format!("ошибка парсинга пути реестра: {e}")),
                    )?);
                if key.set_value("Path", &new_path).is_ok() {
                    out.status_ok()?;
                } else {
                    out.status_bad()?;
                }
            }
        }
        Err(_) => {
            out.status_bad()?;
        }
    }

    crate::ui::pause()?;
    Ok(())
}
