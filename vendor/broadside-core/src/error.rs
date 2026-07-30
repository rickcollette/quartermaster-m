use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum BroadsideError {
    Io(io::Error),
    InvalidArgument(String),
    InvalidImage(String),
    Unsupported(String),
    Filesystem(String),
    NotFound(String),
    AlreadyExists(String),
    NoSpace,
}

impl Display for BroadsideError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::InvalidArgument(s) => write!(f, "invalid argument: {s}"),
            Self::InvalidImage(s) => write!(f, "invalid image: {s}"),
            Self::Unsupported(s) => write!(f, "unsupported: {s}"),
            Self::Filesystem(s) => write!(f, "filesystem error: {s}"),
            Self::NotFound(s) => write!(f, "not found: {s}"),
            Self::AlreadyExists(s) => write!(f, "already exists: {s}"),
            Self::NoSpace => write!(f, "not enough free space in image"),
        }
    }
}

impl std::error::Error for BroadsideError {}

impl From<io::Error> for BroadsideError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub type Result<T> = std::result::Result<T, BroadsideError>;
