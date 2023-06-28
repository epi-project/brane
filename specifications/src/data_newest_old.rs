//  DATA NEW.rs
//    by Lut99
// 
//  Created:
//    18 Jun 2023, 18:22:18
//  Last edited:
//    28 Jun 2023, 09:16:12
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines file structures for datasets.
// 

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::PathBuf;
use std::str::FromStr;

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};

use brane_shr::info::{JsonInfo, YamlInfo};
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





/***** AUXILLARY *****/
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





/***** LIBRARY *****/
/// Defines the `data.yml` file's layout.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataInfo {
    /// Anything common is also for internal usage, so we defer to the [`Datametadata`] struct.
    #[serde(flatten)]
    pub metadata : DataMetadata,
    /// The rest is kind-specific
    #[serde(alias = "implementation", alias = "contents")]
    pub layout   : DataSpecificInfo,
}
impl JsonInfo for DataInfo {}
impl YamlInfo for DataInfo {}


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


/// Defines what we need to know per package type.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSpecificInfo {
    /// It's a raw file.
    File(FileInfo),
    /// It's a raw directory.
    Directory(DirectoryInfo),
}
impl DataSpecificInfo {
    /// Returns an enum that can be used to represent the kind of this info.
    /// 
    /// # Returns
    /// A [`PackageKind`] that represents the kind of this info.
    #[inline]
    pub fn kind(&self) -> DataKind {
        use DataSpecificInfo::*;
        match self {
            File(_)      => DataKind::File,
            Directory(_) => DataKind::Directory,
        }
    }

    /// Returns if this DataSpecificInfo is a [`DataSpecificInfo::File`].
    /// 
    /// # Returns
    /// True if it is, or false otherwise.
    #[inline]
    pub fn is_file(&self) -> bool { matches!(self, Self::File(_)) }
    /// Provides quick immutable access to the internal info as if this was a [`DataSpecificInfo::File`].
    /// 
    /// # Returns
    /// A reference to the internal [`FileInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`DataSpecificInfo::File`].
    #[track_caller]
    #[inline]
    pub fn file(&self) -> &FileInfo { if let Self::File(file) = self { file } else { panic!("Cannot unwrap {:?} as a DataSpecificInfo::File", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`DataSpecificInfo::File`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`FileInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`DataSpecificInfo::File`].
    #[track_caller]
    #[inline]
    pub fn file_mut(&mut self) -> &mut FileInfo { if let Self::File(file) = self { file } else { panic!("Cannot unwrap {:?} as a DataSpecificInfo::File", self.variant()); } }
    /// Returns the internal info as if this was a [`DataSpecificInfo::File`].
    /// 
    /// # Returns
    /// The internal [`FileInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`DataSpecificInfo::File`].
    #[track_caller]
    #[inline]
    pub fn into_file(self) -> FileInfo { if let Self::File(file) = self { file } else { panic!("Cannot unwrap {:?} as a DataSpecificInfo::File", self.variant()); } }

    /// Returns if this DataSpecificInfo is a [`DataSpecificInfo::Directory`].
    /// 
    /// # Returns
    /// True if it is, or false otherwise.
    #[inline]
    pub fn is_directory(&self) -> bool { matches!(self, Self::Directory(_)) }
    /// Provides quick immutable access to the internal info as if this was a [`DataSpecificInfo::Directory`].
    /// 
    /// # Returns
    /// A reference to the internal [`DirectoryInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`DataSpecificInfo::Directory`].
    #[track_caller]
    #[inline]
    pub fn directory(&self) -> &DirectoryInfo { if let Self::Directory(dir) = self { dir } else { panic!("Cannot unwrap {:?} as a DataSpecificInfo::Directory", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`DataSpecificInfo::Directory`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`DirectoryInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`DataSpecificInfo::Directory`].
    #[track_caller]
    #[inline]
    pub fn directory_mut(&mut self) -> &mut DirectoryInfo { if let Self::Directory(dir) = self { dir } else { panic!("Cannot unwrap {:?} as a DataSpecificInfo::Directory", self.variant()); } }
    /// Returns the internal info as if this was a [`DataSpecificInfo::Directory`].
    /// 
    /// # Returns
    /// The internal [`DirectoryInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`DataSpecificInfo::Directory`].
    #[track_caller]
    #[inline]
    pub fn into_directory(self) -> DirectoryInfo { if let Self::Directory(dir) = self { dir } else { panic!("Cannot unwrap {:?} as a DataSpecificInfo::Directory", self.variant()); } }
}


/// Defines what we need to know for File datasets.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileInfo {
    /// Defines the path where we can find the file.
    pub path : PathBuf,
}

/// Defines what we need to know for Directory datasets.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DirectoryInfo {
    /// Defines the path where we can find the directory.
    pub path : PathBuf,
}
