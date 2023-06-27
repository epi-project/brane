//  INTERNAL.rs
//    by Lut99
// 
//  Created:
//    21 Jun 2023, 12:05:15
//  Last edited:
//    27 Jun 2023, 18:56:32
//  Auto updated?
//    Yes
// 
//  Description:
//!   Everything `branelet` needs to know about the package.
// 

use std::collections::HashMap;

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};

use brane_shr::info::JsonInfo;
use brane_shr::identifier::Identifier;
use brane_shr::version::Version;

use super::common::{self, PackageKind, PackageMetadata};


/***** TYPES *****/
/// Re-export of the Function since there is only one flavour anyway in this cast.
pub type Function = common::Function<FunctionEcu>;
/// Re-export of the Class since there is only one flavour anyway in this cast.
pub type Class = common::Class<FunctionEcu>;





/***** HELPER FUNCTIONS *****/
/// Returns the default entrypoint argument string.
#[inline]
fn default_entrypoint() -> Vec<String> { vec![ "/bin/bash".into(), "-c".into() ] }





/***** LIBRARY *****/
/// Defines all the package information that `branelet` needs.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageInfo {
    /// What we know about this package that is implementation-agnostic (e.g., name, version, ...)
    #[serde(flatten)]
    pub metadata : PackageMetadata,
    /// The kind of this package.
    pub kind     : PackageKind,

    /// Defines the functions, each of which define kind-specific implementation details to launch the package. Note, though, that branelet only works for ECU-packages, so no choice there.
    pub functions  : HashMap<Identifier, Function>,
    /// Defines the functions, each of which define kind-specific implementation details to launch the package. Note, though, that branelet only works for ECU-packages, so no choice there.
    pub classes    : HashMap<Identifier, Class>,
}
impl PackageInfo {
    /// Constructor for the PackageInfo.
    /// 
    /// # Arguments
    /// - `name`: The name of the package.
    /// - `version`: The version of the package.
    /// - `owners`: The list of owners of this package, if any.
    /// - `description`: The description of this package, if any.
    /// - `kind`: The kind of this package.
    /// - `functions`: The list of functions in this package.
    /// - `classes`: The list of classes in this package.
    /// 
    /// # Returns
    /// A new instance of Self.
    #[inline]
    pub fn new<S: Into<String>>(name: impl Into<Identifier>, version: impl Into<Version>, owners: Option<impl IntoIterator<Item = S>>, description: Option<impl Into<String>>, kind: PackageKind, functions: HashMap<Identifier, Function>, classes: HashMap<Identifier, Class>) -> Self {
        Self {
            metadata : PackageMetadata {
                name        : name.into(),
                version     : version.into(),
                owners      : owners.map(|o| o.into_iter().map(|s| s.into()).collect()),
                description : description.map(|d| d.into()),
            },
            kind,

            functions,
            classes,
        }
    }
}
impl JsonInfo for PackageInfo {}


/// Defines the implementation of a Function for ECU packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FunctionEcu {
    /// Any additional environment variables to override or set for this function only.
    #[serde(alias = "environment", default = "HashMap::new")]
    pub env          : HashMap<String, String>,
    /// Defines how to run this function, as a command.
    #[serde(alias = "command", alias = "cmd", alias = "run", default = "default_entrypoint")]
    pub entrypoint   : Vec<String>,
    /// Defines additional arguments to pass to the entrypoint.
    #[serde(default = "Vec::new")]
    pub args         : Vec<String>,
    /// How to capture the output of the function.
    #[serde(default = "CaptureMode::info_default")]
    pub capture      : CaptureMode,
}

/// Defines how to capture the input stream.
#[derive(Clone, Copy, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMode {
    /// Captures the entire stream.
    #[serde(alias = "complete")]
    Full,
    /// Captures everything in between start/stop area.
    #[serde(alias = "marked")]
    Area,
    /// Captures everything prefixed by a special string (`~~> `).
    Prefixed,
    /// Captures... nothing!
    #[serde(alias = "none")]
    Nothing,
}
impl CaptureMode {
    /// Returns the default capture mode used in the [`PackageInfo`].
    /// 
    /// # Returns
    /// A new instance of Self used in the package info.
    #[inline]
    pub(super) fn info_default() -> Self { Self::Full }
}
