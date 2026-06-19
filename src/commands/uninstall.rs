use crate::config::config_dir;
use crate::error::{AppError, AppResult};
use crate::registry::{self, INSTALL_EXECUTABLE_PATH, INSTALL_PATH, REG_SYSTEM_ENV_PATH};
use crate::ui::{OutputWriter, TerminalWriter};
use ::registry::{Data, Hive, Security};
use std::fs;

/// Удаляет CNC Remedy из системы.
///
/// Удаляет записи реестра, исполняемый файл и очищает системную
/// переменную `PATH`. Требует права администратора.
pub fn uninstall() -> AppResult<()> {
    clearscreen::clear()?;
    let mut out = TerminalWriter;

    out.print("Удаление из контекстного меню ")?;
    match registry::uninstall_all() {
        Ok(_) => out.status_ok()?,
        Err(_) => out.status_bad()?,
    }

    out.print("\nУдаление файла ")?;
    match fs::remove_file(INSTALL_EXECUTABLE_PATH) {
        Ok(_) => out.status_ok()?,
        Err(_) => out.status_bad()?,
    }

    out.print("\nУдаление из PATH ")?;
    match Hive::LocalMachine.open(REG_SYSTEM_ENV_PATH, Security::AllAccess) {
        Ok(key) => {
            if let Ok(path_val) = key.value("Path") {
                let cleaned = path_val
                    .to_string()
                    .replace(format!(";{}", INSTALL_PATH).as_str(), "");
                let new_path = Data::String(cleaned.parse().map_err(|e| {
                    AppError::Registry(format!("ошибка парсинга пути реестра: {e}"))
                })?);
                match key.set_value("Path", &new_path) {
                    Ok(_) => out.status_ok()?,
                    Err(_) => out.status_bad()?,
                }
            }
        }
        Err(_) => {
            out.status_bad()?;
        }
    }

    out.print("\nУдаление конфига ")?;
    let cfg_dir = config_dir();
    match fs::remove_dir_all(&cfg_dir) {
        Ok(_) => out.status_ok()?,
        Err(_) => out.status_info(" [не найден]")?,
    }

    crate::ui::pause()?;
    Ok(())
}
