//  COMMON.rs
//    by Lut99
// 
//  Created:
//    21 Jun 2023, 10:08:46
//  Last edited:
//    26 Jun 2023, 12:18:50
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines parts of the file specification re-used across the
//!   frontend/backend.
// 

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::str::FromStr;

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Visitor};

use brane_shr::serialize::Identifier;
use brane_shr::version::Version;


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





/***** HELPER FUNCTIONS *****/
/// Returns false.
#[inline]
fn default_optional() -> bool { false }





/***** LIBRARY *****/
/// Enumerates the possible package kinds.
#[derive(Clone, Copy, Debug, Deserialize, EnumDebug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
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



/// Defines what we know about a package that is implementation-agnostic.
/// 
/// Note that this is not really true. We typically also know about [`Function`]s and [`Class`]es.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageMetadata {
    /// The name/programming ID of this package.
    pub name        : Identifier,
    /// The version of this package.
    pub version     : Version,
    /// The list of owners of this package.
    pub owners      : Option<Vec<String>>,
    /// A short description of the package.
    pub description : Option<String>,
}



/// Defines a function's metadata.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Function<I> {
    /// An optional description of the function.
    #[serde(default = "String::new")]
    pub description : String,

    /// The inputs of this function.
    #[serde(alias = "params", alias = "parameters", default = "Vec::new")]
    pub input  : Vec<Parameter>,
    /// The outputs of this function, as a map of key name to type.
    #[serde(alias = "returns", default = "HashMap::new")]
    pub output : HashMap<Identifier, DataType>,

    /// The remainder -the implementation- is left to the generic parameter.
    #[serde(flatten)]
    pub implementation : I,
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
    pub data_type : DataTypeKind,
    /// Is it optional?
    #[serde(default = "default_optional")]
    pub optional  : bool,
}


/// Defines a single data type that may be used as output.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataType {
    /// An optional description of the parameter.
    #[serde(default = "String::new")]
    pub description : String,

    /// The actual data type of the value.
    #[serde(alias = "type")]
    pub data_type : DataTypeKind,
}

/// Defines the possible data types we can parse.
#[derive(Clone, Debug, EnumDebug)]
pub enum DataTypeKind {
    // Atomic types
    /// Boolean values (true | false)
    Boolean,
    /// Integral types (0, 42, -42, ...)
    Integer,
    /// Real/Floating-point types (0.0, -0.0, 42.0, -42.0, 0.42, -0.42, ...)
    Real,
    /// String types ("Hello there!", "A", ...)
    String,

    // Builtin composite types
    /// Defines references to datasets.
    Data,
    /// Defines references to intermediate results.
    IntermediateResult,

    // Composites
    /// Defines an homogeneous array of types.
    Array { elem_type : Box<Self> },
    /// Defines a heterogeneous map of types.
    Class { name: String },
}
impl Display for DataTypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use DataTypeKind::*;
        match self {
            Boolean => write!(f, "bool"),
            Integer => write!(f, "int"),
            Real    => write!(f, "real"),
            String  => write!(f, "string"),

            Data               => write!(f, "Data"),
            IntermediateResult => write!(f, "IntermediateResult"),

            Array { elem_type } => write!(f, "{elem_type}[]"),
            Class { name }      => write!(f, "{name}"),
        }
    }
}
impl Serialize for DataTypeKind {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for DataTypeKind {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// Implements a visitor for the [`DataTypeKind`].
        struct DataTypeKindVisitor;
        impl<'de> Visitor<'de> for DataTypeKindVisitor {
            type Value = DataTypeKind;

            fn expecting(&self, f: &mut Formatter) -> FResult {
                write!(f, "a data type")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Simply call [`DataTypeKind::from_str()`]
                // SAFETY: Unwrap is OK because DataTypeKind's FromStr::Err is infallible (it can never error)
                Ok(DataTypeKind::from_str(v).unwrap())
            }
        }

        // Call the visitor
        deserializer.deserialize_str(DataTypeKindVisitor)
    }
}
impl From<String> for DataTypeKind {
    #[inline]
    fn from(value: String) -> Self {
        // Use the string-one
        Self::from(value.as_str())
    }
}
impl From<&String> for DataTypeKind {
    #[inline]
    fn from(value: &String) -> Self {
        // Use the string-one
        Self::from(value.as_str())
    }
}
impl From<&mut String> for DataTypeKind {
    #[inline]
    fn from(value: &mut String) -> Self { Self::from(value.as_str()) }
}
impl From<&str> for DataTypeKind {
    fn from(value: &str) -> Self {
        // First: any arrays are done recursively
        if !value.is_empty() && &value[..1] == "[" && &value[value.len() - 1..] == "]" {
            return Self::Array{ elem_type: Box::new(Self::from(&value[1..value.len() - 1])) };
        } else if value.len() >= 2 && &value[value.len() - 2..] == "[]" {
            return Self::Array{ elem_type: Box::new(Self::from(&value[..value.len() - 2])) };
        }

        // Otherwise, match literals & classes
        use DataTypeKind::*;
        match value {
            // Literal types
            "bool" | "boolean" => Boolean,
            "int"  | "integer" => Integer,
            "float" | "real"   => Real,
            "str" | "string"   => String,

            // Builtin classes
            "Data"               => Data,
            "IntermediateResult" => IntermediateResult,

            // The rest is always a class
            value => Class { name: value.into() },
        }
    }
}
impl FromStr for DataTypeKind {
    type Err = std::convert::Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> { Ok(Self::from(s)) }
}



/// Defines a custom class within a package.
/// 
/// # Generic parameters
/// - `I`: Some serializable struct that describes the layout of the part that describes how to implement a function (e.g., CLI-arguments to pass, environment variabels to set, etc).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Class<I> {
    /// An optional description of the class.
    #[serde(default = "String::new")]
    pub description : String,

    /// The properties for this class, as a map of name to value.
    #[serde(alias = "fields", default = "HashMap::new")]
    pub props   : HashMap<Identifier, DataType>,
    /// The functions defined in this class, as a map of name to definition.
    #[serde(alias = "functions", alias = "actions", default = "HashMap::new")]
    pub methods : HashMap<Identifier, Function<I>>,
}
