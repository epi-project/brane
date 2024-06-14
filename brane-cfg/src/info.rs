//  INFO.rs
//    by Lut99
//
//  Created:
//    28 Feb 2023, 10:07:36
//  Last edited:
//    14 Jun 2024, 15:12:07
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the general [`Info`]-trait, which is used to abstract over the
//!   various types of disk-stored configuration files.
//

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::fs::File as TFile;
use tokio::io::AsyncReadExt as _;


/***** ERRORS *****/
/// Defines general errors for configs.
#[derive(Debug)]
pub enum InfoError<E: Debug> {
    /// Failed to create the output file.
    OutputCreateError { path: PathBuf, err: std::io::Error },
    /// Failed to open the input file.
    InputOpenError { path: PathBuf, err: std::io::Error },
    /// Failed to read the input file.
    InputReadError { path: PathBuf, err: std::io::Error },

    /// Failed to serialize the config to a string.
    StringSerializeError { err: E },
    /// Failed to serialize the config to a given writer.
    WriterSerializeError { err: E },
    /// Failed to serialize the config to a given file.
    FileSerializeError { path: PathBuf, err: E },

    /// Failed to deserialize a string to the config.
    StringDeserializeError { err: E },
    /// Failed to deserialize a reader to the config.
    ReaderDeserializeError { err: E },
    /// Failed to deserialize a file to the config.
    FileDeserializeError { path: PathBuf, err: E },
}
impl<E: Error> Display for InfoError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use InfoError::*;
        match self {
            OutputCreateError { path, .. } => write!(f, "Failed to create output file '{}'", path.display()),
            InputOpenError { path, .. } => write!(f, "Failed to open input file '{}'", path.display()),
            InputReadError { path, .. } => write!(f, "Failed to read input file '{}'", path.display()),

            StringSerializeError { .. } => write!(f, "Failed to serialize to string"),
            WriterSerializeError { .. } => write!(f, "Failed to serialize to a writer"),
            FileSerializeError { path, .. } => write!(f, "Failed to serialize to output file '{}'", path.display()),

            StringDeserializeError { .. } => write!(f, "Failed to deserialize from string"),
            ReaderDeserializeError { .. } => write!(f, "Failed to deserialize from a reader"),
            FileDeserializeError { path, .. } => write!(f, "Failed to deserialize from input file '{}'", path.display()),
        }
    }
}
impl<E: 'static + Error> Error for InfoError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use InfoError::*;
        match self {
            OutputCreateError { err, .. } => Some(err),
            InputOpenError { err, .. } => Some(err),
            InputReadError { err, .. } => Some(err),

            StringSerializeError { err } => Some(err),
            WriterSerializeError { err } => Some(err),
            FileSerializeError { err, .. } => Some(err),

            StringDeserializeError { err } => Some(err),
            ReaderDeserializeError { err } => Some(err),
            FileDeserializeError { err, .. } => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// Defines a serializable struct that we typically use for structs that are directly read and written to disk.
#[async_trait]
pub trait Info: Clone + Debug {
    /// The types of errors that may be thrown by the serialization function(s).
    type Error: Error;


    // Child-provided
    /// Serializes this Config to a string.
    ///
    /// # Arguments
    /// - `pretty`: If true, then it will be serialized using a pretty version of the backend (if available).
    ///
    /// # Returns
    /// A new String that represents this config but serialized.
    ///
    /// # Errors
    /// This function may error if the serialization failed.
    fn to_string(&self, pretty: bool) -> Result<String, InfoError<Self::Error>>;
    /// Serializes this Config to a reader.
    ///
    /// # Arguments
    /// - `writer`: The `Write`r to write the serialized representation to.
    /// - `pretty`: If true, then it will be serialized using a pretty version of the backend (if available).
    ///
    /// # Errors
    /// This function may error if the serialization failed or if we failed to write to the given writer.
    fn to_writer(&self, writer: impl Write, pretty: bool) -> Result<(), InfoError<Self::Error>>;

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
    fn from_string(raw: impl AsRef<str>) -> Result<Self, InfoError<Self::Error>>;
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
    fn from_reader(reader: impl Read) -> Result<Self, InfoError<Self::Error>>;


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
    fn to_path(&self, path: impl AsRef<Path>) -> Result<(), InfoError<Self::Error>> {
        let path: &Path = path.as_ref();

        // Attempt to create the new file
        let handle: File = match File::create(path) {
            Ok(handle) => handle,
            Err(err) => {
                return Err(InfoError::OutputCreateError { path: path.into(), err });
            },
        };

        // Write it using the child function, wrapping the error that may occur
        match self.to_writer(handle, true) {
            Ok(_) => Ok(()),
            Err(InfoError::WriterSerializeError { err }) => Err(InfoError::FileSerializeError { path: path.into(), err }),
            Err(err) => Err(err),
        }
    }

    /// Deserializes this Config from the file at the given path.
    ///
    /// # Arguments
    /// - `path`: The path where to read the file from.
    ///
    /// # Errors
    /// This function may fail if we failed to open/read from the file or if its contents were invalid for this object.
    fn from_path(path: impl AsRef<Path>) -> Result<Self, InfoError<Self::Error>> {
        let path: &Path = path.as_ref();

        // Attempt to open the given file
        let handle: File = match File::open(path) {
            Ok(handle) => handle,
            Err(err) => {
                return Err(InfoError::InputOpenError { path: path.into(), err });
            },
        };

        // Write it using the child function, wrapping the error that may occur
        match Self::from_reader(handle) {
            Ok(config) => Ok(config),
            Err(InfoError::ReaderDeserializeError { err }) => Err(InfoError::FileDeserializeError { path: path.into(), err }),
            Err(err) => Err(err),
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
    async fn from_path_async(path: impl Send + AsRef<Path>) -> Result<Self, InfoError<Self::Error>> {
        let path: &Path = path.as_ref();

        // Read the file to a string
        let raw: String = {
            // Attempt to open the given file
            let mut handle: TFile = match TFile::open(path).await {
                Ok(handle) => handle,
                Err(err) => {
                    return Err(InfoError::InputOpenError { path: path.into(), err });
                },
            };

            // Read everything to a string
            let mut raw: String = String::new();
            if let Err(err) = handle.read_to_string(&mut raw).await {
                return Err(InfoError::InputReadError { path: path.into(), err });
            }
            raw
        };

        // Write it using the child function, wrapping the error that may occur
        match Self::from_string(raw) {
            Ok(config) => Ok(config),
            Err(InfoError::ReaderDeserializeError { err }) => Err(InfoError::FileDeserializeError { path: path.into(), err }),
            Err(err) => Err(err),
        }
    }
}



/// A marker trait that will let the compiler implement `Config` for this object using the `serde_yaml` backend.
pub trait YamlInfo<'de>: Clone + Debug + Deserialize<'de> + Serialize {}
impl<T: DeserializeOwned + Serialize + for<'de> YamlInfo<'de>> Info for T {
    type Error = serde_yaml::Error;

    fn to_string(&self, _pretty: bool) -> Result<String, InfoError<Self::Error>> {
        match serde_yaml::to_string(self) {
            Ok(raw) => Ok(raw),
            Err(err) => Err(InfoError::StringSerializeError { err }),
        }
    }

    fn to_writer(&self, writer: impl Write, _pretty: bool) -> Result<(), InfoError<Self::Error>> {
        match serde_yaml::to_writer(writer, self) {
            Ok(raw) => Ok(raw),
            Err(err) => Err(InfoError::ReaderDeserializeError { err }),
        }
    }

    fn from_string(raw: impl AsRef<str>) -> Result<Self, InfoError<Self::Error>> {
        match serde_yaml::from_str(raw.as_ref()) {
            Ok(config) => Ok(config),
            Err(err) => Err(InfoError::StringDeserializeError { err }),
        }
    }

    fn from_reader(reader: impl Read) -> Result<Self, InfoError<Self::Error>> {
        match serde_yaml::from_reader(reader) {
            Ok(config) => Ok(config),
            Err(err) => Err(InfoError::ReaderDeserializeError { err }),
        }
    }
}

/// A type alias for the ConfigError for the YamlConfig.
pub type YamlError = InfoError<serde_yaml::Error>;
