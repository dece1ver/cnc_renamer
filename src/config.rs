use serde::{Deserialize, Serialize};
use std::{fs, io, path::{Path, PathBuf}};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    /// Использовать очередь вместо прямого перемещения
    pub use_queue: bool,
    /// Путь к папке очереди (куда пишутся заявки на перемещение)
    pub queue_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            use_queue: false,
            queue_path: String::new(),
        }
    }
}

pub fn config_path() -> PathBuf {
    std::env::current_exe()
        .unwrap_or_default()
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("cncr.toml")
}

pub fn load_config() -> Config {
    let path = config_path();
    if let Ok(content) = fs::read_to_string(&path) {
        toml::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    }
}

pub fn save_config(config: &Config) -> io::Result<()> {
    save_config_to(config, &config_path())
}

pub fn save_config_to(config: &Config, path: &Path) -> io::Result<()> {
    let content = toml::to_string_pretty(config)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    fs::write(path, content)
}