//  SERIALIZE.rs
//    by Lut99
// 
//  Created:
//    18 Jun 2023, 18:25:39
//  Last edited:
//    21 Jun 2023, 16:58:11
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines common structs that are handy for serializing/deserializing
//!   with serde.
// 

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Visitor};


/***** ERRORS *****/
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





/***** LIBRARY *****/
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

impl From<&str> for Identifier {
    #[inline]
    fn from(value: &str) -> Self { Self(value.to_string()) }
}
impl From<&String> for Identifier {
    #[inline]
    fn from(value: &String) -> Self { Self(value.clone()) }
}
impl From<&mut String> for Identifier {
    #[inline]
    fn from(value: &mut String) -> Self { Self(value.clone()) }
}
impl From<String> for Identifier {
    #[inline]
    fn from(value: String) -> Self { Self(value) }
}
impl From<Identifier> for String {
    #[inline]
    fn from(value: Identifier) -> Self { value.0 }
}
impl From<&Identifier> for String {
    #[inline]
    fn from(value: &Identifier) -> Self { value.0.clone() }
}
impl From<&mut Identifier> for String {
    #[inline]
    fn from(value: &mut Identifier) -> Self { Self::from(&*value) }
}
