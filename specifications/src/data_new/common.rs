//  COMMON.rs
//    by Lut99
// 
//  Created:
//    28 Jun 2023, 09:15:57
//  Last edited:
//    28 Jun 2023, 09:18:32
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines part of the data info that is common to both frontend and
//!   backend variations.
// 

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::str::FromStr;

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};

use brane_shr::identifier::Identifier;
use brane_shr::version::Version;


/***** ERRORS *****/
/// Defines the errors that may occur when parsing [`DataKind`]s.
#[derive(Debug)]
pub enum DataKindParseError {
    /// An unknown data kind was given.
    UnknownKind { raw: String },
}
impl Display for DataKindParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use DataKindParseError::*;
        match self {
            UnknownKind { raw } => writeln!(f, "Unknown data kind '{raw}'"),
        }
    }
}
impl Error for DataKindParseError {}





/***** LIBRARY *****/
/// Enumerates the possible package kinds.
#[derive(Clone, Copy, Debug, EnumDebug, Eq, Hash, PartialEq)]
pub enum DataKind {
    /// The data refers to a file.
    File,
    /// The data refers to a directory of files/nested directories.
    Directory,
}

impl DataKind {
    /// Returns whether this kind is a file or not.
    /// 
    /// # Returns
    /// True if it is, false if it isn't.
    pub fn is_file(&self) -> bool { matches!(self, Self::File) }
    /// Returns whether this kind is a directory or not.
    /// 
    /// # Returns
    /// True if it is, false if it isn't.
    pub fn is_directory(&self) -> bool { matches!(self, Self::Directory) }
}

impl Display for DataKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use DataKind::*;
        match self {
            File      => write!(f, "File"),
            Directory => write!(f, "Directory"),
        }
    }
}
impl FromStr for DataKind {
    type Err = DataKindParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file"              => Ok(Self::File),
            "dir" | "directory" => Ok(Self::Directory),
            s                   => Err(DataKindParseError::UnknownKind { raw: s.into() }),
        }
    }
}



/// Defines what we need to know for the backend only.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataMetadata {
    /// The name/programming ID of this package.
    pub name        : Identifier,
    /// The version of this package.
    pub version     : Version,
    /// The list of owners of this package.
    pub owners      : Option<Vec<String>>,
    /// A short description of the package.
    pub description : Option<String>,
}
