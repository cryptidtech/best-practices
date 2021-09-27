use anyhow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // auto-convert io Errors
    #[error("io error")]
    IoError(#[from] std::io::Error),

    // auto-convert fmt Errors
    #[error("fmt error")]
    FmtError(#[from] std::fmt::Error),

    // log Error
    #[error("log error {0}")]
    LogError(String),

    // path does not point at a directory
    #[error("not a directory path")]
    NotADir(std::path::PathBuf),

    // path does not point at a file
    #[error("not a file path")]
    NotAFile(std::path::PathBuf),

    // invalid file format
    #[error("invalid file format {0}")]
    InvalidFormat(String),
}

// create a convenient alias
pub type Result<T> = anyhow::Result<T, Error>;
