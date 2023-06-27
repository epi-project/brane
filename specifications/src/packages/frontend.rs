//  FRONTEND.rs
//    by Lut99
// 
//  Created:
//    20 Jun 2023, 17:12:20
//  Last edited:
//    27 Jun 2023, 18:56:28
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the variation of the package info that the user will see.
//!   Contains information on how to build the image, as well as how to
//!   run it.
// 

use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FResult};
use std::str::FromStr;

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Visitor};

use brane_shr::info::YamlInfo;
use brane_shr::identifier::Identifier;

use super::common::{Class, Function, PackageKind, PackageMetadata};
use super::{backend, internal};


/***** TESTS *****/
#[cfg(test)]
mod tests {
    use brane_shr::errors::ErrorTrace as _;
    use brane_shr::info::Info as _;
    use super::*;

    #[test]
    fn test_package_info_frontend_hello_world() {
        // Define the simple hello world string
        let container_yml: &str = r#"
name: hello_world
version: 1.0.0
kind: ecu

functions:
  hello_world:
    run:
    - hello_world.sh
    output:
      message:
        type: string

build:
- copy:
  - hello_world.sh
"#;

        // Attempt to parse that without errors
        let info: PackageInfo = match PackageInfo::from_string(container_yml) {
            Ok(info) => info,
            Err(err) => { panic!("Failed to parse input YAML: {}", err.trace()); },
        };
        println!("\n{info:#?}");
    }

    #[test]
    fn test_package_info_frontend_base64() {
        // Define the simple hello world string
        let container_yml: &str = r#"
name: base64
version: 1.0.0
kind: ecu

functions:
  encode:
    run:
    - python3
    - code.py
    args:
    - encode
    input:
    - name: input
      type: string
    output:
      output:
        type: string
  decode:
    run:
    - python3
    - code.py
    args:
    - decode
    input:
    - name: input
      type: string
    output:
      output:
        type: string

entrypoint:
  file: code.py

build:
- dependencies:
  - python3
  - python3-yaml

- copy:
  - code.py
"#;

        // Attempt to parse that without errors
        let info: PackageInfo = match PackageInfo::from_string(container_yml) {
            Ok(info) => info,
            Err(err) => { panic!("Failed to parse input YAML: {}", err.trace()); },
        };
        println!("\n{info:#?}");
    }

    #[test]
    fn test_package_info_frontend_epi() {
        // Define the simple hello world string
        let container_yml: &str = r#"
name: epi
version: 1.0.0
kind: ecu
owners:
- Rosanne Turner
description: |
  Package used for implementing the EPI Proof-of-Concept (PoC). This PoC
  runs a local compute on two partitions of the same data that live at
  different hospitals. Then, at the location of a trusted third-party, a global
  step is executed.



build:
# Install the base dependencies in apt
- dependencies:
  - build-essential
  - dirmngr
  - gnupg
  - apt-transport-https
  - ca-certificates
  - software-properties-common
  - libxml2-dev
  - libssl-dev
  - libcurl4-openssl-dev
  - pkg-config

# Install R and the required packages
- run:
  - |
    apt-key adv --keyserver keyserver.ubuntu.com --recv-keys E298A3A825C0D65DFD57CBB651716619E084DAB9 && \
    add-apt-repository 'deb https://cloud.r-project.org/bin/linux/ubuntu focal-cran40/' && \
    apt install -y r-base && \
    rm -rf /var/lib/apt/lists/*
  - mkdir -p /Rlibs && mkdir -p /opt/wd && echo ".libPaths(\"/Rlibs\")" > /opt/wd/.Rprofile
  - echo "install.packages(\"tidyverse\", lib=\"/Rlibs\")" | R --save

- copy:
  - run.sh
  - central code stratified confidence sequence.R
  - local code stratified confidence sequence.R
  - stratifiedConfidenceFunctions POC.R
  - POC helper functions.R



functions:
  local_scs:
    run:
    - run.sh
    args:
    - local_scs
    input:
    - name: input
      type: Data
    output:
      output:
        type: IntermediateResult
  central_scs:
    run:
    - run.sh
    args:
    - central_scs
    input:
    - name: inputs
      type: IntermediateResult[]
    output:
      output:
        type: IntermediateResult
    capture: marked
"#;

        // Attempt to parse that without errors
        let info: PackageInfo = match PackageInfo::from_string(container_yml) {
            Ok(info) => info,
            Err(err) => { panic!("Failed to parse input YAML: {}", err.trace()); },
        };
        println!("\n{info:#?}");
    }
}





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





/***** LIBRARY *****/
/// Defines the `package.yml` file's layout.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageInfo {
    /// What we know about this package that is implementation-agnostic (e.g., name, version, ...)
    #[serde(flatten)]
    pub metadata       : PackageMetadata,
    /// This defines everything implementation-specific about the package.
    #[serde(flatten)]
    pub implementation : PackageBuildInfo,
}
impl YamlInfo for PackageInfo {}



/// Defines what we need to know per package type public-facing.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackageBuildInfo {
    /// It's a container.
    Ecu(EcuInfo),
    /// It's a BraneScript/Bakery package.
    Dsl(DslInfo),
    /// It's a CWL package.
    Cwl(CwlInfo),
}
impl PackageBuildInfo {
    /// Returns an enum that can be used to represent the kind of this info.
    /// 
    /// # Returns
    /// A [`PackageKind`] that represents the kind of this info.
    #[inline]
    pub fn kind(&self) -> PackageKind {
        use PackageBuildInfo::*;
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



/// Defines what we need to know for ECU packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EcuInfo {
    /// Defines the base image to use for the container
    #[serde(alias = "image", default = "Image::info_default")]
    pub base            : Image,
    /// Defines the package manager to use.
    #[serde(alias = "dependency_resolver", default = "PackageManager::info_default")]
    pub package_manager : PackageManager,

    /// Sets any environment variables while building this container.
    #[serde(alias = "build_args", default = "HashMap::new")]
    pub args  : HashMap<String, String>,
    /// Sets any environment variables while building this container _and_ beyond.
    #[serde(alias = "environment", default = "HashMap::new")]
    pub env   : HashMap<String, String>,
    /// Defines the steps to do in the container.
    #[serde(alias = "build", default = "Vec::new")]
    pub steps : Vec<BuildStep>,

    /// Defines optional ecu-specific options for each function
    #[serde(alias = "actions", default = "HashMap::new")]
    pub functions  : HashMap<Identifier, Function<FunctionEcu>>,
    /// Defines optional ecu-specific options for each class
    #[serde(alias = "types", default = "HashMap::new")]
    pub classes    : HashMap<Identifier, Class<FunctionEcu>>,
}


/// Defines the implementation of a Function for ECU packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FunctionEcu {
    /// Defines the part necessary for launching the container.
    #[serde(flatten)]
    pub backend  : backend::FunctionEcu,
    /// Defines the part necessary for branelet.
    #[serde(flatten)]
    pub internal : internal::FunctionEcu,
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
    /// Returns the default image used in the [`PackageInfo`].
    /// 
    /// # Returns
    /// A new instance of Self used in the package info.
    #[inline]
    fn info_default() -> Self {
        Self {
            name    : "ubuntu".into(),
            version : Some("22.04".into()),
            digest  : None,
        }
    }

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
#[derive(Clone, Copy, Debug, EnumDebug, Serialize)]
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
impl PackageManager {
    /// Returns the default package manager used in the [`PackageInfo`].
    /// 
    /// # Returns
    /// A new instance of Self used in the package info.
    #[inline]
    fn info_default() -> Self { Self::Auto }
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



/// Defines what we need to know for BraneScript/Bakery packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DslInfo {}



/// Defines what we need to know for CWL packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CwlInfo {}
