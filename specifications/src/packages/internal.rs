//  INTERNAL.rs
//    by Lut99
// 
//  Created:
//    21 Jun 2023, 12:05:15
//  Last edited:
//    21 Jun 2023, 12:34:05
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
use brane_shr::serialize::Identifier;
use brane_shr::version::Version;

use super::common::{Class, Function, PackageKind, PackageMetadata};


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
    pub functions  : HashMap<Identifier, Function<FunctionEcu>>,
    /// Defines the functions, each of which define kind-specific implementation details to launch the package. Note, though, that branelet only works for ECU-packages, so no choice there.
    pub classes    : HashMap<Identifier, Class<FunctionEcu>>,
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
    pub fn new<S: Into<String>>(name: impl Into<Identifier>, version: impl Into<Version>, owners: Option<impl IntoIterator<Item = S>>, description: Option<impl Into<String>>, kind: PackageKind, functions: HashMap<Identifier, Function<FunctionEcu>>, classes: HashMap<Identifier, Class<FunctionEcu>>) -> Self {
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
    #[serde(default = "OutputCapture::info_default")]
    pub capture      : OutputCapture,
}

/// Defines how the output of a function may be captured.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OutputCapture {
    /// Defines what to capture (stdout or stderr).
    #[serde(alias = "stream", default = "CaptureChannel::info_default")]
    pub channel : CaptureChannel,
    /// Defines the method of capturing.
    #[serde(alias = "method", alias = "kind", default = "CaptureMode::info_default")]
    pub mode    : CaptureMode,
}
impl OutputCapture {
    /// Returns the default capture settings used in the [`PackageInfo`].
    /// 
    /// # Returns
    /// A new instance of Self used in the package info.
    #[inline]
    pub(super) fn info_default() -> Self {
        Self {
            channel : CaptureChannel::info_default(),
            mode    : CaptureMode::info_default(),
        }
    }
}

/// Defines what to capture from a container.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureChannel {
    /// Capture nothing
    Nothing,
    /// Capture stdout only
    Stdout,
    /// Capture stderr only
    Stderr,
    /// Capture both
    Both,
}
impl CaptureChannel {
    /// Returns the default capture channel used in the [`PackageInfo`].
    /// 
    /// # Returns
    /// A new instance of Self used in the package info.
    #[inline]
    pub(super) fn info_default() -> Self { Self::Stdout }
}

/// Defines how to capture the input stream.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureMode {
    /// Captures the entire stream.
    #[serde(rename = "complete")]
    Full,
    /// Captures everything in between start/stop area.
    #[serde(rename = "marked")]
    Area,
    /// Captures everything prefixed by a special string (`~~> `).
    Prefixed,
}
impl CaptureMode {
    /// Returns the default capture mode used in the [`PackageInfo`].
    /// 
    /// # Returns
    /// A new instance of Self used in the package info.
    #[inline]
    pub(super) fn info_default() -> Self { Self::Full }
}
