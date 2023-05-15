//  K 8S.rs
//    by Lut99
// 
//  Created:
//    08 May 2023, 13:01:23
//  Last edited:
//    15 May 2023, 16:53:44
//  Auto updated?
//    Yes
// 
//  Description:
//!   Provides an API for running Brane tasks on a Kubernetes backend.
// 

use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::str::FromStr as _;

pub use kube::Config;
pub use k8s_openapi::api::core::v1::Pod;
pub use k8s_openapi::api::batch::v1::Job;
use enum_debug::EnumDebug;
use hex_literal::hex;
use k8s_openapi::NamespaceResourceScope;
use kube::api::{Api, PostParams, Resource};
use kube::config::{Kubeconfig, KubeConfigOptions};
use log::{debug, info, warn};
use rand::Rng as _;
use rand::distributions::Alphanumeric;
use serde_json::json;
use tokio::fs as tfs;

use brane_shr::fs::{download_file_async, set_executable, unarchive_async, DownloadSecurity};
use specifications::address::Address;
use specifications::container::Image;
use specifications::version::Version;

use crate::docker::ImageSource;


/***** TESTS *****/
#[cfg(test)]
mod tests {
    use brane_shr::errors::ErrorTrace as _;
    use super::*;

    /// Function that tests downloading the crane executable from the internet.
    /// 
    /// Essentially just checks if everything proceeds without errors, and if we can then call '--version' on it.
    #[tokio::test]
    async fn test_crane_download() {
        // Prepare a temporary directory
        if let Err(err) = tfs::create_dir_all("./temp").await { panic!("Failed to create temporary directory './temp': {err}"); }

        // Ensure the executable exists
        if let Err(err) = ensure_crane_exe("./temp/crane", "./temp").await {
            if let Err(err) = tfs::remove_dir_all("./temp").await { warn!("Failed to cleanup temporary directory './temp': {err}"); }
            panic!("Failed to ensure crane executable: {}", err.trace());
        }

        // Attempt to run `--version`
        let mut cmd: Command = Command::new("./temp/crane");
        cmd.arg("version");
        match cmd.status() {
            Ok(status) => if !status.success() {
                if let Err(err) = tfs::remove_dir_all("./temp").await { warn!("Failed to cleanup temporary directory './temp': {err}"); }
                panic!("Failed to run './temp/crane' (see output above)");
            },
            Err(err)   => {
                if let Err(err) = tfs::remove_dir_all("./temp").await { warn!("Failed to cleanup temporary directory './temp': {err}"); }
                panic!("Failed to spawn job './temp/crane': {err}");
            },
        }

        // Done! Cleanup
        if let Err(err) = tfs::remove_dir_all("./temp").await { warn!("Failed to cleanup temporary directory './temp': {err}"); }
    }
}





/***** CONSTANTS *****/
/// Defines the address we download the x86-64 `crane` tar from.
pub const CRANE_TAR_URL_X86_64: &'static str = "https://github.com/google/go-containerregistry/releases/download/v0.15.1/go-containerregistry_Linux_x86_64.tar.gz";
/// Defines the address we download the ARM64 `crane` tar from.
pub const CRANE_TAR_URL_ARM64: &'static str = "https://github.com/google/go-containerregistry/releases/download/v0.15.1/go-containerregistry_Linux_arm64.tar.gz";

/// The location where we expect the `crane` executable to be, locally.
pub const CRANE_PATH: &'static str = "/tmp/crane";
/// The checksum of the executable.
pub const CRANE_TAR_CHECKSUM: [u8; 32] = hex!("d4710014a3bd135eb1d4a9142f509cfd61d2be242e5f5785788e404448a4f3f2");





/***** ERRORS *****/
/// Defines errors that occur when downloading the `crane` executable.
#[derive(Debug)]
pub enum CraneError {
    /// Failed to download the `crane` executable tarball.
    DownloadCraneTar{ from: &'static str, to: PathBuf, err: brane_shr::fs::Error },
    /// Failed to unpack the `crane` executable tarball.
    UnpackCraneTar{ from: PathBuf, to: PathBuf, err: brane_shr::fs::Error },
    /// Failed to move the `crane` executable from the downloaded folder to the target path.
    MoveCrane{ from: PathBuf, to: PathBuf, err: std::io::Error },
    /// Failed to make the `crane` executable... executable.
    MakeCraneExecutable{ path: PathBuf, err: brane_shr::fs::Error },
}
impl Display for CraneError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use CraneError::*;
        match self {
            DownloadCraneTar{ from, to, .. } => write!(f, "Failed to download tarball from '{}' to '{}'", from, to.display()),
            UnpackCraneTar{ from, to, .. }   => write!(f, "Failed to unpack tarball '{}' to '{}'", from.display(), to.display()),
            MoveCrane{ from, to, .. }        => write!(f, "Failed to move executable from '{}' to '{}'", from.display(), to.display()),
            MakeCraneExecutable{ path, .. }  => write!(f, "Failed to make executable '{}' executable", path.display()),
        }
    }
}
impl Error for CraneError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use CraneError::*;
        match self {
            DownloadCraneTar{ err, .. }    => Some(err),
            UnpackCraneTar{ err, .. }      => Some(err),
            MoveCrane{ err, .. }           => Some(err),
            MakeCraneExecutable{ err, .. } => Some(err),
        }
    }
}



/// Defines errors that occur when resolving an image source (i.e., pushing to a registry).
#[derive(Debug)]
pub enum ResolveError {
    /// Failed to download the crane client.
    CraneExe{ err: CraneError },
    /// Failed to launch the command to login to a registry.
    LaunchLogin{ what: Command, err: std::io::Error },
    /// Failed to launch the command to push the image to a remote registry.
    LaunchPush{ what: Command, err: std::io::Error },
    /// The login command was launched successfully, but failed.
    LoginFailure{ registry: Address, err: Box<Self> },
    /// The push command was launched successfully, but failed.
    PushFailure{ path: PathBuf, image: String, err: Box<Self> },
    /// Some command failed.
    CommandFailure{ what: Command, status: ExitStatus, stdout: String, stderr: String },
}
impl Display for ResolveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ResolveError::*;
        match self {
            CraneExe{ .. }                                 => write!(f, "Failed to download `crane` registry client"),
            LaunchLogin{ what, .. }                        => write!(f, "Failed to launch command '{what:?}' to login to remote registry"),
            LaunchPush{ what, .. }                         => write!(f, "Failed to launch command '{what:?}' to push image"),
            LoginFailure{ registry, .. }                   => write!(f, "Failed to login to remote registry '{registry}'"),
            PushFailure{ path, image, .. }                 => write!(f, "Failed to push image '{}' to '{}'", path.display(), image),
            CommandFailure{ what, status, stdout, stderr } => write!(f, "Command '{:?}' failed with exit code {}\n\nstdout:\n{}\n{}\n{}\n\nstderr:\n{}\n{}\n{}\n\n", what, status.code().unwrap_or(-1), (0..80).map(|_| '-').collect::<String>(), stdout, (0..80).map(|_| '-').collect::<String>(), (0..80).map(|_| '-').collect::<String>(), stderr, (0..80).map(|_| '-').collect::<String>()),
        }
    }
}
impl Error for ResolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use ResolveError::*;
        match self {
            CraneExe{ err }          => Some(err),
            LaunchLogin{ err, .. }   => Some(err),
            LaunchPush{ err, .. }    => Some(err),
            LoginFailure{ err, .. }  => Some(err),
            PushFailure{ err, .. }   => Some(err),
            CommandFailure{ .. } => None,
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
    /// Failed to spawn the Kubernetes job
    CreateJob{ name: String, version: Version, id: String, err: kube::Error },
}
impl Display for ConnectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ConnectionError::*;
        match self {
            CreateJob { name, version, id, .. } => write!(f, "Failed to launch package {}:{} as Kubernetes job with ID '{}'", name, version, id),
        }
    }
}
impl Error for ConnectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use ConnectionError::*;
        match self {
            CreateJob { err, .. } => Some(err),
        }
    }
}





/***** HELPER FUNCTIONS *****/
/// Ensures that the `crane` executable is downloaded at the given location.
/// 
/// # Arguments
/// - `path`: The path to put the `crane` executable.
/// - `temp_dir`: Some (already existing!) directory to download intermediary files and unpacking them and such.
/// 
/// # Errors
/// This function errors if we faield to find _and_ download the file.
async fn ensure_crane_exe(path: impl AsRef<Path>, temp_dir: impl AsRef<Path>) -> Result<(), CraneError> {
    let path     : &Path = path.as_ref();
    let temp_dir : &Path = temp_dir.as_ref();
    debug!("Ensuring `crate` executable exists at '{}'...", path.display());

    // Resolve where to get the executable from
    #[cfg(target_arch = "x86_64")]
    const URL: &'static str = CRANE_TAR_URL_X86_64;
    #[cfg(target_arch = "aarch64")]
    const URL: &'static str = CRANE_TAR_URL_ARM64;
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    compile_error!("Unsupported non-x86_64, non-ARM64 architecture");

    // Check if it already exists, that's nice then
    if path.exists() {
        debug!("Executable '{}' found, marked as present", path.display());
        return Ok(());
    }
    debug!("Executable '{}' not found, marked as missing", path.display());

    // Otherwise, we should attempt to download the crane executable's tarball
    let tar_path: PathBuf = temp_dir.join("go-containerregistry_Linux.tar.gz");
    debug!("Downloading '{}' to '{}'...", URL, tar_path.display());
    if let Err(err) = download_file_async(URL, &tar_path, DownloadSecurity::all(&CRANE_TAR_CHECKSUM), None).await {
        return Err(CraneError::DownloadCraneTar { from: URL, to: tar_path, err });
    }

    // Unpack the tarball
    let dir_path: PathBuf = temp_dir.join("go-containerregistry_Linux");
    debug!("Unpacking '{}' to '{}'...", tar_path.display(), dir_path.display());
    if let Err(err) = unarchive_async(&tar_path, &dir_path).await {
        return Err(CraneError::UnpackCraneTar{ from: tar_path, to: dir_path, err });
    }

    // Move the directory's crane executable to the target location
    let crane_path: PathBuf = dir_path.join("crane");
    debug!("Extracting '{}' to '{}'...", crane_path.display(), path.display());
    if let Err(err) = tfs::copy(&crane_path, path).await {
        return Err(CraneError::MoveCrane{ from: crane_path, to: path.into(), err });
    }
    // Make it executable, too
    if let Err(err) = set_executable(path) {
        return Err(CraneError::MakeCraneExecutable{ path: path.into(), err });
    }

    // Finally, delete the tar and directory
    if let Err(err) = tfs::remove_dir_all(&dir_path).await { warn!("Failed to remove extracted tarball folder '{}': {}", dir_path.display(), err); }
    if let Err(err) = tfs::remove_file(&tar_path).await { warn!("Failed to remove downloaded tarball '{}': {}", tar_path.display(), err); }

    // Done!
    debug!("Successfully downloaded `crane` executable to {}", path.display());
    Ok(())
}



/// Creates a Kubernetes job description file that we use to launch a job.
/// 
/// # Arguments
/// - `einfo`: The [`ExecuteInfo`] struct that describes the job.
/// 
/// # Returns
/// A tuple of the ID of this job, and the actual [`Job`] struct describing the job.
fn create_k8s_job(einfo: ExecuteInfo) -> (String, Job) {
    // Generate an identifier for this job.
    let id: String = format!("{}-{}-{}", einfo.image.name, einfo.image.version.map(|v| v.to_string().replace('.', "_")).unwrap_or("latest".into()), rand::thread_rng().sample_iter(Alphanumeric).take(8).map(char::from).collect::<String>());

    // Create the Job
    let job: Job = serde_json::from_value(json!({
        // Define the kind of this YAML file
        "apiVersion": "batch/v1",
        "kind": "Job",

        // Define the job ID
        "metadata": {
            "name": id,
        },

        // Now define the rest of the job
        "spec": {
            "backoffLimit": 3,
            "ttlSecondsAfterFinished": 120,
            "template": {
                "spec": {
                    "containers": [{
                        "name": id,
                        "image": einfo.image_source.into_registry(),
                        "args": einfo.command,
                        // "env": <>,
                        "securityContext": {
                            "capabilities": {
                                "drop": ["all"],
                                "add": ["SYS_TIME"],
                            },
                            "privileged": false,
                        },
                    }],
                    "restartPolicy": "Never",
                }
            }
        },
    })).unwrap();

    // Now we can generate the struct efficiently using the JSON macro.
    (id, job)
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
/// - `image`: The image to push. This will determine its name and tag in the container registry.
/// - `source`: The [`ImageSource`] to resolve.
/// - `registry`: The address of the registry to upload the image to if necessary.
/// - `auth`: Any method of authentication that we need for the registry.
/// - `insecure`: If true, we pass the '--insecure' flag to the `crane` executable (i.e., ignore SSL certs and such).
/// 
/// # Returns
/// Another [`ImageSource`] that is the resolved version of `image`.
/// 
/// # Errors
/// This function may error if the given `image` was an [`ImageSource::Path`], and we failed to upload the image.
pub async fn resolve_image_source(name: impl AsRef<Image>, source: impl Into<ImageSource>, registry: impl AsRef<Address>, auth: Option<RegistryAuth>, insecure: bool) -> Result<ImageSource, ResolveError> {
    let name     : &Image      = name.as_ref();
    let source   : ImageSource = source.into();
    let registry : &Address    = registry.as_ref();
    info!("Resolving image by maybe pushing it to a registry");

    // Only resolve if we're a local file
    let path: PathBuf = match source {
        ImageSource::Path(path)    => path,
        ImageSource::Registry(reg) => {
            debug!("Nothing to resolve (image is already a registry image; {reg})");
            return Ok(ImageSource::Registry(reg));
        },
    };
    debug!("Resolving local image '{}' to a registry image @ '{}'", path.display(), registry);

    // Deduce the path to the registry
    let address: String = format!("{registry}/v2/{}:{}", name.name, if let Some(version) = &name.version { version } else { "latest" });

    // Next, ensure the crane executable exists
    debug!("Ensuring `crane` exists...");
    if let Err(err) = ensure_crane_exe(CRANE_PATH, "/tmp").await { return Err(ResolveError::CraneExe{ err }); }

    // If there is any auth, run the command first
    match auth {
        Some(RegistryAuth::Basic(basic)) => {
            info!("Using basic auth to login to registry");

            // Prepare the login command in `crane`
            debug!("Logging in to registry as user '{}'...", basic.username);
            let mut cmd: Command = Command::new(CRANE_PATH);
            cmd.args(["auth", "login", "-u"]);
            cmd.arg(basic.username);
            cmd.arg("-p");
            cmd.arg(basic.password);
            cmd.arg(registry.to_string());
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            // Run it
            let output: Output = match cmd.output() {
                Ok(output) => output,
                Err(err)   => { return Err(ResolveError::LaunchLogin { what: cmd, err }); },
            };
            if !output.status.success() { return Err(ResolveError::LoginFailure { registry: registry.clone(), err: Box::new(ResolveError::CommandFailure { what: cmd, status: output.status, stdout: String::from_utf8_lossy(&output.stdout).into(), stderr: String::from_utf8_lossy(&output.stderr).into() }) }); }        
        },

        None => {},
    }

    // Next up, prepare to launch crane with the tarball path
    debug!("Pushing image using `crane`...");
    let mut cmd: Command = Command::new(CRANE_PATH);
    cmd.args(["push", path.display().to_string().as_str()]);
    cmd.arg(&address);
    if insecure { cmd.arg("--insecure"); }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Run it
    let output: Output = match cmd.output() {
        Ok(output) => output,
        Err(err)   => { return Err(ResolveError::LaunchPush { what: cmd, err }); },
    };
    if !output.status.success() { return Err(ResolveError::PushFailure { path, image: address, err: Box::new(ResolveError::CommandFailure { what: cmd, status: output.status, stdout: String::from_utf8_lossy(&output.stdout).into(), stderr: String::from_utf8_lossy(&output.stderr).into() }) }); }

    // Done
    Ok(ImageSource::Registry(address))
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

/// Defines methods of authentication for registries.
#[derive(Clone, Debug, EnumDebug)]
pub enum RegistryAuth {
    /// It needs a username and a password.
    Basic(BasicAuth),
}
/// Defines the Docker registry basic authentication scheme.
#[derive(Clone, Debug)]
pub struct BasicAuth {
    /// The username of the user.
    pub username : String,
    /// The password of the user.
    pub password : String,
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
    /// The [`Api`] abstraction with which we connect.
    api : Api<R>,
}

impl Connection<Job> {
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
    pub async fn spawn(&self, einfo: ExecuteInfo) -> Result<JobHandle, ConnectionError> {
        info!("Spawning package task '{}' from '{}' on Kubernetes backend", einfo.name, einfo.image);

        // Assert the container has been uploaded
        if !matches!(einfo.image_source, ImageSource::Registry(_)) { panic!("Non-Registry ImageSource must have been resolved before calling Connection::spawn"); }

        // Prepare the Kubernetes config file.
        let image: Image = einfo.image.clone();
        let (id, job): (String, Job) = create_k8s_job(einfo);

        // Submit the job
        if let Err(err) = self.api.create(&PostParams::default(), &job).await {
            return Err(ConnectionError::CreateJob{ name: image.name, version: image.version.map(|v| Version::from_str(&v).ok()).flatten().unwrap_or(Version::latest()), id, err });
        }

        // Done
        Ok(JobHandle{ connection: self, id })
    }
}



/// Represents a job that is currently running within a Kubernetes cluster.
#[derive(Debug)]
pub struct JobHandle<'c> {
    /// The connection of which we are a part.
    connection : &'c Connection<Job>,
    /// The ID of the job we manage.
    id         : String,
}
