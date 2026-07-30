use std::str::FromStr;

use crate::error::{BroadsideError, Result};

/// Common Atari 8-bit floppy and hard-disk style geometries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Geometry {
    Single90K,
    Enhanced130K,
    Double180K,
    DoubleSided360K,
    Quad720K,
    Sparta16M,
}

impl Geometry {
    pub const fn sectors(self) -> u32 {
        match self {
            Self::Single90K => 720,
            Self::Enhanced130K => 1040,
            Self::Double180K => 720,
            Self::DoubleSided360K => 1440,
            Self::Quad720K => 2880,
            Self::Sparta16M => 65_535,
        }
    }

    pub const fn sector_size(self) -> usize {
        match self {
            Self::Single90K | Self::Enhanced130K => 128,
            Self::Double180K | Self::DoubleSided360K | Self::Quad720K | Self::Sparta16M => 256,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Single90K => "Atari 90K single density (720x128)",
            Self::Enhanced130K => "Atari 130K enhanced density (1040x128)",
            Self::Double180K => "Atari 180K double density (720x256)",
            Self::DoubleSided360K => "Atari 360K double-sided double density (1440x256)",
            Self::Quad720K => "Atari 720K quad density (2880x256)",
            Self::Sparta16M => "SpartaDOS 16 MiB-class image (65535x256)",
        }
    }
}

impl FromStr for Geometry {
    type Err = BroadsideError;

    fn from_str(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "s" | "sd" | "90k" | "single" => Ok(Self::Single90K),
            "m" | "ed" | "130k" | "enhanced" => Ok(Self::Enhanced130K),
            "d" | "dd" | "180k" | "double" => Ok(Self::Double180K),
            "ds" | "360k" | "double-sided" => Ok(Self::DoubleSided360K),
            "q" | "qd" | "720k" | "quad" => Ok(Self::Quad720K),
            "16m" | "16mb" | "sparta16m" | "f" => Ok(Self::Sparta16M),
            _ => Err(BroadsideError::InvalidArgument(format!(
                "unknown geometry '{value}'"
            ))),
        }
    }
}
