//  METADATA.rs
//    by Lut99
// 
//  Created:
//    20 Jun 2023, 17:11:06
//  Last edited:
//    20 Jun 2023, 17:17:23
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the metadata that we want/need to know of a package. This is
//!   shared in both the front-facing version of the package info, as well
//!   as the backend.
// 

use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FResult};
use std::str::FromStr;

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Visitor};

use brane_shr::serialize::Identifier;
use brane_shr::version::Version;


/***** LIBRARY *****/
/// Defines what we need to know for the backend only.
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

    /// The functions that this package supports.
    #[serde(alias = "actions")]
    pub functions : HashMap<String, Function>,
    /// The classes/types that this package adds.
    #[serde(alias = "types")]
    pub classes   : HashMap<String, Class>,
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
    /// The outputs of this function, as a map of key name to type.
    #[serde(alias = "returns")]
    pub output : HashMap<Identifier, DataType>,
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
