use std::io;

/// Ошибки приложения CNC Remedy.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Ошибка ввода-вывода.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// Ошибка разбора TOML-конфига.
    #[error("Config parse error: {0}")]
    ConfigParse(#[from] toml::de::Error),
    /// Ошибка сериализации TOML.
    #[error("Config serialize error: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),
    /// Ошибка работы с реестром Windows.
    #[error("Registry error: {0}")]
    Registry(String),
    /// Произвольная ошибка с сообщением.
    #[error("{0}")]
    Msg(String),
}

/// Псевдоним для `Result<T, AppError>`.
pub type AppResult<T> = Result<T, AppError>;

/// Преобразует любую ошибку с [`Display`] в [`AppError::Registry`].
pub fn registry_err(e: impl std::fmt::Display) -> AppError {
    AppError::Registry(e.to_string())
}

/// Позволяет `?` конвертировать `AppError` в `io::Error` в CLI-режиме.
impl From<AppError> for io::Error {
    fn from(e: AppError) -> Self {
        io::Error::other(e.to_string())
    }
}

/// Позволяет `?` конвертировать `clearscreen::Error` в `AppError`.
impl From<clearscreen::Error> for AppError {
    fn from(e: clearscreen::Error) -> Self {
        AppError::Msg(e.to_string())
    }
}
