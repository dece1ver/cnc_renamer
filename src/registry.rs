use registry::{Data, Hive, Security};
use std::io;
use std::path::Path;

pub const INSTALL_PATH: &str = r"C:\Program Files\dece1ver\CNC Remedy";
pub const INSTALL_EXECUTABLE_PATH: &str = r"C:\Program Files\dece1ver\CNC Remedy\cncr.exe";
pub const REG_FILE_PATH: &str = r"*\shell\cnc_remedy";
pub const REG_DIR_PATH: &str = r"Directory\shell\cnc_remedy";
pub const REG_BGDIR_PATH: &str = r"Directory\Background\shell\cnc_remedy";
pub const REG_FILE_COMMAND_PATH: &str = r"*\shell\cnc_remedy\command";
pub const REG_DIR_COMMAND_PATH: &str = r"Directory\shell\cnc_remedy\command";
pub const REG_BGDIR_COMMAND_PATH: &str = r"Directory\Background\shell\cnc_remedy\command";
pub const REG_ARCHIVE_PATH: &str = r"*\shell\cnc_remedy_archive";
pub const REG_ARCHIVE_COMMAND_PATH: &str = r"*\shell\cnc_remedy_archive\command";
pub const REG_SYSTEM_ENV_PATH: &str =
    r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

pub fn is_installed() -> bool {
    if !Path::new(INSTALL_EXECUTABLE_PATH).exists()
        || Hive::ClassesRoot.open(REG_FILE_PATH, Security::Read).is_err()
        || Hive::ClassesRoot.open(REG_DIR_PATH, Security::Read).is_err()
        || Hive::ClassesRoot.open(REG_BGDIR_PATH, Security::Read).is_err()
        || Hive::ClassesRoot.open(REG_FILE_COMMAND_PATH, Security::Read).is_err()
        || Hive::ClassesRoot.open(REG_DIR_COMMAND_PATH, Security::Read).is_err()
        || Hive::ClassesRoot.open(REG_BGDIR_COMMAND_PATH, Security::Read).is_err()
        || Hive::ClassesRoot.open(REG_ARCHIVE_COMMAND_PATH, Security::Read).is_err()
    {
        return false;
    }
    if let Ok(key) = Hive::ClassesRoot.open(REG_SYSTEM_ENV_PATH, Security::Read) {
        if let Ok(path) = key.value("Path") {
            if !path.to_string().contains(INSTALL_PATH) {
                return false;
            }
        }
    }
    true
}

pub fn install_key<T: AsRef<str>>(
    base_key: &str,
    command_key: &str,
    args: &[T],
    command_name: &str,
) -> io::Result<()> {
    let key = Hive::ClassesRoot
        .create(base_key, Security::Write)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    key.set_value("", &Data::String(command_name.parse().unwrap()))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    key.set_value(
        "Icon",
        &Data::String(
            format!("\"{}\",2", INSTALL_EXECUTABLE_PATH).parse().unwrap(),
        ),
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let cmd_key = Hive::ClassesRoot
        .create(command_key, Security::Write)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let cmd_value = format!(
        "\"{}\" {}",
        INSTALL_EXECUTABLE_PATH,
        args.iter()
            .map(|a| format!("\"{}\"", a.as_ref()))
            .collect::<Vec<_>>()
            .join(" ")
    );
    cmd_key
        .set_value("", &Data::String(cmd_value.parse().unwrap()))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    Ok(())
}