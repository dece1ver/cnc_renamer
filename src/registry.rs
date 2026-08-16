use crate::config::Config;
use crate::error::{AppResult, registry_err};
use ::registry::{Data, Hive, Security};
use std::path::Path;

/// Путь установки программы.
pub const INSTALL_PATH: &str = r"C:\Program Files\dece1ver\CNC Remedy";
/// Полный путь к установленному исполняемому файлу.
pub const INSTALL_EXECUTABLE_PATH: &str = r"C:\Program Files\dece1ver\CNC Remedy\cncr.exe";
/// Ключ реестра для подменю контекстного меню.
const SUBMENU_KEY: &str = "CNCRemedy";
/// Префикс пути для записей реестра.
const CLASSES_PREFIX: &str = r"Software\Classes";
/// Путь в реестре к системной переменной `PATH`.
pub const REG_SYSTEM_ENV_PATH: &str =
    r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

/// Суффиксы ключей реестра для разных целей контекстного меню.
const TARGETS: &[(&str, &str)] = &[
    ("file", "*"),
    ("directory", "Directory"),
    ("background", "Directory\\Background"),
];

/// Проверяет, полностью ли установлен CNC Remedy.
pub fn is_installed() -> bool {
    if !Path::new(INSTALL_EXECUTABLE_PATH).exists() {
        return false;
    }
    for &(target_name, target_key) in TARGETS {
        let key = if target_name == "background" {
            format!(r"{CLASSES_PREFIX}\{target_key}\shell\rename")
        } else {
            format!(r"{CLASSES_PREFIX}\{target_key}\shell\{SUBMENU_KEY}")
        };
        if Hive::LocalMachine.open(&key, Security::Read).is_err() {
            return false;
        }
    }
    if let Ok(key) = Hive::LocalMachine.open(REG_SYSTEM_ENV_PATH, Security::Read)
        && let Ok(path) = key.value("Path")
        && !path.to_string().contains(INSTALL_PATH)
    {
        return false;
    }
    true
}

/// Создаёт все записи реестра для контекстного меню (HKLM).
pub fn install_all(config: &Config) -> AppResult<()> {
    remove_legacy()?;
    remove_current()?;

    for &(target_name, target_key) in TARGETS {
        let shell_base = format!(r"{CLASSES_PREFIX}\{target_key}\shell\{SUBMENU_KEY}");

        if target_name == "background" {
            let shell_root = format!(r"{CLASSES_PREFIX}\{target_key}\shell");

            let _ = Hive::LocalMachine.delete(&shell_base, true);

            let mut cmd_names: Vec<&String> = config.commands.keys().collect();
            cmd_names.sort();

            for name in &cmd_names {
                let cmd_cfg = &config.commands[*name];
                if !cmd_cfg.enabled {
                    continue;
                }
                if !cmd_cfg.targets.iter().any(|t| t.as_str() == "background") {
                    continue;
                }

                let verb_path = format!(r"{shell_root}\{name}");
                let command_path = format!(r"{verb_path}\command");

                let _ = Hive::LocalMachine.delete(&verb_path, true);

                let verb_key = Hive::LocalMachine
                    .create(&verb_path, Security::Write)
                    .map_err(registry_err)?;
                verb_key
                    .set_value(
                        "",
                        &Data::String(cmd_cfg.label.parse().map_err(registry_err)?),
                    )
                    .map_err(registry_err)?;

                verb_key
                    .set_value(
                        "Icon",
                        &Data::String(
                            format!("\"{INSTALL_EXECUTABLE_PATH}\",0")
                                .parse()
                                .map_err(registry_err)?,
                        ),
                    )
                    .map_err(registry_err)?;

                let cmd_key = Hive::LocalMachine
                    .create(&command_path, Security::Write)
                    .map_err(registry_err)?;
                let cmd_value = format!("\"{INSTALL_EXECUTABLE_PATH}\" {name} \"%V\"");
                cmd_key
                    .set_value("", &Data::String(cmd_value.parse().map_err(registry_err)?))
                    .map_err(registry_err)?;
            }
        } else {
            let _ = Hive::LocalMachine.delete(&shell_base, true);

            let menu_key = Hive::LocalMachine
                .create(&shell_base, Security::Write)
                .map_err(registry_err)?;

            let menu_name = config.context_menu_name.as_deref().unwrap_or("CNC Remedy");

            menu_key
                .set_value(
                    "MUIVerb",
                    &Data::String(menu_name.parse().map_err(registry_err)?),
                )
                .map_err(registry_err)?;

            menu_key
                .set_value(
                    "SubCommands",
                    &Data::String("".parse().map_err(registry_err)?),
                )
                .map_err(registry_err)?;

            menu_key
                .set_value(
                    "Icon",
                    &Data::String(
                        format!("\"{INSTALL_EXECUTABLE_PATH}\",0")
                            .parse()
                            .map_err(registry_err)?,
                    ),
                )
                .map_err(registry_err)?;

            let shell_container = format!(r"{shell_base}\Shell");
            Hive::LocalMachine
                .create(&shell_container, Security::Write)
                .map_err(registry_err)?;

            let mut cmd_index = 0u32;
            let mut cmd_names: Vec<&String> = config.commands.keys().collect();
            cmd_names.sort();

            for name in &cmd_names {
                let cmd_cfg = &config.commands[*name];
                if !cmd_cfg.enabled {
                    continue;
                }
                if !cmd_cfg.targets.iter().any(|t| t.as_str() == target_name) {
                    continue;
                }

                cmd_index += 1;
                let sort_prefix = format!("{:02}", cmd_index);
                let verb_name = format!("{sort_prefix}-{name}");
                let verb_path = format!(r"{shell_container}\{verb_name}");
                let command_path = format!(r"{verb_path}\command");

                let verb_key = Hive::LocalMachine
                    .create(&verb_path, Security::Write)
                    .map_err(registry_err)?;
                verb_key
                    .set_value(
                        "",
                        &Data::String(cmd_cfg.label.parse().map_err(registry_err)?),
                    )
                    .map_err(registry_err)?;

                let cmd_key = Hive::LocalMachine
                    .create(&command_path, Security::Write)
                    .map_err(registry_err)?;
                let cmd_value = format!("\"{INSTALL_EXECUTABLE_PATH}\" {name} \"%1\"");
                cmd_key
                    .set_value("", &Data::String(cmd_value.parse().map_err(registry_err)?))
                    .map_err(registry_err)?;
            }
        }
    }

    Ok(())
}

/// Удаляет старые ключи реестра (legacy v1.x и cnc_renamer/nc_renamer).
fn remove_legacy() -> AppResult<()> {
    let legacy_keys = [
        // cnc_remedy (v1.x с подчёркиванием)
        r"*\shell\cnc_remedy",
        r"Directory\shell\cnc_remedy",
        r"Directory\Background\shell\cnc_remedy",
        r"*\shell\cnc_remedy_archive",
        // cnc_renamer
        r"*\shell\cnc_renamer",
        r"Directory\shell\cnc_renamer",
        r"Directory\Background\shell\cnc_renamer",
        r"*\shell\cnc_renamer_archive",
        // nc_renamer (самая первая версия)
        r"*\shell\nc_renamer",
    ];
    for key in &legacy_keys {
        let path = format!(r"{CLASSES_PREFIX}\{key}");
        let _ = Hive::LocalMachine.delete(&path, true);
        let _ = Hive::CurrentUser.delete(&path, true);
        let _ = Hive::ClassesRoot.delete(*key, true);
    }
    Ok(())
}

/// Удаляет ключи текущей версии (CNCRemedy) из HKLM, HKCU и HKCR.
fn remove_current() -> AppResult<()> {
    let keys = [
        r"*\shell\CNCRemedy",
        r"Directory\shell\CNCRemedy",
        r"Directory\Background\shell\CNCRemedy",
        r"Directory\Background\shell\rename",
    ];
    for key in &keys {
        let path = format!(r"{CLASSES_PREFIX}\{key}");
        let _ = Hive::LocalMachine.delete(&path, true);
        let _ = Hive::CurrentUser.delete(&path, true);
        let _ = Hive::ClassesRoot.delete(*key, true);
    }
    Ok(())
}

/// Удаляет все записи CNC Remedy из реестра.
pub fn uninstall_all() -> AppResult<()> {
    remove_legacy()?;
    remove_current()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_valid_paths() {
        assert!(INSTALL_PATH.contains("CNC Remedy"));
        assert!(INSTALL_EXECUTABLE_PATH.contains("cncr.exe"));
        assert!(REG_SYSTEM_ENV_PATH.contains("Environment"));
    }

    #[test]
    fn targets_are_valid() {
        assert_eq!(TARGETS.len(), 3);
        assert_eq!(TARGETS[0], ("file", "*"));
        assert_eq!(TARGETS[1], ("directory", "Directory"));
        assert_eq!(TARGETS[2], ("background", "Directory\\Background"));
    }
}
