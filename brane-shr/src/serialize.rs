//  SERIALIZE.rs
//    by Lut99
// 
//  Created:
//    18 Jun 2023, 18:25:39
//  Last edited:
//    27 Jun 2023, 16:31:19
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines common structs that are handy for serializing/deserializing
//!   with serde.
// 

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
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
// /// Defines a reference to a [`str`] that only contains alphanumerical characters and an underscore.
// #[derive(Clone, Copy, Eq, Hash, PartialEq)]
// pub struct IdentifierRef<'s>(&'s str);
// impl<'s> IdentifierRef<'s> {
//     /// Copies the underlying string into an owned [`Identifier`].
//     #[inline]
//     pub fn clone_id(&self) -> Identifier { Identifier(self.0.into()) }
// }

// impl<'s> Debug for IdentifierRef<'s> {
//     #[inline]
//     fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
//         write!(f, "Identifier(&{:?})", self.0)
//     }
// }
// impl<'s> Display for IdentifierRef<'s> {
//     #[inline]
//     fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
//         write!(f, "Identifier(&{})", self.0)
//     }
// }

// impl<'s> Deref for IdentifierRef<'s> {
//     type Target = str;

//     #[inline]
//     fn deref(&self) -> &Self::Target { self.0 }
// }

// impl<'s> AsRef<str> for IdentifierRef<'s> {
//     #[inline]
//     fn as_ref(&self) -> &str { self.0 }
// }
// impl<'s> From<&'s str> for IdentifierRef<'s> {
//     #[inline]
//     fn from(value: &'s str) -> Self { Self(value) }
// }
// impl<'s> From<&'s mut str> for IdentifierRef<'s> {
//     #[inline]
//     fn from(value: &'s mut str) -> Self { Self(value) }
// }
// impl<'s> From<&'s String> for IdentifierRef<'s> {
//     #[inline]
//     fn from(value: &'s String) -> Self { Self(value) }
// }
// impl<'s> From<&'s mut String> for IdentifierRef<'s> {
//     #[inline]
//     fn from(value: &'s mut String) -> Self { Self(value) }
// }

// impl<'s> AsRef<IdentifierRef<'s>> for IdentifierRef<'s> {
//     #[inline]
//     fn as_ref(&self) -> &Self { self }
// }
// impl<'s> From<&IdentifierRef<'s>> for IdentifierRef<'s> {
//     #[inline]
//     fn from(value: &IdentifierRef<'s>) -> Self { *value }
// }
// impl<'s> From<&mut IdentifierRef<'s>> for IdentifierRef<'s> {
//     #[inline]
//     fn from(value: &mut IdentifierRef<'s>) -> Self { *value }
// }
// impl<'s> From<IdentifierMut<'s>> for IdentifierRef<'s> {
//     #[inline]
//     fn from(value: IdentifierMut<'s>) -> Self { IdentifierRef(value.0) }
// }
// impl<'s> From<&'s Identifier> for IdentifierRef<'s> {
//     #[inline]
//     fn from(value: &'s Identifier) -> Self { IdentifierRef(&value.0) }
// }
// impl<'s> From<&'s mut Identifier> for IdentifierRef<'s> {
//     #[inline]
//     fn from(value: &'s mut Identifier) -> Self { IdentifierRef(&value.0) }
// }



// /// Defines a mutable reference to a [`String`] that only contains alphanumerical characters and an underscore.
// #[derive(Eq, Hash, PartialEq)]
// pub struct IdentifierMut<'s>(&'s mut String);
// impl<'s> IdentifierMut<'s> {
//     /// Consumes the underlying string into an owned [`Identifier`], leaving a [`String::new()`] in its place.
//     #[inline]
//     pub fn take_id(&mut self) -> Identifier { Identifier(mem::take(self.0)) }
//     /// Copies the underlying string into an owned [`Identifier`].
//     #[inline]
//     pub fn clone_id(&self) -> Identifier { Identifier(self.0.clone()) }
// }

// impl<'s> Debug for IdentifierMut<'s> {
//     #[inline]
//     fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
//         write!(f, "Identifier(&{:?})", self.0)
//     }
// }
// impl<'s> Display for IdentifierMut<'s> {
//     #[inline]
//     fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
//         write!(f, "Identifier(&{})", self.0)
//     }
// }

// impl<'s> Deref for IdentifierMut<'s> {
//     type Target = String;

//     #[inline]
//     fn deref(&self) -> &Self::Target { self.0 }
// }
// impl<'s> DerefMut for IdentifierMut<'s> {
//     #[inline]
//     fn deref_mut(&mut self) -> &mut Self::Target { self.0 }
// }

// impl<'s> AsRef<str> for IdentifierMut<'s> {
//     #[inline]
//     fn as_ref(&self) -> &str { self.0 }
// }
// impl<'s> AsMut<str> for IdentifierMut<'s> {
//     #[inline]
//     fn as_mut(&mut self) -> &mut str { self.0 }
// }
// impl<'s> AsRef<String> for IdentifierMut<'s> {
//     #[inline]
//     fn as_ref(&self) -> &String { self.0 }
// }
// impl<'s> AsMut<String> for IdentifierMut<'s> {
//     #[inline]
//     fn as_mut(&mut self) -> &mut String { self.0 }
// }
// impl<'s> From<&'s mut String> for IdentifierMut<'s> {
//     #[inline]
//     fn from(value: &'s mut String) -> Self { Self(value) }
// }

// impl<'s> AsRef<IdentifierMut<'s>> for IdentifierMut<'s> {
//     #[inline]
//     fn as_ref(&self) -> &Self { self }
// }
// impl<'s> From<&'s mut Identifier> for IdentifierMut<'s> {
//     #[inline]
//     fn from(value: &'s mut Identifier) -> Self { IdentifierMut(&mut value.0) }
// }



/// Defines an owned [`String`] that only contains alphanumerical characters and an underscore.
#[derive(Clone, Eq, Hash, PartialEq, Serialize)]
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



    // /// Get the identifier as an [`IdentifierRef`].
    // pub fn as_id_ref(&self) -> IdentifierRef { IdentifierRef(&self.0) }
    // /// Get the identifier as an [`IdentifierMut`].
    // pub fn as_id_mut(&mut self) -> IdentifierMut { IdentifierMut(&mut self.0) }
}

impl Debug for Identifier {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "Identifier(&{:?})", self.0)
    }
}
impl Display for Identifier {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "Identifier(&{})", self.0)
    }
}
impl FromStr for Identifier {
    type Err = IdentifierParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(c) = Self::is_valid(s) { return Err(IdentifierParseError::IllegalChar { raw: s.into(), c }); }
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

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for Identifier {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl AsRef<str> for Identifier {
    #[inline]
    fn as_ref(&self) -> &str { &self.0 }
}
impl AsMut<str> for Identifier {
    #[inline]
    fn as_mut(&mut self) -> &mut str { &mut self.0 }
}
impl AsRef<String> for Identifier {
    #[inline]
    fn as_ref(&self) -> &String { &self.0 }
}
impl AsMut<String> for Identifier {
    #[inline]
    fn as_mut(&mut self) -> &mut String { &mut self.0 }
}
impl From<&str> for Identifier {
    #[inline]
    fn from(value: &str) -> Self { Self(value.into()) }
}
impl From<&mut str> for Identifier {
    #[inline]
    fn from(value: &mut str) -> Self { Self(value.into()) }
}
impl From<String> for Identifier {
    #[inline]
    fn from(value: String) -> Self { Self(value) }
}
impl From<&String> for Identifier {
    #[inline]
    fn from(value: &String) -> Self { Self(value.clone()) }
}
impl From<&mut String> for Identifier {
    #[inline]
    fn from(value: &mut String) -> Self { Self(value.clone()) }
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
    fn from(value: &mut Identifier) -> Self { value.0.clone() }
}
impl<'i> From<&'i Identifier> for &'i String {
    #[inline]
    fn from(value: &'i Identifier) -> Self { &value.0 }
}
impl<'i> From<&'i mut Identifier> for &'i String {
    #[inline]
    fn from(value: &'i mut Identifier) -> Self { &value.0 }
}
impl<'i> From<&'i mut Identifier> for &'i mut String {
    #[inline]
    fn from(value: &'i mut Identifier) -> Self { &mut value.0 }
}

impl AsRef<Identifier> for Identifier {
    #[inline]
    fn as_ref(&self) -> &Self { self }
}
impl AsMut<Identifier> for Identifier {
    #[inline]
    fn as_mut(&mut self) -> &mut Self { self }
}
