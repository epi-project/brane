//  ERRORS.rs
//    by Lut99
//
//  Created:
//    26 Sep 2022, 15:13:34
//  Last edited:
//    16 Jan 2024, 17:27:57
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the errors that may occur in the `brane-reg` crate.
//

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::net::SocketAddr;
use std::path::PathBuf;


/***** LIBRARY *****/
/// Defines Store-related errors.
#[derive(Debug)]
pub enum StoreError {
    /// Failed to parse from the given reader.
    ReaderParseError { err: serde_yaml::Error },

    /// Failed to open the store file.
    FileOpenError { path: PathBuf, err: std::io::Error },
    /// Failed to parse the store file.
    FileParseError { path: PathBuf, err: serde_yaml::Error },

    /// Failed to read the given directory.
    DirReadError { path: PathBuf, err: std::io::Error },
    /// Failed to read an entry in the given directory.
    DirReadEntryError { path: PathBuf, i: usize, err: std::io::Error },
    /// Failed to read the AssetInfo file.
    AssetInfoReadError { path: PathBuf, err: specifications::data::AssetInfoError },
}

impl Display for StoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use StoreError::*;
        match self {
            ReaderParseError { .. } => write!(f, "Failed to parse the given store reader as YAML"),

            FileOpenError { path, .. } => write!(f, "Failed to open store file '{}'", path.display()),
            FileParseError { path, .. } => write!(f, "Failed to parse store file '{}' as YAML", path.display()),

            DirReadError { path, .. } => write!(f, "Failed to read directory '{}'", path.display()),
            DirReadEntryError { path, i, .. } => write!(f, "Failed to read entry {} in directory '{}'", i, path.display()),
            AssetInfoReadError { path, .. } => write!(f, "Failed to load asset info file '{}'", path.display()),
        }
    }
}

impl Error for StoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use StoreError::*;
        match self {
            ReaderParseError { err } => Some(err),

            FileOpenError { err, .. } => Some(err),
            FileParseError { err, .. } => Some(err),

            DirReadError { err, .. } => Some(err),
            DirReadEntryError { err, .. } => Some(err),
            AssetInfoReadError { err, .. } => Some(err),
        }
    }
}



/// Errors that relate to the customized serving process of warp.
#[derive(Debug)]
pub enum ServerError {
    /// Failed to create a new TcpListener and bind it to the given address.
    ServerBindError { address: SocketAddr, err: std::io::Error },
    /// Failed to load the keypair.
    KeypairLoadError { err: brane_cfg::certs::Error },
    /// Failed to load the certificate root store.
    StoreLoadError { err: brane_cfg::certs::Error },
    /// Failed to create a new ServerConfig for the TLS setup.
    ServerConfigError { err: rustls::Error },
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ServerError::*;
        match self {
            ServerBindError { address, .. } => write!(f, "Failed to bind new TCP server to '{address}'"),
            KeypairLoadError { .. } => write!(f, "Failed to load keypair"),
            StoreLoadError { .. } => write!(f, "Failed to load root store"),
            ServerConfigError { .. } => write!(f, "Failed to create new TLS server configuration"),
        }
    }
}

impl Error for ServerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use ServerError::*;
        match self {
            ServerBindError { err, .. } => Some(err),
            KeypairLoadError { err } => Some(err),
            StoreLoadError { err } => Some(err),
            ServerConfigError { err } => Some(err),
        }
    }
}



/// Errors that relate to the `/data` path (and nested).
#[derive(Debug)]
pub enum DataError {
    /// Failed to serialize the contents of the store file (i.e., all known datasets)
    StoreSerializeError { err: serde_json::Error },
    /// Failed to serialize the contents of a single dataset.
    AssetSerializeError { name: String, err: serde_json::Error },

    /// Failed to create a temporary directory.
    TempDirCreateError { err: std::io::Error },
    /// Failed to archive the given dataset.
    DataArchiveError { err: brane_shr::fs::Error },
    /// Failed to re-open the tar file after compressing.
    TarOpenError { path: PathBuf, err: std::io::Error },
    /// Failed to read from the tar file.
    TarReadError { path: PathBuf, err: std::io::Error },
    /// Failed to send chunk of bytes on the body.
    TarSendError { err: warp::hyper::Error },
    /// The given file was not a file, nor a directory.
    UnknownFileTypeError { path: PathBuf },
    /// The given data path does not point to a data set, curiously enough.
    MissingData { name: String, path: PathBuf },
    /// The given result does not point to a data set, curiously enough.
    MissingResult { name: String, path: PathBuf },
}

impl Display for DataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use DataError::*;
        match self {
            StoreSerializeError { .. } => write!(f, "Failed to serialize known datasets"),
            AssetSerializeError { name, .. } => write!(f, "Failed to serialize dataset metadata for dataset '{name}'"),

            TempDirCreateError { .. } => write!(f, "Failed to create a temporary directory"),
            DataArchiveError { .. } => write!(f, "Failed to archive data"),
            TarOpenError { path, .. } => write!(f, "Failed to re-open tarball file '{}'", path.display()),
            TarReadError { path, .. } => write!(f, "Failed to read from tarball file '{}'", path.display()),
            TarSendError { .. } => write!(f, "Failed to send chunk of tarball file as body"),
            UnknownFileTypeError { path } => {
                write!(f, "Dataset file '{}' is neither a file, nor a directory; don't know what to do with it", path.display())
            },
            MissingData { name, path } => write!(f, "The data of dataset '{}' should be at '{}', but doesn't exist", name, path.display()),
            MissingResult { name, path } => {
                write!(f, "The data of intermediate result '{}' should be at '{}', but doesn't exist", name, path.display())
            },
        }
    }
}

impl Error for DataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use DataError::*;
        match self {
            StoreSerializeError { err } => Some(err),
            AssetSerializeError { err, .. } => Some(err),

            TempDirCreateError { err } => Some(err),
            DataArchiveError { err } => Some(err),
            TarOpenError { err, .. } => Some(err),
            TarReadError { err, .. } => Some(err),
            TarSendError { err, .. } => Some(err),
            UnknownFileTypeError { .. } => None,
            MissingData { .. } => None,
            MissingResult { .. } => None,
        }
    }
}

impl warp::reject::Reject for DataError {}



/// Errors that relate to checker authorization.
#[derive(Debug)]
pub enum AuthorizeError {
    /// The client did not provide us with a certificate.
    ClientNoCert,

    /// Failed to load the policy file.
    PolicyFileError { err: brane_cfg::policies::Error },
    /// No policy matched this user/data pair.
    NoUserPolicy { user: String, data: String },
}

impl Display for AuthorizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use AuthorizeError::*;
        match self {
            ClientNoCert => write!(f, "No certificate provided"),

            PolicyFileError { .. } => write!(f, "Failed to load policy file"),
            NoUserPolicy { user, data } => {
                write!(f, "No matching policy rule found for user '{user}' / data '{data}' (did you forget a final AllowAll/DenyAll?)")
            },
        }
    }
}

impl Error for AuthorizeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use AuthorizeError::*;
        match self {
            ClientNoCert => None,

            PolicyFileError { err } => Some(err),
            NoUserPolicy { .. } => None,
        }
    }
}
