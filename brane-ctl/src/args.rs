//  ARGS.rs
//    by Lut99
//
//  Created:
//    04 Jan 2024, 13:27:44
//  Last edited:
//    04 Jan 2024, 13:48:35
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines arguments to the `branectl`-executable.
//

use std::path::PathBuf;

use brane_tsk::docker::ClientVersion;
use clap::{Parser, Subcommand};
use lazy_static::lazy_static;


/***** STATICS *****/
lazy_static! {
    /// The default Docker client API version suggested by bollard
    pub static ref API_DEFAULT_VERSION: String = bollard::API_DEFAULT_VERSION.to_string();
}





/***** AUXILLARY *****/





/***** LIBRARY *****/
/// Defines the direct arguments to the `branectl`-executable.
#[derive(Debug, Parser)]
pub struct Arguments {
    /// If given, prints `info` and `debug` prints.
    #[clap(long, global = true, help = "If given, prints additional information during execution (DEBUG- and INFO-levels).")]
    pub debug: bool,
    /// If given, prints `info`, `debug` and `trace` prints.
    #[clap(long, global = true, help = "If given, prints maximum information during execution (TRACE-, DEBUG- and INFO-levels; implies '--debug').")]
    pub trace: bool,

    /// The subcommand to run then.
    #[clap(subcommand)]
    pub action: Subcommands,
}

/// Defines the toplevel actions that can be taken.
#[derive(Debug, Subcommand)]
pub enum Subcommands {
    // Action groups
    #[clap(name = "generate", alias = "gen", about = "Groups commands to generate configuration for a (new) node.")]
    Generate(GenerateArguments),

    // Concrete actions
    /// Starts a node instance.
    #[clap(name = "start", alias = "up", about = "Starts a node described by a given `node.yml` file.")]
    Start(StartArguments),
}



/// Defines the arguments to `branectl generate ...`.
#[derive(Debug, Parser)]
pub struct GenerateArguments {}



/// Defines the arguments to `branectl start ...`.
#[derive(Debug, Parser)]
pub struct StartArguments {
    /// The node to start.
    #[clap(short, long, default_value = "./node.yml", help = "The location of the `node.yml` that describes the node we are starting.")]
    node: PathBuf,

    /// The Docker Compose executable to run.
    #[clap(
        short = 'c',
        long,
        default_value = "docker compose",
        help = "The command used to access the installed Docker Compose. Will be parsed using default shell rules (e.g., spaces are meaningful, you \
                can use `~`, etc)."
    )]
    docker_compose_command: String,
    /// The Docker Compose file to run.
    #[clap(short = 'f', long, help = "If given, uses a custom Docker Compose file instead of the default one (see `branectl extract compose`).")]
    docker_compose_file: Option<PathBuf>,
    /// The Docker socket to use to talk to the Docker client for loading/pulling images.
    #[clap(
        short = 's',
        long,
        default_value = "/var/run/docker.sock",
        help = "The location of the Docker socket used to talk to the backend Docker daemon. This is used to load/pull the required images."
    )]
    docker_socket: PathBuf,
    /// The Docker client version to use to interact with.
    #[clap(short = 'v', long, default_value = API_DEFAULT_VERSION.as_str(), help = "The Docker API version to use to talk to the backend Docker deamon.")]
    docker_version: ClientVersion,
}
