//  BACKEND.rs
//    by Lut99
// 
//  Created:
//    28 Jun 2023, 09:15:24
//  Last edited:
//    28 Jun 2023, 09:20:56
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the variation of the data info that the system uses once the
//!   dataset is built.
// 

use std::collections::HashSet;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};

use brane_shr::info::JsonInfo;
use brane_shr::location::Location;

use super::common::{DataKind, DataMetadata};


/***** LIBRARY *****/
/// Defines the `data.yml` file's layout used by the system itself.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataInfo {
    /// Anything common is also for internal usage, so we defer to the [`DataMetadata`] struct.
    #[serde(flatten)]
    pub metadata  : DataMetadata,
    /// Defines when this package was created.
    pub created   : DateTime<Utc>,
    /// Defines on which locations the package may be found.
    #[serde(default = "HashSet::new")]
    pub locations : HashSet<Location>,

    /// The rest is kind-specific
    #[serde(alias = "implementation", alias = "contents")]
    pub layout : DataSpecificInfo,
}
impl JsonInfo for DataInfo {}



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
