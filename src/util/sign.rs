use super::error::{Error, Result};
use std::fmt::{Display, Formatter, Result as fmtResult};
use std::str::FromStr;

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SignType {
    CMS,          // signed method for a CMS signed data
    Authenticode, // signed method for signing EFI image using authenticode spec
}

impl Display for SignType {
    fn fmt(&self, f: &mut Formatter) -> fmtResult {
        match self {
            SignType::CMS => write!(f, "cms"),
            SignType::Authenticode => write!(f, "authenticode"),
        }
    }
}

impl FromStr for SignType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "cms" => Ok(SignType::CMS),
            "authenticode" => Ok(SignType::Authenticode),
            _ => Err(Error::ParameterError("Invalid sign_type param".to_string())),
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FileType {
    RPM,
    CheckSum,
    KernelModule,
    EfiImage,
}

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter) -> fmtResult {
        match self {
            FileType::RPM => write!(f, "rpm"),
            FileType::CheckSum => write!(f, "checksum"),
            FileType::KernelModule => write!(f, "ko"),
            FileType::EfiImage => write!(f, "efi"),
        }
    }
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum KeyType {
    PGP,
    X509,
}

impl Display for KeyType {
    fn fmt(&self, f: &mut Formatter) -> fmtResult {
        match self {
            KeyType::PGP => write!(f, "pgp"),
            KeyType::X509 => write!(f, "x509"),
        }
    }
}
