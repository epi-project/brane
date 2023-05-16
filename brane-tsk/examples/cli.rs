//  CLI.rs
//    by Lut99
// 
//  Created:
//    15 May 2023, 11:15:47
//  Last edited:
//    16 May 2023, 17:08:28
//  Auto updated?
//    Yes
// 
//  Description:
//!   An auxillary binary that we can use to test some functionality of
//!   the worker without having to spin up a service and send it requests.
// 

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use console::style;
use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info};

use brane_shr::errors::ErrorTrace as _;
use specifications::address::Address;
use specifications::container::Image;
use specifications::package::PackageInfo;

use brane_tsk::docker::ImageSource;
use brane_tsk::k8s::{read_config_async, resolve_image_source, BasicAuth, Client, Config, ExecuteInfo, Handle, Pod, RegistryAuth, Scope, Secret};


/***** ARGUMENTS *****/
/// Defines the arguments for this helper binary.
#[derive(Debug, Parser)]
struct Arguments {
    /// Whether to enable trace debugging
    #[clap(long, global=true, help="If given, enables full logging verbosity (implies '--debug')")]
    trace : bool,
    /// Whether to enable debug debugging
    #[clap(long, global=true, help="If given, enables more verbose logging capability")]
    debug : bool,

    /// The subcommand to run
    #[clap(subcommand)]
    subcommand : CliSubcommand,
}

/// Defines the toplevel subcommands.
#[derive(Debug, Subcommand)]
enum CliSubcommand {
    /// Defines everything Kubernetes-related.
    #[clap(name = "k8s", alias = "kubernetes", about = "Groups all subcommands relating to testing Kubernetes.")]
    K8s(K8sArguments),
}

/// Defines the arguments relating to the K8s-subcommand.
#[derive(Debug, Parser)]
struct K8sArguments {
    /// The subcommand to run next.
    #[clap(subcommand)]
    subcommand : K8sSubcommand,
}
/// Defines the subcommands relating to Kubernetes.
#[derive(Debug, Subcommand)]
enum K8sSubcommand {
    /// Pushes an image to a local registry.
    #[clap(name = "push", about = "Pushes a local package (.tar file) to the given remote registry.")]
    Push(K8sPushArguments),

    /// Launches a job with the given parameters.
    #[clap(name = "launch", about = "Launches a given job on the given Kubernetes backend.")]
    Launch(K8sLaunchArguments),
}

/// Defines the arguments to push a package to a local registry.
#[derive(Debug, Parser)]
struct K8sPushArguments {
    /// Defines the image path to push.
    #[clap(name="PATH", help="The image .tar file to push to the registry.")]
    path     : PathBuf,
    /// Defines the registry address to push to.
    #[clap(name="REGISTRY", help="The address of the registry to push to.")]
    registry : Address,
    /// The tag of the image to push.
    #[clap(name="TAG", help="The tag of the image (given as '<name>:<version>') to push.")]
    tag      : Image,

    /// If given, ignores any certificates and junk when pushing containers.
    #[clap(short, long, help="If given, makes the backend image pusher ignore certificates.")]
    insecure : bool,
    /// The user's username, if using basic auth.
    #[clap(short, long, requires="password", help="If given, use a username/password pair to login to the registry. Note that this one must always appear with '--password'")]
    username : Option<String>,
    /// The user's password, if using basic auth.
    #[clap(short, long, requires="username", help="If given, use a username/password pair to login to the registry. Note that this one must always appear with '--username'")]
    password : Option<String>,
}

/// Defines the arguments to push a package to a local registry.
#[derive(Debug, Parser)]
struct K8sLaunchArguments {
    /// Defines the path to the image to launch.
    #[clap(name="IMAGE_PATH", help="The image .tar file to push to the registry.")]
    image    : PathBuf,
    /// Defines the path to the package.yml to launch.
    #[clap(name="PACKAGE_YML_PATH", help="The package.yml file that describes the container.")]
    package  : PathBuf,
    /// Defines the registry address to push to.
    #[clap(name="REGISTRY", help="The address of the registry to push to.")]
    registry : Address,

    /// Defines the path to the Kubernetes config to use to connect.
    #[clap(short, long, default_value="~/.kube/config", help="The Kubernetes config YAML file that provides which cluster to connect to and how.")]
    config   : PathBuf,
    /// If given, ignores any certificates and junk when pushing containers.
    #[clap(short, long, help="If given, makes the backend image pusher ignore certificates.")]
    insecure : bool,
    /// The user's username, if using basic auth.
    #[clap(short, long, requires="password", help="If given, use a username/password pair to login to the registry. Note that this one must always appear with '--password'")]
    username : Option<String>,
    /// The user's password, if using basic auth.
    #[clap(short, long, requires="username", help="If given, use a username/password pair to login to the registry. Note that this one must always appear with '--username'")]
    password : Option<String>,
}





/***** ENTRYPOINT *****/
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Parse the CLI arguments
    let args: Arguments = Arguments::parse();

    // Setup the logger
    if let Err(err) = HumanLogger::terminal(DebugMode::from_flags(args.trace, args.debug)).init() {
        eprintln!("WARNING: Failed to setup logger: {err} (no logging enabled for this session)");
    }
    info!("Initializing {} cli v{}...", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    // Match on the subcommand
    match args.subcommand {
        CliSubcommand::K8s(k8s) => match k8s.subcommand {
            K8sSubcommand::Push(push) => {
                info!("Pushing {} to {}...", push.path.display(), push.registry);

                // Deduce the auth method from the input
                let auth: Option<RegistryAuth> = match (push.username, push.password) {
                    (Some(username), Some(password)) => Some(RegistryAuth::Basic(BasicAuth{ username, password })),
                    (None, None)                     => None,

                    // Anything else should never occur
                    _ => { unreachable!(); },
                };

                // Push the image
                let source: ImageSource = match resolve_image_source(&push.tag, ImageSource::Path(push.path.clone()), &push.registry, auth, push.insecure).await {
                    Ok(source) => source,
                    Err(err)   => { error!("{}", err.trace()); std::process::exit(1); },
                };

                // Done!
                println!("Successfully pushed image {} to {}", style(push.path.display()).bold().blue(), style(source.into_registry()).bold().blue());
            },

            K8sSubcommand::Launch(launch) => {
                info!("Launching image {} (package {}) to cluster through registry {}", launch.image.display(), launch.package.display(), launch.registry);

                // Deduce the auth method from the input
                let auth: Option<RegistryAuth> = match (launch.username, launch.password) {
                    (Some(username), Some(password)) => Some(RegistryAuth::Basic(BasicAuth{ username, password })),
                    (None, None)                     => None,

                    // Anything else should never occur
                    _ => { unreachable!(); },
                };

                // Load the Kubernetes config file
                let config_path: PathBuf = shellexpand::tilde(&launch.config.to_string_lossy()).as_ref().into();
                debug!("Loading Kubernetes config file '{}'...", config_path.display());
                let config: Config = match read_config_async(&config_path).await {
                    Ok(config) => config,
                    Err(err)   =>{ error!("{}", err.trace()); std::process::exit(1); },
                };

                // Load the package YAML
                debug!("Loading package.yml '{}'...", launch.package.display());
                let package: PackageInfo = match PackageInfo::from_path(launch.package) {
                    Ok(package) => package,
                    Err(err)    => { error!("{}", err.trace()); std::process::exit(1); },
                };

                // Attempt to resolve the image file
                debug!("Resolving image source '{}'...", launch.image.display());
                let image: Image = Image::new(&package.name, Some(&package.version), None::<String>);
                let source: ImageSource = match resolve_image_source(&image, ImageSource::Path(launch.image.clone()), launch.registry.clone(), auth.clone(), launch.insecure).await {
                    Ok(source) => source,
                    Err(err)   => { error!("{}", err.trace()); std::process::exit(1); },
                };

                // Now connect to the cluster
                debug!("Connecting to cluster...");
                let client: Client = match Client::new(config) {
                    Ok(client) => client,
                    Err(err)   => { error!("{}", err.trace()); std::process::exit(1); },
                };

                // Create a secret for the registry
                let secret: Option<Handle<Secret>> = match auth {
                    Some(auth) => {
                        // Attempt to create the secret
                        debug!("Creating Docker registry credential secret...");
                        let scope: Scope<Secret> = client.scope("default");
                        match scope.create_registry_secret(launch.registry, auth).await {
                            Ok(handle) => Some(handle),
                            Err(err)   => { error!("{}", err.trace()); std::process::exit(1); },
                        }
                    },

                    None => None,
                };

                // Launch the job!
                debug!("Spawning job...");
                let scope: Scope<Pod> = client.scope("default");
                let handle: Handle<Pod> = match scope.spawn(ExecuteInfo {
                    image,
                    image_source : source,

                    command : vec![],
                }, secret.as_ref()).await {
                    Ok(handle) => handle,
                    Err(err)   => { error!("{}", err.trace()); std::process::exit(1); },
                };

                // Detach the job, since this command only launches it
                println!("Launched package {}{} (as pod '{}')", style(package.name).bold().blue(), if !package.version.is_latest() { style(package.version).bold().blue().to_string() } else { String::new() }, handle.detach());

                // Destroy the secret, since we won't need it anymore
                if let Some(secret) = secret { if let Err(err) = secret.terminate().await { error!("{}", err.trace()); std::process::exit(1); } }
            },
        },
    }

    // Done!
}
