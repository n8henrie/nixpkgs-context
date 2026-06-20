use std::{io, path::PathBuf, string::FromUtf8Error, sync::mpsc::SendError};

use thiserror::Error;
use tree_sitter::LanguageError;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Send(#[from] SendError<PathBuf>),

    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),

    #[error(transparent)]
    Language(#[from] LanguageError),

    #[error("parse error: {0}")]
    Parse(PathBuf),
}
