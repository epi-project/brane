//  K 8S.rs
//    by Lut99
// 
//  Created:
//    08 May 2023, 13:01:23
//  Last edited:
//    10 May 2023, 16:51:33
//  Auto updated?
//    Yes
// 
//  Description:
//!   Provides an API for running Brane tasks on a Kubernetes backend.
// 

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::path::{Path, PathBuf};

pub use kube::Config;
pub use k8s_openapi::api::core::v1::Pod;
pub use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::NamespaceResourceScope;
use kube::api::{Api, Resource};
use kube::config::{Kubeconfig, KubeConfigOptions};
use log::{debug, info, warn};
use tokio::fs as tfs;

use brane_shr::fs::{download_file_async, set_executable, unarchive_async, DownloadSecurity};
use specifications::address::Address;
use specifications::container::Image;

use crate::docker::ImageSource;


/***** CONSTANTS *****/
/// Defines the address we download the x86-64 `crane` tar from.
pub const CRANE_TAR_URL_X86_64: &'static str = "https://github.com/google/go-containerregistry/releases/download/v0.15.1/go-containerregistry_Linux_x86_64.tar.gz";
/// Defines the address we download the ARM64 `crane` tar from.
pub const CRANE_TAR_URL_ARM64: &'static str = "https://github.com/google/go-containerregistry/releases/download/v0.15.1/go-containerregistry_Linux_arm64.tar.gz";

/// The location where we expect the `crane` executable to be, locally.
pub const CRANE_PATH: &'static str = "/tmp/crane";
/// The checksum of the executable.
pub const CRANE_TAR_CHECKSUM: &'static str = "";





/***** ERRORS *****/
/// Defines errors that occur when reading a config.
#[derive(Debug)]
pub enum ConfigError {
    /// Failed to open the config file for reading.
    FileRead{ path: PathBuf, err: std::io::Error },
    /// Failed to parse the given file as a valid kube config YAML file.
    Parse{ path: PathBuf, err: serde_yaml::Error },
    /// Failed to compile the parsed YAML to a Kubernetes configuration.
    Compile{ path: PathBuf, err: kube::config::KubeconfigError },
}
impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ConfigError::*;
        match self {
            FileRead { path, err } => write!(f, "Failed to read file '{}': {}", path.display(), err),
            Parse{ path, .. }      => write!(f, "Failed to parse file '{}' as a Kubernetes configuration YAML file", path.display()),
            Compile{ path, .. }    => write!(f, "Failed to compile parsed Kubernetes configuration YAML (from '{}')", path.display()),
        }
    }
}
impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use ConfigError::*;
        match self {
            FileRead { .. }    => None,
            Parse{ err, .. }   => Some(err),
            Compile{ err, .. } => Some(err),
        }
    }
}

/// Defines errors that occur when working with clients.
#[derive(Debug)]
pub enum ClientError {
    /// Failed to load a given config file.
    LoadConfig{ err: ConfigError },
    /// Failed to create a [`kube::Client`] from a [`kube::Config`].
    CreateClient{ err: kube::Error },
}
impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ClientError::*;
        match self {
            LoadConfig{ .. }    => write!(f, "Failed to load Kubernetes client config file"),
            CreateClient { .. } => write!(f, "Failed to create client from given config"),
        }
    }
}
impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use ClientError::*;
        match self {
            LoadConfig{ err }    => Some(err),
            CreateClient { err } => Some(err),
        }
    }
}

/// Defines errors that occur when working with connections.
#[derive(Debug)]
pub enum ConnectionError {
    /// Failed to download the `crane` executable tarball.
    DownloadCraneTar{ from: &'static str, to: PathBuf, err: brane_shr::fs::Error },
    /// Failed to unpack the `crane` executable tarball.
    UnpackCraneTar{ from: PathBuf, to: PathBuf, err: brane_shr::fs::Error },
    /// Failed to move the `crane` executable from the downloaded folder to the target path.
    MoveCrane{ from: PathBuf, to: PathBuf, err: std::io::Error },
    /// Failed to make the `crane` executable... executable.
    MakeCraneExecutable{ path: PathBuf, err: brane_shr::fs::Error },
}
impl Display for ConnectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ConnectionError::*;
        match self {
            DownloadCraneTar{ from, to, .. } => write!(f, "Failed to download `crane` executable tarball from '{}' to '{}'", from, to.display()),
            UnpackCraneTar{ from, to, .. }   => write!(f, "Failed to unpack `crane` executable tarball from '{}' to '{}'", from.display(), to.display()),
            MoveCrane{ from, to, .. }        => write!(f, "Failed to move `crane` executable from '{}' to '{}'", from.display(), to.display()),
            MakeCraneExecutable{ path, .. }  => write!(f, "Failed to make `crane` executable '{}' executable", path.display()),
        }
    }
}
impl Error for ConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use ConnectionError::*;
        match self {
            DownloadCraneTar{ err, .. }    => Some(err),
            UnpackCraneTar{ err, .. }      => Some(err),
            MoveCrane{ err, .. }           => Some(err),
            MakeCraneExecutable{ err, .. } => Some(err),
        }
    }
}





/***** HELPER FUNCTIONS *****/
/// Ensures that the `crane` executable is downloaded is some recognizable location.
/// 
/// # Errors
/// This function errors if we failed to find _and_ download it.
async fn ensure_crate_exe() -> Result<(), ConnectionError> {
    debug!("Ensuring `crate` executable existance...");

    // Resolve where to get the executable from
    #[cfg(target_arch = "x86_64")]
    const URL: &'static str = CRANE_TAR_URL_X86_64;
    #[cfg(target_arch = "aarch64")]
    const URL: &'static str = CRANE_TAR_URL_ARM64;
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    compile_error!("Unsupported non-x86_64, non-ARM64 architecture");

    // Check if it already exists, that's nice then
    if PathBuf::from(CRANE_PATH).exists() {
        debug!("Executable '{}' found, marked as present", CRANE_PATH);
        return Ok(());
    }

    // Otherwise, we should attempt to download the crane executable's tarball
    let tar_path: PathBuf = "/tmp/go-containerregistry_Linux.tar.gz".into();
    debug!("Downloading '{}' to '{}'...", URL, tar_path.display());
    if let Err(err) = download_file_async(URL, &tar_path, DownloadSecurity::all(CRANE_TAR_CHECKSUM.as_bytes()), None).await {
        return Err(ConnectionError::DownloadCraneTar { from: URL, to: tar_path, err });
    }

    // Unpack the tarball
    let dir_path: PathBuf = "/tmp/go-containerregistry_Linux".into();
    debug!("Unpacking '{}' to '{}'...", tar_path.display(), dir_path.display());
    if let Err(err) = unarchive_async(&tar_path, &dir_path).await {
        return Err(ConnectionError::UnpackCraneTar{ from: tar_path, to: dir_path, err });
    }

    // Move the directory's crane executable to the target location
    let crane_path: PathBuf = dir_path.join("crane");
    debug!("Extracting '{}' to '{}'...", crane_path.display(), CRANE_PATH);
    if let Err(err) = tfs::copy(&crane_path, CRANE_PATH).await {
        return Err(ConnectionError::MoveCrane{ from: crane_path, to: CRANE_PATH.into(), err });
    }
    // Make it executable, too
    if let Err(err) = set_executable(CRANE_PATH) {
        return Err(ConnectionError::MakeCraneExecutable{ path: CRANE_PATH.into(), err });
    }

    // Finally, delete the tar and directory
    if let Err(err) = tfs::remove_dir_all(&dir_path).await { warn!("Failed to remove extracted tarball folder '{}': {}", dir_path.display(), err); }
    if let Err(err) = tfs::remove_file(&tar_path).await { warn!("Failed to remove downloaded tarball '{}': {}", tar_path.display(), err); }

    // Done!
    debug!("Successfully downloaded `crane` executable to {CRANE_PATH}");
    Ok(())
}





/***** AUXILLARY FUNCTIONS *****/
/// Reads a Kubernetes config file from the given path on the disk.
/// 
/// This file can then be used to configure a Kubernetes client.
/// 
/// # Arguments
/// - `path`: The [`Path`]-like to read the config file from.
/// 
/// # Returns
/// A new [`Config`] that can be used to connect to the instance it describes.
/// 
/// # Errors
/// This function may error if we failed to read the config file or if it was invalid.
pub async fn read_config_async(path: impl AsRef<Path>) -> Result<Config, ConfigError> {
    let path: &Path = path.as_ref();

    // Read the file to a string
    let raw: String = match tfs::read_to_string(path).await {
        Ok(raw)  => raw,
        Err(err) => { return Err(ConfigError::FileRead { path: path.into(), err }); },
    };

    // Read it with serde to a Kubeconfig
    let config: Kubeconfig = match serde_yaml::from_str(&raw) {
        Ok(config) => config,
        Err(err)   => { return Err(ConfigError::Parse{ path: path.into(), err }); },
    };

    // Finally, wrap that in an official config
    match Config::from_custom_kubeconfig(config, &KubeConfigOptions::default()).await {
        Ok(config) => Ok(config),
        Err(err)   => Err(ConfigError::Compile{ path: path.into(), err }),
    }
}



/// Resolves the [`ImageSource`] to an [`ImageSource::Registry`].
/// 
/// If we're given an [`ImageSource::Path`], we upload the container to the given registry. Otherwise, we just return as-is.
/// 
/// # Arguments
/// - `image`: The [`ImageSource`] to resolve.
/// - `registry`: The address of the registry to upload the image to if necessary.
/// 
/// # Returns
/// Another [`ImageSource`] that is the resolved version of `image`.
/// 
/// # Errors
/// This function may error if the given `image` was an [`ImageSource::Path`], and we failed to upload the image.
pub fn resolve_image_source(image: ImageSource, registry: Address) -> Result<ImageSource, ResolveError> {
    
}





/***** AUXILLARY STRUCTURES *****/
/// Defines a struct with K8s-specific options to pass to this API.
#[derive(Clone, Debug)]
pub struct K8sOptions {
    /// The path to the Kubernetes config file to connect with.
    pub config           : PathBuf,
    /// The address of the Docker registry that we push container images to.
    pub registry_address : Address,
}

/// Defines a struct that describes everything we need to know about a job for a Kubernetes task.
#[derive(Clone, Debug)]
pub struct ExecuteInfo {
    /// The name of the container-to-be.
    pub name         : String,
    /// The image name to use for the container.
    pub image        : Image,
    /// The location where we import (as file) or create (from repo) the image from.
    pub image_source : ImageSource,

    /// The command(s) to pass to Branelet.
    pub command      : Vec<String>,
}





/***** LIBRARY *****/
/// Represents a client for a Kubernetes cluster. Practically acts as a builder for a connection.
#[derive(Clone)]
pub struct Client {
    /// A Kubernetes config to wrap around.
    client           : kube::Client,
    /// The registry address which we use to transfer images to Kubernetes.
    registry_address : Address,
}

impl Client {
    /// Constructor for the Client.
    /// 
    /// # Arguments
    /// - `config`: The [`Config`] that we use to known to which cluster to connect and how.
    /// - `registry_address`: The address of the Docker Registry that we can use to temporarily upload Docker images.
    /// 
    /// # Returns
    /// A new Client instance that can be used to connect to the cluster described in the given config.
    /// 
    /// # Errors
    /// This function errors if we failed to create a [`kube::Client`] from the given `config`.
    #[inline]
    pub fn new(config: impl Into<Config>, registry_address: impl Into<Address>) -> Result<Self, ClientError> {
        // Attempt to create a client from the given config
        let client: kube::Client = match kube::Client::try_from(config.into()) {
            Ok(client) => client,
            Err(err)   => { return Err(ClientError::CreateClient{ err }); },
        };

        // Return ourselves with the client
        Ok(Self {
            client,
            registry_address : registry_address.into(),
        })
    }

    /// Constructor for the Client that parses the Kubernetes config from the given path.
    /// 
    /// # Arguments
    /// - `path`: The [`Path`]-like to parse the Kubernetes config from.
    /// - `registry_address`: The address of the Docker Registry that we can use to temporarily upload Docker images.
    /// 
    /// # Returns
    /// A new Client instance that can be used to connect to the cluster described in the given config.
    /// 
    /// # Errors
    /// This function may error if we failed to parse the given file or if we failed to create a [`kube::Client`] from the given `config`.
    pub async fn from_path_async(path: impl AsRef<Path>, registry_address: impl Into<Address>) -> Result<Self, ClientError> {
        // Attempt to load the configuration file
        let config: Config = match read_config_async(path).await {
            Ok(config) => config,
            Err(err)   => { return Err(ClientError::LoadConfig { err }); },
        };

        // Create a client from that
        let client: kube::Client = match kube::Client::try_from(config) {
            Ok(client) => client,
            Err(err)   => { return Err(ClientError::CreateClient { err }); },
        };

        // Return ourselves with the client
        Ok(Self {
            client,
            registry_address : registry_address.into(),
        })
    }



    /// Instantiates a connection with the remote cluster.
    /// 
    /// # Generic arguments
    /// - `R`: The type of [`Resource`] to make this connection for. This scopes the connection to a particular set of things you can do.
    /// 
    /// # Arguments
    /// - `namespace`: The Kubernetes namespace to use for the request.
    /// 
    /// # Returns
    /// A new [`Connection`] representing it.
    #[inline]
    pub fn connect<R: Resource<Scope = NamespaceResourceScope>>(&self, namespace: impl AsRef<str>) -> Connection<R> where R::DynamicType: Default {
        // We create the requested API interface and return that
        Connection {
            api              : Api::namespaced(self.client.clone(), namespace.as_ref()),
            registry_address : &self.registry_address,
        }
    }
}

impl Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        // Format as a typical struct
        f.debug_struct("Client")
            .field("Client", &"<kube::Client>")
            .finish()
    }
}



/// Represents a "connection" between the client and the Kubernetes cluster (at least conceptually).
/// 
/// # Generic arguments
/// - `K`: 
#[derive(Debug)]
pub struct Connection<'c, R> {
    /// The [`API`] abstraction with which we connect.
    api              : Api<R>,
    /// The registry address which we use to transfer images to Kubernetes.
    registry_address : &'c Address,
}

impl<'c> Connection<'c, Job> {
    /// Launches a given job in Kubernetes.
    /// 
    /// # Arguments
    /// - `einfo`: The [`ExecuteInfo`] that describes the job to launch.
    /// 
    /// # Returns
    /// A new [`JobHandle`] struct that can be used to cancel a job or otherwise manage it.
    /// 
    /// # Errors
    /// This function errors if we failed to push the container to the local registry (if it was a file), connect to the cluster or if Kubernetes failed to launch the job.
    pub async fn spawn<'s>(&'s self, einfo: ExecuteInfo) -> Result<JobHandle<'c, 's>, ConnectionError> {
        info!("Spawning package task '{}' from '{}' on Kubernetes backend", einfo.name, einfo.image);

        // Assert the container has been uploaded
        if !matches!(einfo.image_source, ImageSource::Registry(_)) { panic!("Non-Registry ImageSource must have been resolved before calling Connection::spawn"); }

        // Done
        Ok(JobHandle{ connection: self })
    }
}



/// Represents a job that is currently running within a Kubernetes cluster.
#[derive(Debug)]
pub struct JobHandle<'c1, 'c2> {
    /// The connection of which we are a part.
    connection : &'c2 Connection<'c1, Job>,
}
