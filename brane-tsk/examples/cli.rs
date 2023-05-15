//  CLI.rs
//    by Lut99
// 
//  Created:
//    15 May 2023, 11:15:47
//  Last edited:
//    15 May 2023, 11:52:02
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
use log::{error, info};

use brane_shr::errors::ErrorTrace as _;
use specifications::address::Address;
use specifications::container::Image;

use brane_tsk::docker::ImageSource;
use brane_tsk::k8s::resolve_image_source;


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
                if let Err(err) = resolve_image_source(push.tag.clone(), ImageSource::Path(push.path.clone()), push.registry.clone()).await { error!("{}", err.trace()); std::process::exit(1); }
                println!("Successfully pushed image {} to {}", style(push.path.display()).bold().blue(), style(format!("{}/v2/{}:{}", push.registry, push.tag.name, push.tag.version.unwrap())).bold().blue());
            },
        },
    }

    // Done!
}
