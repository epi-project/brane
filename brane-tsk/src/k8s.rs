//  K 8S.rs
//    by Lut99
// 
//  Created:
//    08 May 2023, 13:01:23
//  Last edited:
//    09 May 2023, 14:51:49
//  Auto updated?
//    Yes
// 
//  Description:
//!   Provides an API for running Brane tasks on a Kubernetes backend.
// 

use std::error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::path::{Path, PathBuf};

pub use kube::Config;
pub use kube::api::Resource;
pub use k8s_openapi::api::core::v1::Volume;
pub use k8s_openapi::api::batch::v1::Job;
use k8s_openapi::NamespaceResourceScope;
use kube::api::Api;
use kube::config::{Kubeconfig, KubeConfigOptions};
use tokio::fs as tfs;

use specifications::address::Address;


/***** ERRORS *****/
/// Defines the errors that may occur when working with Kubernetes.
#[derive(Debug)]
pub enum Error {
    /// Failed to read a certain configuration file.
    ConfigError{ err: ConfigError },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            ConfigError{ .. } => write!(f, "Failed to read Kubernetes configuration file"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            ConfigError{ err } => Some(err),
        }
    }
}



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
impl error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
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
impl error::Error for ClientError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use ClientError::*;
        match self {
            LoadConfig{ err }    => Some(err),
            CreateClient { err } => Some(err),
        }
    }
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





/***** AUXILLARY STRUCTURES *****/
/// Defines a struct with K8s-specific options to pass to this API.
#[derive(Clone, Debug)]
pub struct K8sOptions {
    /// The address or URL of the machine to connect to. Should include the port if so.
    pub cluster_address  : Address,
    /// The address of the Docker registry that we push container images to.
    pub registry_address : Address,
    /// The path to the Kubernetes config file to connect with.
    pub config           : PathBuf,
}





/***** LIBRARY *****/
/// Represents a client for a Kubernetes cluster. Practically acts as a builder for a connection.
#[derive(Clone)]
pub struct Client {
    /// A Kubernetes config to wrap around.
    client : kube::Client,
}

impl Client {
    /// Constructor for the Client.
    /// 
    /// # Arguments
    /// - `config`: The [`Config`] that we use to known to which cluster to connect and how.
    /// 
    /// # Returns
    /// A new Client instance that can be used to connect to the cluster described in the given config.
    /// 
    /// # Errors
    /// This function errors if we failed to create a [`kube::Client`] from the given `config`.
    #[inline]
    pub fn new(config: impl Into<Config>) -> Result<Self, ClientError> {
        // Attempt to create a client from the given config
        let client: kube::Client = match kube::Client::try_from(config.into()) {
            Ok(client) => client,
            Err(err)   => { return Err(ClientError::CreateClient{ err }); },
        };

        // Return ourselves with the client
        Ok(Self {
            client,
        })
    }

    /// Constructor for the Client that parses the Kubernetes config from the given path.
    /// 
    /// # Arguments
    /// - `path`: The [`Path`]-like to parse the Kubernetes config from.
    /// 
    /// # Returns
    /// A new Client instance that can be used to connect to the cluster described in the given config.
    /// 
    /// # Errors
    /// This function may error if we failed to parse the given file or if we failed to create a [`kube::Client`] from the given `config`.
    pub async fn from_path_async(path: impl AsRef<Path>) -> Result<Self, ClientError> {
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
            api : Api::namespaced(self.client.clone(), namespace.as_ref()),
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
pub struct Connection<R> {
    /// The [`API`] abstraction with which we connect.
    api : Api<R>,
}

impl<R> Connection<R> {
    
}
