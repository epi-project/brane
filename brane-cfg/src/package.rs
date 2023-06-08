//  PACKAGE.rs
//    by Lut99
// 
//  Created:
//    07 Jun 2023, 16:23:43
//  Last edited:
//    08 Jun 2023, 15:31:54
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the `package.yml` file layouts.
//!   
//!   This is just the user-facing version. For internally used counterparts,
//!   check the [`metadata`](crate::metadata) submodule.
// 

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Visitor};

use specifications::version::Version;

use crate::info::YamlInfo;


/***** ERRORS *****/
/// Defines the errors that may occur when parsing [`PackageKind`]s.
#[derive(Debug)]
pub enum PackageKindParseError {
    /// An unknown package kind was given.
    UnknownKind { raw: String },
}
impl Display for PackageKindParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use PackageKindParseError::*;
        match self {
            UnknownKind { raw } => writeln!(f, "Unknown package kind '{raw}'"),
        }
    }
}
impl Error for PackageKindParseError {}

/// Defines the errors that may occur when parsing [`Identifier`]s.
#[derive(Debug)]
pub enum IdentifierParseError {
    /// The identifier had an illegal character
    IllegalChar { raw: String, c: char },
}
impl Display for IdentifierParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use IdentifierParseError::*;
        match self {
            IllegalChar { raw, c } => writeln!(f, "Identifier '{raw}' cannot contain character '{c}', only alphanumerical characters and underscores"),
        }
    }
}
impl Error for IdentifierParseError {}





/***** HELPER FUNCTIONS *****/
/// Defines the default base image to use.
#[inline]
fn default_base_image() -> Image { Image::new("ubuntu", Some("22.04"), None::<String>) }

/// Defines the default package manager.
#[inline]
fn default_package_manager() -> PackageManager { PackageManager::Auto }





/***** FORMATTERS *****/
/// Serializes an Image to a way that Docker likes.
#[derive(Debug)]
pub struct ImageDockerFormatter<'a> {
    /// The image to format
    image : &'a Image,
}
impl<'a> Display for ImageDockerFormatter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}", if let Some(digest) = &self.image.digest { digest[7..].into() } else { format!("{}{}", self.image.name, if let Some(version) = &self.image.version { format!(":{version}") } else { String::new() }) })
    }
}





/***** AUXILLARY *****/
/// Enumerates the possible package kinds.
#[derive(Clone, Copy, Debug, EnumDebug, Eq, Hash, PartialEq)]
pub enum PackageKind {
    /// It's executable code
    Ecu,
    /// It's a BraneScript/Bakery package
    Dsl,
    /// It's a Common Workflow Language package.
    Cwl,
}

impl PackageKind {
    /// Returns whether this kind is an Executable Container Unit (ECU) or not.
    /// 
    /// # Returns
    /// True if it is, false if it isn't.
    pub fn is_ecu(&self) -> bool { matches!(self, Self::Ecu) }
    /// Returns whether this kind is a BraneScript/Bakery package or not.
    /// 
    /// # Returns
    /// True if it is, false if it isn't.
    pub fn is_dsl(&self) -> bool { matches!(self, Self::Dsl) }
    /// Returns whether this kind is an Common Workflow Language package (CWL) or not.
    /// 
    /// # Returns
    /// True if it is, false if it isn't.
    pub fn is_cwl(&self) -> bool { matches!(self, Self::Cwl) }
}

impl Display for PackageKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use PackageKind::*;
        match self {
            Ecu => write!(f, "Executable Container Unit"),
            Dsl => write!(f, "BraneScript/Bakery"),
            Cwl => write!(f, "Common Workflow Language"),
        }
    }
}
impl FromStr for PackageKind {
    type Err = PackageKindParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ecu" => Ok(Self::Ecu),
            "dsl" => Ok(Self::Dsl),
            "cwl" => Ok(Self::Cwl),
            s     => Err(PackageKindParseError::UnknownKind { raw: s.into() }),
        }
    }
}



/// Defines a name that only parses a few identifiers.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct Identifier(String);

impl Identifier {
    /// Helper function that checks if a string is valid according to the identifier.
    /// 
    /// # Returns
    /// [`None`] if it is, or [`Some`] and the character that was illegal.
    #[inline]
    fn is_valid(s: &str) -> Option<char> {
        for c in s.chars() {
            if (c < 'a' || c > 'z') && (c < 'A' || c > 'Z') && (c < '0' || c > '9') && c != '_' {
                return Some(c);
            }
        }
        None
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}", self.0)
    }
}
impl FromStr for Identifier {
    type Err = IdentifierParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Assert it exists of only allowed characters
        if let Some(c) = Self::is_valid(s) {
            return Err(IdentifierParseError::IllegalChar { raw: s.into(), c });
        }

        // It's OK
        Ok(Self(s.into()))
    }
}
impl<'de> Deserialize<'de> for Identifier {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// The Visitor for the [`Identifier`].
        struct IdentifierVisitor;
        impl<'de> Visitor<'de> for IdentifierVisitor {
            type Value = Identifier;

            fn expecting(&self, f: &mut Formatter) -> FResult {
                write!(f, "an identifier")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Simply forward to [`Self::from_str()`]
                match Identifier::from_str(v) {
                    Ok(value) => Ok(value),
                    Err(err)  => Err(E::custom(err)),
                }
            }
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Do it ourselves but more efficient

                // Assert it exists of only allowed characters
                if let Some(c) = Identifier::is_valid(&v) {
                    return Err(E::custom(IdentifierParseError::IllegalChar { raw: v, c }));
                }

                // It's OK
                Ok(Identifier(v))
            }
        }

        // Visit the visitor
        deserializer.deserialize_string(IdentifierVisitor)
    }
}

impl Deref for Identifier {
    type Target = String;

    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for Identifier {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}





/***** LIBRARY *****/
/// Defines the `package.yml` file's layout.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageInfo {
    /// The name/programming ID of this package.
    pub name        : Identifier,
    /// The version of this package.
    pub version     : Version,
    /// The list of owners of this package.
    pub owners      : Option<Vec<String>>,
    /// A short description of the package.
    pub description : Option<String>,

    /// The functions that this package supports.
    #[serde(alias = "actions")]
    pub functions : HashMap<String, Function>,
    /// The classes/types that this package adds.
    #[serde(alias = "types")]
    pub classes   : HashMap<String, Class>,

    /// The rest is kind-specific
    #[serde(alias = "implementation", alias = "layout")]
    pub contents : PackageSpecificInfo,
}
impl<'de> YamlInfo<'de> for PackageInfo {}


/// Defines what we need to know per package type.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageSpecificInfo {
    /// It's a container.
    Ecu(EcuInfo),
    /// It's a BraneScript/Bakery package.
    Dsl(DslInfo),
    /// It's a CWL package.
    Cwl(CwlInfo),
}
impl PackageSpecificInfo {
    /// Returns an enum that can be used to represent the kind of this info.
    /// 
    /// # Returns
    /// A [`PackageKind`] that represents the kind of this info.
    #[inline]
    pub fn kind(&self) -> PackageKind {
        use PackageSpecificInfo::*;
        match self {
            Ecu(_) => PackageKind::Ecu,
            Dsl(_) => PackageKind::Dsl,
            Cwl(_) => PackageKind::Cwl,
        }
    }

    /// Returns if this PackageSpecificInfo is a [`PackageSpecificInfo::Ecu`].
    /// 
    /// # Returns
    /// True if it is, or false otherwise.
    #[inline]
    pub fn is_ecu(&self) -> bool { matches!(self, Self::Ecu(_)) }
    /// Provides quick immutable access to the internal info as if this was a [`PackageSpecificInfo::Ecu`].
    /// 
    /// # Returns
    /// A reference to the internal [`EcuInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Ecu`].
    #[track_caller]
    #[inline]
    pub fn ecu(&self) -> &EcuInfo { if let Self::Ecu(ecu) = self { ecu } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Ecu", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`PackageSpecificInfo::Ecu`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`EcuInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Ecu`].
    #[track_caller]
    #[inline]
    pub fn ecu_mut(&mut self) -> &mut EcuInfo { if let Self::Ecu(ecu) = self { ecu } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Ecu", self.variant()); } }
    /// Returns the internal info as if this was a [`PackageSpecificInfo::Ecu`].
    /// 
    /// # Returns
    /// The internal [`EcuInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Ecu`].
    #[track_caller]
    #[inline]
    pub fn into_ecu(self) -> EcuInfo { if let Self::Ecu(ecu) = self { ecu } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Ecu", self.variant()); } }

    /// Returns if this PackageSpecificInfo is a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// True if it is, or false otherwise.
    #[inline]
    pub fn is_dsl(&self) -> bool { matches!(self, Self::Dsl(_)) }
    /// Provides quick immutable access to the internal info as if this was a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// A reference to the internal [`DslInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Dsl`].
    #[track_caller]
    #[inline]
    pub fn dsl(&self) -> &DslInfo { if let Self::Dsl(dsl) = self { dsl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Dsl", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`DslInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Dsl`].
    #[track_caller]
    #[inline]
    pub fn dsl_mut(&mut self) -> &mut DslInfo { if let Self::Dsl(dsl) = self { dsl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Dsl", self.variant()); } }
    /// Returns the internal info as if this was a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// The internal [`DslInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Dsl`].
    #[track_caller]
    #[inline]
    pub fn into_dsl(self) -> DslInfo { if let Self::Dsl(dsl) = self { dsl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Dsl", self.variant()); } }

    /// Returns if this PackageSpecificInfo is a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// True if it is, or false otherwise.
    #[inline]
    pub fn is_cwl(&self) -> bool { matches!(self, Self::Cwl(_)) }
    /// Provides quick immutable access to the internal info as if this was a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// A reference to the internal [`CwlInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Cwl`].
    #[track_caller]
    #[inline]
    pub fn cwl(&self) -> &CwlInfo { if let Self::Cwl(cwl) = self { cwl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Cwl", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`CwlInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Cwl`].
    #[track_caller]
    #[inline]
    pub fn cwl_mut(&mut self) -> &mut CwlInfo { if let Self::Cwl(cwl) = self { cwl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Cwl", self.variant()); } }
    /// Returns the internal info as if this was a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// The internal [`CwlInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Cwl`].
    #[track_caller]
    #[inline]
    pub fn into_cwl(self) -> CwlInfo { if let Self::Cwl(cwl) = self { cwl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Cwl", self.variant()); } }
}


/// Defines the layout of a function definition.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Function {
    /// An optional description of the function.
    #[serde(default = "String::new")]
    pub description : String,

    /// The inputs of this function.
    #[serde(alias = "params", alias = "parameters")]
    pub input  : Vec<Parameter>,
    /// The outputs of this function.
    #[serde(alias = "returns")]
    pub output : Vec<DataType>,
}

/// Defines a single data type that may be used as input.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Parameter {
    /// An optional description of the parameter.
    #[serde(default = "String::new")]
    pub description : String,

    /// The name of the value.
    pub name      : Identifier,
    /// The data type of the value.
    #[serde(alias = "type")]
    pub data_type : brane_ast::DataType,
}

/// Defines a single data type that may be used as output.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataType {
    /// An optional description of the parameter.
    #[serde(default = "String::new")]
    pub description : String,

    /// The data type of the value.
    #[serde(alias = "type")]
    pub data_type : brane_ast::DataType,
}


/// Defines the layout of a class definition.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Class {
    /// An optional description of the class.
    #[serde(default = "String::new")]
    pub description : String,

    /// The properties for this class, as a map of name to value.
    #[serde(alias = "fields")]
    pub props   : HashMap<String, DataType>,
    /// The functions defined in this class, as a map of name to definition.
    #[serde(alias = "functions", alias = "actions")]
    pub methods : HashMap<String, Function>,
}



/// Defines what we need to know for ECU packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EcuInfo {
    /// Defines the base image to use for the container
    #[serde(alias = "image", default = "default_base_image")]
    pub base            : Image,
    /// Defines the package manager to use.
    #[serde(alias = "dependency_resolver", default = "default_package_manager")]
    pub package_manager : PackageManager,

    /// Sets any environment variables while building this container.
    #[serde(alias = "build_args")]
    pub args  : HashMap<String, String>,
    /// Sets any environment variables while building this container _and_ beyond.
    #[serde(alias = "environment")]
    pub env   : HashMap<String, String>,
    /// Defines the steps to do in the container.
    #[serde(alias = "build")]
    pub steps : Vec<BuildStep>,

    /// Defines the command to run as entrypoint to the container.
    pub entrypoint : Vec<String>,
    /// Defines optional ecu-specific options for each function
    #[serde(alias = "actions")]
    pub functions  : HashMap<String, FunctionEcu>,
    /// Defines optional ecu-specific options for each class
    #[serde(alias = "types")]
    pub classes    : HashMap<String, ClassEcu>,
}


/// Specifies the name of an Image, possibly with digest.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// The name/label of the image.
    pub name    : String,
    /// The version/label of the image, if any.
    pub version : Option<String>,
    /// If we know a digest of the image, this field tells us it.
    pub digest  : Option<String>,
}

impl Image {
    /// Constructor for the Image that instantiates it with the given name.
    /// 
    /// # Arguments
    /// - `name`: The name/label of the image.
    /// - `digest`: The digest of the image if this is already known.
    /// 
    /// # Returns
    /// A new Image instance.
    #[inline]
    pub fn new(name: impl Into<String>, version: Option<impl Into<String>>, digest: Option<impl Into<String>>) -> Self {
        Self {
            name    : name.into(),
            version : version.map(|v| v.into()),
            digest  : digest.map(|d| d.into()),
        }
    }



    /// Returns the name-part of the Image (i.e., the name + version).
    #[inline]
    pub fn name(&self) -> String { format!("{}{}", self.name, if let Some(version) = &self.version { format!(":{version}") } else { String::new() }) }

    /// Returns the digest-part of the Image.
    #[inline]
    pub fn digest(&self) -> Option<&str> { self.digest.as_deref() }

    /// Returns the Docker-compatible serialization of this Image.
    /// 
    /// # Returns
    /// An ImageDockerFormatter which handles the formatting.
    #[inline]
    pub fn docker(&self) -> ImageDockerFormatter { ImageDockerFormatter{ image: self } }
}

impl Display for Image {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}{}{}", self.name, if let Some(version) = &self.version { format!(":{version}") } else { String::new() }, if let Some(digest) = &self.digest { format!("@{digest}") } else { String::new() })
    }
}
impl From<String> for Image {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}
impl From<&String> for Image {
    fn from(s: &String) -> Self {
        Self::from(s.as_str())
    }
}
impl From<&str> for Image {
    fn from(s: &str) -> Self {
        // First, split between digest and rest, if any '@' is present
        let (rest, digest): (&str, Option<&str>) = if let Some(idx) = s.rfind('@') {
            (&s[..idx], Some(&s[idx + 1..]))
        } else {
            (s, None)
        };

        // Next, search if there is a version or something
        let (name, version): (&str, Option<&str>) = if let Some(idx) = s.rfind(':') {
            (&rest[..idx], Some(&rest[idx + 1..]))
        } else {
            (rest, None)
        };

        // We can combine those in an Image
        Image {
            name    : name.into(),
            version : version.map(|s| s.into()),
            digest  : digest.map(|s| s.into()),
        }
    }
}
impl FromStr for Image {
    type Err = std::convert::Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl AsRef<Image> for Image {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}
impl AsMut<Image> for Image {
    #[inline]
    fn as_mut(&mut self) -> &mut Self { self }
}
impl From<&Image> for Image {
    #[inline]
    fn from(value: &Image) -> Self {
        value.clone()
    }
}
impl From<&mut Image> for Image {
    #[inline]
    fn from(value: &mut Image) -> Self {
        value.clone()
    }
}


/// Defines the supported package managers by BRANE.
#[derive(Clone, Debug, EnumDebug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PackageManager {
    // Meta options
    /// Attempts to automatically resolve the package manager based on the chosen image.
    Auto,
    /// Unrecognized package manager.
    Unrecognized,

    // Actual options
    /// Ubuntu's apt-get.
    #[serde(alias = "apt-get")]
    Apt,
    /// Alpine's apk
    Apk,
}
impl From<String> for PackageManager {
    #[inline]
    fn from(value: String) -> Self { Self::from(value.as_str()) }
}
impl From<&String> for PackageManager {
    #[inline]
    fn from(value: &String) -> Self { Self::from(value.as_str()) }
}
impl From<&mut String> for PackageManager {
    #[inline]
    fn from(value: &mut String) -> Self { Self::from(value.as_str()) }
}
impl From<&str> for PackageManager {
    #[inline]
    fn from(value: &str) -> Self {
        match value {
            "auto"         => Self::Auto,

            "apt" | "apt-get" => Self::Apt,
            "apk"             => Self::Apk,

            // The rest always resolves to unrecognized
            _ => Self::Unrecognized,
        }
    }
}
impl<'de> Deserialize<'de> for PackageManager {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// A visitor for the PackageManager.
        struct PackageManagerVisitor;
        impl<'de> Visitor<'de> for PackageManagerVisitor {
            type Value = PackageManager;

            #[inline]
            fn expecting(&self, f: &mut Formatter) -> FResult {
                write!(f, "a package manager identifier")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PackageManager::from(v))
            }
        }

        // Then simply visit
        deserializer.deserialize_str(PackageManagerVisitor)
    }
}
impl FromStr for PackageManager {
    type Err = std::convert::Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Self::from(s)) }
}


/// Defines a possible buildstep for the container.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildStep {
    /// Copy one or more files to the image
    #[serde(alias = "file", alias = "files")]
    Copy(CopyStep),
    /// Install something using the package manager
    #[serde(alias = "dependency", alias = "dependencies")]
    Install(InstallStep),
    /// Run an arbitrary command.
    #[serde(alias = "execute")]
    Run(RunStep),
}

/// Defines a copy step.
/// 
/// The only field for this struct is the list of files to copy in/out the container.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CopyStep(Vec<String>);

/// Defines a dependency step.
/// 
/// The only field for this struct is the list of packages to install.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InstallStep(Vec<String>);

/// Defines a run step.
/// 
/// The only field for this struct is the list of commands to run.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunStep(Vec<String>);


/// Defines the layout of a class definition.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FunctionEcu {
    /// How to capture the output of the function.
    pub capture : OutputCapture,
    /// Any additional environment variables to override or set for this function only.
    pub env     : HashMap<String, String>,
    /// Any command-line arguments to give for this function.
    pub command : Vec<String>,
}

/// Defines how the output of a function may be captured.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OutputCapture {
    /// Defines what to capture (stdout or stderr).
    #[serde(alias = "stream")]
    pub channel : CaptureChannel,
    /// Defines the method of capturing.
    #[serde(alias = "method")]
    pub kind    : CaptureKind,
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

/// Defines how to capture the input stream.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureKind {
    /// Captures the entire stream.
    #[serde(rename = "complete")]
    Full,
    /// Captures everything in between start/stop area.
    #[serde(rename = "marked")]
    Area,
    /// Captures everything prefixed by a special string (`~~> `).
    Prefixed,
}


/// Defines the layout of a class definition.
/// 
/// The only field in this struct defines a further map to specify additional properties per method in the class.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClassEcu(HashMap<String, FunctionEcu>);



/// Defines what we need to know for BraneScript/Bakery packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DslInfo {}



/// Defines what we need to know for CWL packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CwlInfo {}
