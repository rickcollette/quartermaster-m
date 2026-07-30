use crate::error::{BroadsideError, Result};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub const SECTOR_SINGLE: usize = 128;
pub const SECTOR_DOUBLE: usize = 256;
pub const ATR_HEADER_SIZE: usize = 16;
pub const MAX_SECTORS: u32 = 65_535;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    Atr,
    Raw,
}
impl FromStr for MediaType {
    type Err = BroadsideError;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "a" | "atr" => Ok(Self::Atr),
            "r" | "raw" | "xfd" => Ok(Self::Raw),
            _ => Err(BroadsideError::InvalidArgument(format!(
                "unknown media type '{s}'"
            ))),
        }
    }
}
impl Display for MediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Atr => "ATR",
            Self::Raw => "RAW",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsType {
    Dos2,
    Sparta2,
}
impl FromStr for FsType {
    type Err = BroadsideError;
    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "a" | "dos2" | "atari" => Ok(Self::Dos2),
            "s" | "sparta" | "sparta2" => Ok(Self::Sparta2),
            _ => Err(BroadsideError::InvalidArgument(format!(
                "unknown filesystem '{s}'"
            ))),
        }
    }
}
impl Display for FsType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Dos2 => "Atari DOS 2",
            Self::Sparta2 => "SpartaDOS 2",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType {
    Standard,
    Sio2Ide,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub size_bytes: usize,
    pub size_sectors: u16,
    pub first_sector: u16,
    pub attributes: u8,
    pub file_id: u8,
}

impl FileEntry {
    pub fn locked(&self) -> bool {
        self.attributes & 0x20 != 0
    }
}
