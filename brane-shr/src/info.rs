//  INFO.rs
//    by Lut99
// 
//  Created:
//    12 Jun 2023, 14:55:22
//  Last edited:
//    21 Jun 2023, 12:01:30
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the [`Info`] trait, with the associated auto-implementing
//!   [`YamlInfo`] and [`JsonInfo`] traits, that provide uniform helper
//!   functions that ease serializing/deserializing serde structs.
// 


/***** LIBRARY *****/
use std::error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs::File as TFile;
use tokio::io::AsyncReadExt as _;


/***** ERRORS *****/
/// Defines general errors for configs.
#[derive(Debug)]
pub enum Error<E: Debug> {
    /// Failed to create the output file.
    OutputCreateError{ path: PathBuf, err: std::io::Error },
    /// Failed to open the input file.
    InputOpenError{ path: PathBuf, err: std::io::Error },
    /// Failed to read the input file.
    InputReadError{ path: PathBuf, err: std::io::Error },

    /// Failed to serialize the config to a string.
    StringSerializeError{ err: E },
    /// Failed to serialize the config to a given writer.
    WriterSerializeError{ err: E },
    /// Failed to serialize the config to a given file.
    FileSerializeError{ path: PathBuf, err: E },

    /// Failed to deserialize a string to the config.
    StringDeserializeError{ err: E },
    /// Failed to deserialize a reader to the config.
    ReaderDeserializeError{ err: E },
    /// Failed to deserialize a file to the config.
    FileDeserializeError{ path: PathBuf, err: E },
}
impl<E: error::Error> Display for Error<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            OutputCreateError{ path, .. } => write!(f, "Failed to create output file '{}'", path.display()),
            InputOpenError{ path, .. }    => write!(f, "Failed to open input file '{}'", path.display()),
            InputReadError{ path, .. }    => write!(f, "Failed to read input file '{}'", path.display()),

            StringSerializeError{ .. }     => write!(f, "Failed to serialize to string"),
            WriterSerializeError{ .. }     => write!(f, "Failed to serialize to a writer"),
            FileSerializeError{ path, .. } => write!(f, "Failed to serialize to output file '{}'", path.display()),

            StringDeserializeError{ .. }     => write!(f, "Failed to deserialize from string"),
            ReaderDeserializeError{ .. }     => write!(f, "Failed to deserialize from a reader"),
            FileDeserializeError{ path, .. } => write!(f, "Failed to deserialize from input file '{}'", path.display()),
        }
    }
}
impl<E: 'static + error::Error> error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            OutputCreateError{ err, .. } => Some(err),
            InputOpenError{ err, .. }    => Some(err),
            InputReadError{ err, .. }    => Some(err),

            StringSerializeError{ err }   => Some(err),
            WriterSerializeError{ err }   => Some(err),
            FileSerializeError{ err, .. } => Some(err),

            StringDeserializeError{ err }   => Some(err),
            ReaderDeserializeError{ err }   => Some(err),
            FileDeserializeError{ err, .. } => Some(err),
        }
    }
}



/// A type alias for the ConfigError for the YamlConfig.
pub type YamlError = Error<serde_yaml::Error>;

/// A type alias for the ConfigError for the YamlConfig.
pub type JsonError = Error<serde_json::Error>;





/***** AUXILLARY *****/
/// Defines an interface to some serializer/deserializer for the [`Info`] trait.
pub trait Interface {
    /// Defines the errors returned by the `InfoInterface`'s functions.
    type Error: error::Error;


    /// Attempts to deserialize the given string as an instance of the given object.
    /// 
    /// # Generic arguments
    /// - `D`: A [`Deserialize`]-capable type to deserialize to.
    /// 
    /// # Arguments
    /// - `raw`: The raw string to deserialize.
    /// 
    /// # Returns
    /// An instance of `D` that is parsed from `raw`.
    /// 
    /// # Errors
    /// This function may error if the string was not valid for `D`.
    fn from_string<'de, D: Deserialize<'de>>(raw: &'de str) -> Result<D, Self::Error>;

    /// Attempts to deserialize the contents of the given reader as an instance of the given object.
    /// 
    /// # Generic arguments
    /// - `D`: A [`Deserialize`]-capable type to deserialize to.
    /// 
    /// # Arguments
    /// - `reader`: The [`Read`]er who's contents to deserialize.
    /// 
    /// # Returns
    /// An instance of `D` that is parsed from `reader`.
    /// 
    /// # Errors
    /// This function may error if the reader failed or if the contents were not valid for `D`.
    fn from_reader<D: for<'de> Deserialize<'de>>(reader: impl Read) -> Result<D, Self::Error>;


    /// Serializes a given object to string.
    /// 
    /// # Arguments
    /// - `info`: Some [`Serialize`]able type.
    /// 
    /// # Returns
    /// A [`String`] containing the serialized `info`.
    /// 
    /// # Errors
    /// This function is allowed to error if the input couldn't be serialized.
    fn to_string(info: impl Serialize) -> Result<String, Self::Error>;
    /// Serializes a given object to string, using a more human-readable format than the typical [`IntoInterface::to_string()`] would.
    /// 
    /// Note that by default, this function just redirects to [`IntoInterface::to_string()`].
    /// 
    /// # Arguments
    /// - `info`: Some [`Serialize`]able type.
    /// 
    /// # Returns
    /// A [`String`] containing the serialized `info`.
    /// 
    /// # Errors
    /// This function is allowed to error if the input couldn't be serialized.
    #[inline]
    fn to_string_pretty(info: impl Serialize) -> Result<String, Self::Error> { Self::to_string(info) }

    /// Serializes a given object to a writer.
    /// 
    /// # Arguments
    /// - `writer`: The [`Write`]er to write the serialized `info` to.
    /// - `info`: Some [`Serialize`]able type.
    /// 
    /// # Errors
    /// This function is allowed to error if the input couldn't be serialized or the writer couldn't be written to.
    fn to_writer(writer: impl Write, info: impl Serialize) -> Result<(), Self::Error>;
    /// Serializes a given object to a writer, using a more human-readable format than the typical [`IntoInterface::to_string`] would.
    /// 
    /// Note that by default, this function just redirects to [`IntoInterface::to_writer()`].
    /// 
    /// # Arguments
    /// - `writer`: The [`Write`]er to write the serialized `info` to.
    /// - `info`: Some [`Serialize`]able type.
    /// 
    /// # Errors
    /// This function is allowed to error if the input couldn't be serialized or the writer couldn't be written to.
    fn to_writer_pretty(writer: impl Write, info: impl Serialize) -> Result<(), Self::Error> { Self::to_writer(writer, info) }
}



/// Defines an [`Interface`] that serializes/deserializes for JSON files.
#[derive(Clone, Copy, Debug)]
pub struct JsonInterface;
impl Interface for JsonInterface {
    type Error = serde_json::Error;


    #[inline]
    fn from_string<'de, D: Deserialize<'de>>(raw: &'de str) -> Result<D, Self::Error> {
        serde_json::from_str(raw.as_ref())
    }

    #[inline]
    fn from_reader<D: for<'de> Deserialize<'de>>(reader: impl Read) -> Result<D, Self::Error> {
        serde_json::from_reader(reader)
    }


    #[inline]
    fn to_string(info: impl Serialize) -> Result<String, Self::Error> {
        serde_json::to_string(&info)
    }
    #[inline]
    fn to_string_pretty(info: impl Serialize) -> Result<String, Self::Error> {
        serde_json::to_string_pretty(&info)
    }

    #[inline]
    fn to_writer(writer: impl Write, info: impl Serialize) -> Result<(), Self::Error> {
        serde_json::to_writer(writer, &info)
    }
    #[inline]
    fn to_writer_pretty(writer: impl Write, info: impl Serialize) -> Result<(), Self::Error> {
        serde_json::to_writer_pretty(writer, &info)
    }
}

/// Defines an [`Interface`] that serializes/deserializes for YAML files.
#[derive(Clone, Copy, Debug)]
pub struct YamlInterface;
impl Interface for YamlInterface {
    type Error = serde_yaml::Error;


    #[inline]
    fn from_string<'de, D: Deserialize<'de>>(raw: &'de str) -> Result<D, Self::Error> {
        serde_yaml::from_str(raw.as_ref())
    }

    #[inline]
    fn from_reader<D: for<'de> Deserialize<'de>>(reader: impl Read) -> Result<D, Self::Error> {
        serde_yaml::from_reader(reader)
    }


    #[inline]
    fn to_string(info: impl Serialize) -> Result<String, Self::Error> {
        serde_yaml::to_string(&info)
    }

    #[inline]
    fn to_writer(writer: impl Write, info: impl Serialize) -> Result<(), Self::Error> {
        serde_yaml::to_writer(writer, &info)
    }
}





/***** LIBRARY *****/
/// Defines a serializable struct that we typically use for structs that are directly read and written to disk.
#[async_trait]
pub trait Info<I: Interface>: Clone + Debug + for<'de> Deserialize<'de> + Serialize {
    // Child-provided
    /// Serializes this Config to a string.
    /// 
    /// In contrast to `Info::to_string_pretty()`, this may generate a less human-readable version (but that depends on the backend [`Interface`]).
    /// 
    /// # Returns
    /// A new String that represents this config but serialized.
    /// 
    /// # Errors
    /// This function may error if the serialization failed.
    #[inline]
    fn to_string(&self) -> Result<String, Error<I::Error>> {
        match I::to_string(self) {
            Ok(raw)  => Ok(raw),
            Err(err) => Err(Error::StringSerializeError { err }),
        }
    }
    /// Serializes this Config to a string.
    /// 
    /// In contrast to `Info::to_string()`, this may generate a more human-readable version (but that depends on the backend [`Interface`]).
    /// 
    /// # Returns
    /// A new String that represents this config but serialized.
    /// 
    /// # Errors
    /// This function may error if the serialization failed.
    #[inline]
    fn to_string_pretty(&self) -> Result<String, Error<I::Error>> {
        match I::to_string_pretty(self) {
            Ok(raw)  => Ok(raw),
            Err(err) => Err(Error::StringSerializeError { err }),
        }
    }

    /// Serializes this Config to a reader.
    /// 
    /// In contrast to `Info::to_writer_pretty()`, this may generate a less human-readable version (but that depends on the backend [`Interface`]).
    /// 
    /// # Arguments
    /// - `writer`: The [`Write`]r to write the serialized representation to.
    /// 
    /// # Errors
    /// This function may error if the serialization failed or if we failed to write to the given writer.
    fn to_writer(&self, writer: impl Write) -> Result<(), Error<I::Error>> {
        match I::to_writer(writer, self) {
            Ok(raw)  => Ok(raw),
            Err(err) => Err(Error::WriterSerializeError { err }),
        }
    }
    /// Serializes this Config to a reader.
    /// 
    /// In contrast to `Info::to_writer()`, this may generate a more human-readable version (but that depends on the backend [`Interface`]).
    /// 
    /// # Arguments
    /// - `writer`: The [`Write`]r to write the serialized representation to.
    /// 
    /// # Errors
    /// This function may error if the serialization failed or if we failed to write to the given writer.
    fn to_writer_pretty(&self, writer: impl Write) -> Result<(), Error<I::Error>> {
        match I::to_writer_pretty(writer, self) {
            Ok(raw)  => Ok(raw),
            Err(err) => Err(Error::WriterSerializeError { err }),
        }
    }

    /// Deserializes the given string to an instance of ourselves.
    /// 
    /// # Arguments
    /// - `raw`: The raw string to deserialize.
    /// 
    /// # Returns
    /// A new instance of `Self` with its contents read from the given raw string.
    /// 
    /// # Errors
    /// This function may fail if the input string was invalid for this object.
    fn from_string(raw: impl AsRef<str>) -> Result<Self, Error<I::Error>> {
        match I::from_string(raw.as_ref()) {
            Ok(res)  => Ok(res),
            Err(err) => Err(Error::StringDeserializeError { err }),
        }
    }

    /// Deserializes the contents of the given reader to an instance of ourselves.
    /// 
    /// # Arguments
    /// - `reader`: The `Read`er who's contents to deserialize.
    /// 
    /// # Returns
    /// A new instance of `Self` with its contents read from the given reader.
    /// 
    /// # Errors
    /// This function may fail if we failed to read from the reader or if its contents were invalid for this object.
    fn from_reader(reader: impl Read) -> Result<Self, Error<I::Error>> {
        match I::from_reader(reader) {
            Ok(res)  => Ok(res),
            Err(err) => Err(Error::ReaderDeserializeError { err }),
        }
    }


    // Globally deduced
    /// Serializes this Config to a file at the given path.
    /// 
    /// This will always choose a pretty representation of the serialization (if applicable).
    /// 
    /// # Arguments
    /// - `path`: The path where to write the file to.
    /// 
    /// # Errors
    /// This function may error if the serialization failed or if we failed to create and/or write to the file.
    fn to_path(&self, path: impl AsRef<Path>) -> Result<(), Error<I::Error>> {
        let path: &Path = path.as_ref();

        // Attempt to create the new file
        let handle: File = match File::create(path) {
            Ok(handle) => handle,
            Err(err)   => { return Err(Error::OutputCreateError { path: path.into(), err }); },
        };

        // Write it using the child function, wrapping the error that may occur
        match self.to_writer_pretty(handle) {
            Ok(_)                                         => Ok(()),
            Err(Error::WriterSerializeError{ err }) => Err(Error::FileSerializeError { path: path.into(), err }),
            Err(err)                                      => Err(err),
        }
    }

    /// Deserializes this Config from the file at the given path.
    /// 
    /// # Arguments
    /// - `path`: The path where to read the file from.
    /// 
    /// # Errors
    /// This function may fail if we failed to open/read from the file or if its contents were invalid for this object.
    fn from_path(path: impl AsRef<Path>) -> Result<Self, Error<I::Error>> {
        let path: &Path = path.as_ref();

        // Attempt to open the given file
        let handle: File = match File::open(path) {
            Ok(handle) => handle,
            Err(err)   => { return Err(Error::InputOpenError { path: path.into(), err }); },
        };

        // Write it using the child function, wrapping the error that may occur
        match Self::from_reader(handle) {
            Ok(config)                                => Ok(config),
            Err(Error::ReaderDeserializeError{ err }) => Err(Error::FileDeserializeError { path: path.into(), err }),
            Err(err)                                  => Err(err),
        }
    }
    /// Deserializes this Config from the file at the given path, with the reading part done asynchronously.
    /// 
    /// Note that the parsing path cannot be done asynchronously. Also, note that, because serde does not support asynchronous deserialization, we have to read the entire file in one go.
    /// 
    /// # Arguments
    /// - `path`: The path where to read the file from.
    /// 
    /// # Errors
    /// This function may fail if we failed to open/read from the file or if its contents were invalid for this object.
    async fn from_path_async(path: impl Send + AsRef<Path>) -> Result<Self, Error<I::Error>> {
        let path: &Path = path.as_ref();

        // Read the file to a string
        let raw: String = {
            // Attempt to open the given file
            let mut handle: TFile = match TFile::open(path).await {
                Ok(handle) => handle,
                Err(err)   => { return Err(Error::InputOpenError { path: path.into(), err }); },
            };

            // Read everything to a string
            let mut raw: String = String::new();
            if let Err(err) = handle.read_to_string(&mut raw).await { return Err(Error::InputReadError{ path: path.into(), err }); }
            raw
        };

        // Write it using the child function, wrapping the error that may occur
        match Self::from_string(raw) {
            Ok(config)                                => Ok(config),
            Err(Error::ReaderDeserializeError{ err }) => Err(Error::FileDeserializeError { path: path.into(), err }),
            Err(err)                                  => Err(err),
        }
    }
}



/// Provides a default implementation for an [`Info`] with a [`JsonInterface`].
pub trait JsonInfo: Clone + Debug + for<'de> Deserialize<'de> + Serialize {}
impl<T: JsonInfo> Info<JsonInterface> for T {}

/// Provides a default implementation for an [`Info`] with a [`YamlInterface`].
pub trait YamlInfo: Clone + Debug + for<'de> Deserialize<'de> + Serialize {}
impl<T: YamlInfo> Info<YamlInterface> for T {}
