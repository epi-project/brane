//  SERIALIZE.rs
//    by Lut99
// 
//  Created:
//    18 Jun 2023, 18:25:39
//  Last edited:
//    26 Jun 2023, 18:31:58
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines common structs that are handy for serializing/deserializing
//!   with serde.
// 

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::str::FromStr;

use enum_debug::EnumDebug;
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
#[derive(Clone, EnumDebug, Eq, Hash, PartialEq, Serialize)]
pub enum Identifier<'s> {
    /// Wraps an owned string.
    Owned(String),
    /// Wraps a borrowed string.
    Borrowed(&'s str),
}

impl<'s> Identifier<'s> {
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



    /// Returns the Identifier as a [`str`].
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Owned(s)    => &s,
            Self::Borrowed(s) => s,
        }
    }

    /// Returns the Identifier as a [`String`].
    #[inline]
    pub fn as_string(&self) -> String {
        match self {
            Self::Owned(s)    => s.clone(),
            Self::Borrowed(s) => s.into(),
        }
    }
    /// Returns the Identifier and consumes it into a [`String`].
    #[inline]
    pub fn into_string(self) -> String {
        match self {
            Self::Owned(s)    => s,
            Self::Borrowed(s) => s.into(),
        }
    }
}

impl<'s> Debug for Identifier<'s> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        match self {
            Self::Owned(s)    => write!(f, "Identifier({:?})", s),
            Self::Borrowed(s) => write!(f, "Identifier(&{:?})", s),
        }
    }
}
impl<'s> Display for Identifier<'s> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}", self.as_str())
    }
}
impl<'s> FromStr for Identifier<'s> {
    type Err = IdentifierParseError;

    fn from_str(s: &'s str) -> Result<Self, Self::Err> {
        // Assert it exists of only allowed characters
        if let Some(c) = Self::is_valid(s) {
            return Err(IdentifierParseError::IllegalChar { raw: s.into(), c });
        }

        // It's OK
        Ok(Self::Borrowed(s))
    }
}
impl<'de> Deserialize<'de> for Identifier<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// The Visitor for the [`Identifier`].
        struct IdentifierVisitor;
        impl<'de> Visitor<'de> for IdentifierVisitor {
            type Value = Identifier<'de>;

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

impl AsRef<Identifier> for Identifier {
    #[inline]
    fn as_ref(&self) -> &Self { self }
}
impl AsMut<Identifier> for Identifier {
    #[inline]
    fn as_mut(&self) -> &Self { self }
}
