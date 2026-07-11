use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Настройки отдельной команды, хранящиеся в `cncr.toml`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandConfig {
    /// Надпись, отображаемая в контекстном меню.
    pub label: String,
    /// Включена ли команда в контекстном меню.
    pub enabled: bool,
    /// Цели контекстного меню: `file`, `directory`, `background`.
    pub targets: Vec<String>,
    /// Индекс иконки (ресурс в исполняемом файле). Больше не используется,
    /// оставлено для обратной совместимости конфига.
    #[serde(default)]
    pub icon: Option<u32>,
    /// Дополнительные ключи для команд. Содержимое зависит от команды:
    ///
    /// - `strip-comments`: `keep_string = "MSG,INFO"` — строки с `;`, содержащие
    ///   любой из фрагментов (через запятую), не удаляются.
    ///   `comment_chars = ";,/"` — символы комментариев (умолч. `";"`).
    /// - `rename`: `prefix = "NC_"`, `suffix = "_v2"` — добавляются к имени.
    ///   `overwrite = "true"` — при конфликте оставляет новейший файл.
    ///   `recurse = "true"` — обработка поддиректорий (только background).
    /// - `archive`: `date_format = "%d%m%y.%H%M"` — формат даты подпапки.
    ///   `timestamp_source = "now"` — `now` или `modified` (дата изменения файла).
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

/// Верхнеуровневая конфигурация приложения.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Имя корневого пункта контекстного меню.
    #[serde(default)]
    pub context_menu_name: Option<String>,
    /// Словарь команд: имя → [`CommandConfig`].
    #[serde(default)]
    pub commands: HashMap<String, CommandConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let mut commands = HashMap::new();
        commands.insert(
            "rename".into(),
            CommandConfig {
                label: "Переименовать УП".into(),
                enabled: true,
                targets: vec!["file".into(), "directory".into(), "background".into()],
                icon: Some(1),
                extra: {
                    let mut m = HashMap::new();
                    m.insert("prefix".into(), "".into());
                    m.insert("suffix".into(), "".into());
                    m.insert("overwrite".into(), "false".into());
                    m.insert("recurse".into(), "false".into());
                    m
                },
            },
        );
        commands.insert(
            "archive".into(),
            CommandConfig {
                label: "Архивировать УП".into(),
                enabled: true,
                targets: vec!["file".into()],
                icon: Some(2),
                extra: {
                    let mut m = HashMap::new();
                    m.insert("date_format".into(), "%d%m%y.%H%M".into());
                    m.insert("timestamp_source".into(), "now".into());
                    m
                },
            },
        );
        commands.insert(
            "strip-comments".into(),
            CommandConfig {
                label: "Очистить комментарии".into(),
                enabled: true,
                targets: vec!["file".into()],
                icon: Some(3),
                extra: {
                    let mut m = HashMap::new();
                    m.insert("keep_string".into(), "MSG".into());
                    m.insert("comment_chars".into(), ";".into());
                    m.insert("strip_mode".into(), "starts-with".into());
                    m
                },
            },
        );
        commands.insert(
            "generate-tool-list".into(),
            CommandConfig {
                label: "Сформировать список инструмента".into(),
                enabled: true,
                targets: vec!["file".into()],
                icon: None,
                extra: {
                    let mut m = HashMap::new();
                    m.insert("fanuc_milling_line".into(), "6".into());
                    m.insert("fanuc_lathe_line".into(), "6".into());
                    m.insert("sinumerik_line".into(), "6".into());
                    m.insert("heidenhain_line".into(), "3".into());
                    m
                },
            },
        );
        Self {
            context_menu_name: Some("CNC Remedy".into()),
            commands,
        }
    }
}
    
/// Путь к файлу конфигурации (`%ProgramData%\dece1ver\CNC Remedy\cncr.toml`).
pub fn config_path() -> PathBuf {
    if let Ok(progdata) = std::env::var("ProgramData") {
        Path::new(&progdata)
            .join("dece1ver")
            .join("CNC Remedy")
            .join("cncr.toml")
    } else {
        Path::new(".").join("cncr.toml")
    }
}

/// Путь к директории конфигурации.
pub fn config_dir() -> PathBuf {
    config_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf())
}

/// Загружает конфигурацию из `cncr.toml`, либо возвращает умолчания.
///
/// Если файл существует, но не парсится — возвращаются умолчания.
pub fn load_config() -> Config {
    let path = config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        toml::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

/// Сохраняет конфигурацию в стандартный путь.
pub fn save_config(config: &Config) -> io::Result<()> {
    save_config_to(config, &config_path())
}

/// Сохраняет конфигурацию по указанному пути.
///
/// Создаёт родительскую директорию, если её нет.
pub fn save_config_to(config: &Config, path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config).map_err(|e| io::Error::other(e.to_string()))?;
    fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid_toml() {
        let cfg = Config::default();
        let toml_str = toml::to_string_pretty(&cfg).expect("serialize");
        let deserialized: Config = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(deserialized.context_menu_name, cfg.context_menu_name);
        assert_eq!(deserialized.commands.len(), cfg.commands.len());
    }

    #[test]
    fn command_config_roundtrip() {
        let mut extra = HashMap::new();
        extra.insert("keep_string".into(), "MSG".into());
        let cmd = CommandConfig {
            label: "Test".into(),
            enabled: true,
            targets: vec!["file".into()],
            icon: Some(3),
            extra,
        };
        let toml_str = toml::to_string_pretty(&cmd).expect("serialize");
        let deserialized: CommandConfig = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(deserialized.label, "Test");
        assert_eq!(deserialized.extra["keep_string"], "MSG");
    }

    #[test]
    fn config_path_ends_with_cncr_toml() {
        let path = config_path();
        assert!(path.file_name().map(|n| n == "cncr.toml").unwrap_or(false));
    }

    #[test]
    fn load_result_returns_err_on_bad_input() {
        let result: Result<Config, _> = toml::from_str("this is not toml [[[");
        assert!(result.is_err());
    }
}
