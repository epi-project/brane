//  INDEX.rs
//    by Lut99
// 
//  Created:
//    12 Jun 2023, 17:13:25
//  Last edited:
//    12 Jun 2023, 18:13:40
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the [`PackageIndex`] and [`DataIndex`] structs, which act as
//!   a registry for packages or datasets at some site (or local).
//!   
//!   _Note:_ If the new-syntax ever comes off the ground, we shouldn't need
//!   this one in specifications anymore, but instead we can move it to
//!   `brane-lnk` or wherever the linker then resides.
// 

use std::collections::HashMap;
use std::error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::fs::{self, DirEntry, ReadDir};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use log::{debug, info, warn};
use reqwest::StatusCode;
use reqwest::blocking as breqwest;
use tokio::fs::{self as tfs, DirEntry as TDirEntry, ReadDir as TReadDir};

use brane_shr::address::Address;
use brane_shr::info::{Info, Interface};


/***** ERRORS *****/
/// Defines common errors that originate from any [`Index`].
#[derive(Debug)]
pub enum Error<E: Debug> {
    /// Failed to read the given directory.
    DirRead { path: PathBuf, err: std::io::Error },
    /// Failed to read an entry within a directory.
    DirEntryRead { path: PathBuf, entry: usize, err: std::io::Error },
    /// Failed to deserialize a particular info.
    InfoDeserialize { path: PathBuf, err: brane_shr::info::Error<E> },

    /// Failed to send GET-request to some address.
    GetRequest { address: Address, err: reqwest::Error },
    /// The GET-request succeeded from our side, but the server send a non-success status code.
    GetRequestFailure { address: Address, status: StatusCode, err: Option<String> },
    /// The server responded with non-UTF-8 (or we failed to read the body)
    GetRequestBody { address: Address, err: reqwest::Error },
    /// Failed to parse the server's contents as JSON.
    GetRequestJson { address: Address, err: serde_json::Error },
}
impl<E: Debug> Display for Error<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            DirRead { path, .. }             => write!(f, "Failed to read directory '{}'", path.display()),
            DirEntryRead { path, entry, .. } => write!(f, "Failed to read entry {entry} in directory '{}'", path.display()),
            InfoDeserialize { path, .. }     => write!(f, "Failed to deserialize info file '{}'", path.display()),

            GetRequest { address, .. }                 => write!(f, "Failed to send GET-request to '{address}'"),
            GetRequestFailure { address, status, err } => write!(f, "Remote '{address}' returned status code {} ({}) in response to GET-request{}", status.as_u16(), status.canonical_reason().unwrap_or("???"), if let Some(err) = err { format!("\n\nMessage:\n{err}") } else { String::new() }),
            GetRequestBody { address, .. }             => write!(f, "Failed to read body of response from '{address}' as UTF-8"),
            GetRequestJson { address, .. }             => write!(f, "Failed to read body of response from '{address}' as valid Index JSON"),
        }
    }
}
impl<E: 'static + error::Error> error::Error for Error<E> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            DirRead { err, .. }         => Some(err),
            DirEntryRead { err, .. }    => Some(err),
            InfoDeserialize { err, .. } => Some(err),

            GetRequest { err, .. }     => Some(err),
            GetRequestFailure { .. }   => None,
            GetRequestBody { err, .. } => Some(err),
            GetRequestJson { err, .. } => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// Defines a trait providing common implementations for the various indices.
#[async_trait]
pub trait Index<I: Interface>: Clone + Debug + From<HashMap<String, Self::Info>> {
    /// The [`Info`] that this Index concerns itself with.
    type Info: Send + Sync + Info<I>;


    /// Constructor for the Index that creates it from the information in the given directory.
    /// 
    /// It considers every nested directory to be a separate entry in the index.
    /// 
    /// Once collected, this function relies on [`From<HashMap<String, Self::Info>>`] to create the final `Self`. It likely wants to replace the found entry names with, say, the names of the info.
    /// 
    /// **Note**: Using this function _within_ a Tokio runtime will crash. Instead, see [`Index::local_async()`].
    /// 
    /// # Arguments
    /// - `directory`: The directory to load the entries for this index from.
    /// - `file`: The canonical filename of the to-be-sought-after file. Essentially, together with `directory`, this creates a path like `directory/<FOR EVERY DIR>/file`.
    /// 
    /// # Returns
    /// A new instance of Self.
    /// 
    /// # Errors
    /// This function may error if something about the directory -or reading the directory- was wrong.
    fn local(directory: impl Send + Sync + AsRef<Path>, file: impl Send + Sync + AsRef<Path>) -> Result<Self, Error<I::Error>> {
        let directory: &Path = directory.as_ref();
        let file: &Path = file.as_ref();
        info!("Starting local construction of {} from '{}' (looking for '{}' files)", std::any::type_name::<Self>(), directory.display(), file.display());

        // Attempt to start reading the directory
        let read_dir: ReadDir = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(err)    => { return Err(Error::DirRead { path: directory.into(), err }); },
        };

        // Read every one of them
        let mut entries: HashMap<String, Self::Info> = HashMap::new();
        for (i, entry) in read_dir.enumerate() {
            // Attempt to unwrap the entry
            let entry: DirEntry = match entry {
                Ok(entry) => entry,
                Err(err)  => { return Err(Error::DirEntryRead { path: directory.into(), entry: i, err }); },
            };

            // Check if the entry has a nested file
            let entry_path: PathBuf = entry.path();
            let info_path: PathBuf = entry_path.join(file);
            if !info_path.exists() {
                warn!("Skipping entry '{}' because it does not have a nested '{}' file", entry_path.display(), file.display());
                continue;
            }
            if !info_path.is_file() {
                warn!("Skipping entry '{}' because its nested file '{}' is not a file", entry_path.display(), info_path.display());
                continue;
            }
            debug!("Found info file '{}'", info_path.display());

            // Attempt to load it
            let info: Self::Info = match Self::Info::from_path(&info_path) {
                Ok(info) => info,
                Err(err) => { return Err(Error::InfoDeserialize { path: info_path, err }); },
            };

            // Add the entry under its directory name
            entries.insert(entry.file_name().to_string_lossy().into(), info);
        }

        // Done!
        Ok(entries.into())
    }
    /// Constructor for the Index that creates it from the information in the given directory using [`tokio`]'s async library.
    /// 
    /// It considers every nested directory to be a separate entry in the index.
    /// 
    /// This function relies on [`From<HashMap<String, Self::Info>>`] to create the final `Self`. It likely wants to replace the found entry names with, say, the names of the info.
    /// 
    /// # Arguments
    /// - `directory`: The directory to load the entries for this index from.
    /// - `file`: The canonical filename of the to-be-sought-after file. Essentially, together with `directory`, this creates a path like `directory/<FOR EVERY DIR>/file`.
    /// 
    /// # Returns
    /// A new instance of Self.
    /// 
    /// # Errors
    /// This function may error if something about the directory -or reading the directory- was wrong.
    async fn local_async(directory: impl Send + Sync + AsRef<Path>, file: impl Send + Sync + AsRef<Path>) -> Result<Self, Error<I::Error>> {
        let directory: &Path = directory.as_ref();
        let file: &Path = file.as_ref();
        info!("Starting local construction of {} from '{}' (looking for '{}' files)", std::any::type_name::<Self>(), directory.display(), file.display());

        // Attempt to start reading the directory
        let mut read_dir: TReadDir = match tfs::read_dir(directory).await {
            Ok(entries) => entries,
            Err(err)    => { return Err(Error::DirRead { path: directory.into(), err }); },
        };

        // Read every one of them
        debug!("Iterating through '{}'...", directory.display());
        let mut i: usize = 0;
        let mut entries: HashMap<String, Self::Info> = HashMap::new();
        while let Some(entry) = read_dir.next_entry().await.transpose() {
            // Attempt to unwrap the entry
            let entry: TDirEntry = match entry {
                Ok(entry) => entry,
                Err(err)  => { return Err(Error::DirEntryRead { path: directory.into(), entry: i, err }); },
            };

            // Check if the entry has a nested file
            let entry_path: PathBuf = entry.path();
            let info_path: PathBuf = entry_path.join(file);
            if !info_path.exists() {
                warn!("Skipping entry '{}' because it does not have a nested '{}' file", entry_path.display(), file.display());
                i += 1;
                continue;
            }
            if !info_path.is_file() {
                warn!("Skipping entry '{}' because its nested file '{}' is not a file", entry_path.display(), info_path.display());
                i += 1;
                continue;
            }
            debug!("Found info file '{}'", info_path.display());

            // Attempt to load it
            let info: Self::Info = match Self::Info::from_path(&info_path) {
                Ok(info) => info,
                Err(err) => { return Err(Error::InfoDeserialize { path: info_path, err }); },
            };

            // Add the entry under its directory name
            entries.insert(entry.file_name().to_string_lossy().into(), info);
            i += 1;
        }

        // Done!
        Ok(entries.into())
    }

    /// Constructor for the Index that creates it by sending a request to a particular remote.
    /// 
    /// It assumes that the host will send back a [`HashMap<String, Self::Info>`], which is then turned into a `Self` using [`From<HashMap<OsString, Self::Info>>`].
    /// 
    /// # Arguments
    /// - `address`: The [`Address`] of the remote to send a request to.
    /// 
    /// # Returns
    /// A new instance of Self.
    /// 
    /// # Errors
    /// This function may error if we failed to reach the remote, the remote returned an error or the remote returned a wrong format.
    fn remote(address: impl AsRef<Address>) -> Result<Self, Error<I::Error>> {
        let address: &Address = address.as_ref();
        info!("Starting remote construction of {} from '{}'", std::any::type_name::<Self>(), address);

        // Start a request to the given address
        let res: breqwest::Response = match breqwest::get(address.to_string()) {
            Ok(res)  => res,
            Err(err) => { return Err(Error::GetRequest { address: address.into(), err }); },
        };
        if !res.status().is_success() { return Err(Error::GetRequestFailure { address: address.into(), status: res.status(), err: res.text().ok() }); }

        // Attempt to get the result as a string
        let res: String = match res.text() {
            Ok(res)  => res,
            Err(err) => { return Err(Error::GetRequestBody { address: address.into(), err }); },
        };
        debug!("Remote returned:\n{}\n{res}\n{}\n\n", (0..80).map(|_| '-').collect::<String>(), (0..80).map(|_| '-').collect::<String>());

        // Attempt to parse the result as a map
        let entries: HashMap<String, Self::Info> = match serde_json::from_str(&res) {
            Ok(entries) => entries,
            Err(err)    => { return Err(Error::GetRequestJson { address: address.into(), err }); },
        };

        // Done
        Ok(entries.into())
    }
    /// Constructor for the Index that creates it by sending a request to a particular remote using [`tokio`]'s async library.
    /// 
    /// It assumes that the host will send back a [`HashMap<String, Self::Info>`], which is then turned into a `Self` using [`From<HashMap<OsString, Self::Info>>`].
    /// 
    /// # Arguments
    /// - `address`: The [`Address`] of the remote to send a request to.
    /// 
    /// # Returns
    /// A new instance of Self.
    /// 
    /// # Errors
    /// This function may error if we failed to reach the remote, the remote returned an error or the remote returned a wrong format.
    async fn remote_async(address: impl Send + Sync + AsRef<Address>) -> Result<Self, Error<I::Error>> {
        let address: &Address = address.as_ref();
        info!("Starting remote construction of {} from '{}'", std::any::type_name::<Self>(), address);

        // Start a request to the given address
        let res: reqwest::Response = match reqwest::get(address.to_string()).await {
            Ok(res)  => res,
            Err(err) => { return Err(Error::GetRequest { address: address.into(), err }); },
        };
        if !res.status().is_success() { return Err(Error::GetRequestFailure { address: address.into(), status: res.status(), err: res.text().await.ok() }); }

        // Attempt to get the result as a string
        let res: String = match res.text().await {
            Ok(res)  => res,
            Err(err) => { return Err(Error::GetRequestBody { address: address.into(), err }); },
        };
        debug!("Remote returned:\n{}\n{res}\n{}\n\n", (0..80).map(|_| '-').collect::<String>(), (0..80).map(|_| '-').collect::<String>());

        // Attempt to parse the result as a map
        let entries: HashMap<String, Self::Info> = match serde_json::from_str(&res) {
            Ok(entries) => entries,
            Err(err)    => { return Err(Error::GetRequestJson { address: address.into(), err }); },
        };

        // Done
        Ok(entries.into())
    }
}
