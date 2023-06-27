//  LOCATION.rs
//    by Lut99
// 
//  Created:
//    27 Jun 2023, 18:54:43
//  Last edited:
//    27 Jun 2023, 19:05:37
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines an abstraction over location Locations.
// 

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer, Visitor};


/***** ERRORS *****/
/// Defines the errors that may occur when parsing [`Location`]s.
#[derive(Debug)]
pub enum LocationParseError {
    /// The identifier had an illegal character
    IllegalChar { raw: String, c: char },
}
impl Display for LocationParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use LocationParseError::*;
        match self {
            IllegalChar { raw, c } => writeln!(f, "Location identifier '{raw}' cannot contain character '{c}', only alphanumerical characters and underscores"),
        }
    }
}
impl Error for LocationParseError {}





/***** LIBRARY *****/
/// Defines the location identifier, which is like a normal string except it's limited to alphanumeric characters and underscores.
#[derive(Clone, Eq, Hash, PartialEq, Serialize)]
pub struct Location(String);
impl Location {
    /// Helper function that checks if a string is valid according to the Location.
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

impl Debug for Location {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "Location(&{:?})", self.0)
    }
}
impl Display for Location {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "Location(&{})", self.0)
    }
}
impl FromStr for Location {
    type Err = LocationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = Self::is_valid(s) { return Err(LocationParseError::IllegalChar { raw: s.into(), c }); }
        Ok(Self(s.into()))
    }
}
impl<'de> Deserialize<'de> for Location {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        /// The Visitor for the [`Location`].
        struct LocationVisitor;
        impl<'de> Visitor<'de> for LocationVisitor {
            type Value = Location;

            fn expecting(&self, f: &mut Formatter) -> FResult {
                write!(f, "an Location")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Simply forward to [`Self::from_str()`]
                match Location::from_str(v) {
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
                if let Some(c) = Location::is_valid(&v) {
                    return Err(E::custom(LocationParseError::IllegalChar { raw: v, c }));
                }

                // It's OK
                Ok(Location(v))
            }
        }

        // Visit the visitor
        deserializer.deserialize_string(LocationVisitor)
    }
}

impl Deref for Location {
    type Target = String;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for Location {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl AsRef<str> for Location {
    #[inline]
    fn as_ref(&self) -> &str { &self.0 }
}
impl AsMut<str> for Location {
    #[inline]
    fn as_mut(&mut self) -> &mut str { &mut self.0 }
}
impl AsRef<String> for Location {
    #[inline]
    fn as_ref(&self) -> &String { &self.0 }
}
impl AsMut<String> for Location {
    #[inline]
    fn as_mut(&mut self) -> &mut String { &mut self.0 }
}
impl From<&str> for Location {
    #[inline]
    fn from(value: &str) -> Self { Self(value.into()) }
}
impl From<&mut str> for Location {
    #[inline]
    fn from(value: &mut str) -> Self { Self(value.into()) }
}
impl From<String> for Location {
    #[inline]
    fn from(value: String) -> Self { Self(value) }
}
impl From<&String> for Location {
    #[inline]
    fn from(value: &String) -> Self { Self(value.clone()) }
}
impl From<&mut String> for Location {
    #[inline]
    fn from(value: &mut String) -> Self { Self(value.clone()) }
}
impl From<Location> for String {
    #[inline]
    fn from(value: Location) -> Self { value.0 }
}
impl From<&Location> for String {
    #[inline]
    fn from(value: &Location) -> Self { value.0.clone() }
}
impl From<&mut Location> for String {
    #[inline]
    fn from(value: &mut Location) -> Self { value.0.clone() }
}
impl<'i> From<&'i Location> for &'i String {
    #[inline]
    fn from(value: &'i Location) -> Self { &value.0 }
}
impl<'i> From<&'i mut Location> for &'i String {
    #[inline]
    fn from(value: &'i mut Location) -> Self { &value.0 }
}
impl<'i> From<&'i mut Location> for &'i mut String {
    #[inline]
    fn from(value: &'i mut Location) -> Self { &mut value.0 }
}

impl AsRef<Location> for Location {
    #[inline]
    fn as_ref(&self) -> &Self { self }
}
impl AsMut<Location> for Location {
    #[inline]
    fn as_mut(&mut self) -> &mut Self { self }
}
