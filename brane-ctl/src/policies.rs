//  POLICIES.rs
//    by Lut99
//
//  Created:
//    10 Jan 2024, 15:57:54
//  Last edited:
//    10 Jan 2024, 17:00:57
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements handlers for subcommands to `branectl policies ...`
//

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::{Path, PathBuf};
use std::time::Duration;

use brane_cfg::info::Info;
use brane_cfg::node::{NodeConfig, NodeSpecificConfig, WorkerConfig};
use enum_debug::EnumDebug;
use log::{debug, info};
use srv::models::AddPolicyPostModel;

use crate::spec::PolicyInputLanguage;


/***** ERRORS *****/
/// Defines errors that may originate in `branectl policies ...` subcommands.
#[derive(Debug)]
pub enum Error {
    /// The given node config file was not a worker config file.
    NodeConfigIncompatible { path: PathBuf, got: String },
    /// Failed to load the node configuration file for this node.
    NodeConfigLoad { path: PathBuf, err: brane_cfg::info::YamlError },
    /// Failed to generate a new token.
    TokenGenerate { secret: PathBuf, err: specifications::policy::Error },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            NodeConfigIncompatible { path, got } => {
                write!(f, "Given node configuration file '{}' is for a {} node, but expected a Worker node", path.display(), got)
            },
            NodeConfigLoad { path, .. } => write!(f, "Failed to load node configuration file '{}'", path.display()),
            TokenGenerate { secret, .. } => write!(
                f,
                "Failed to generate one-time authentication token from secret file '{}' (you can manually specify a token using '--token')",
                secret.display()
            ),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            NodeConfigIncompatible { .. } => None,
            NodeConfigLoad { err, .. } => Some(err),
            TokenGenerate { err, .. } => Some(err),
        }
    }
}





/***** LIBRARY *****/
/// Adds the given policy to the checker defined in the given node config file.
///
/// # Arguments
/// - `node_config_path`: The path to the node configuration file that determines which node we're working for.
/// - `input`: The policy (or rather, a path thereto) to submit.
/// - `language`: The language of the input.
/// - `token`: A token used for authentication with the remote checker. If omitted, will attempt to generate one based on the secret file in the node.yml file.
pub async fn add(node_config_path: PathBuf, input: String, token: Option<String>, language: Option<PolicyInputLanguage>) -> Result<(), Error> {
    info!("Adding policy '{}' to checker of node defined by '{}'", input, node_config_path.display());

    // First, load the node config file
    debug!("Loading node configuration file '{}'...", node_config_path.display());
    let node: NodeConfig = match NodeConfig::from_path_async(&node_config_path).await {
        Ok(node) => node,
        Err(err) => return Err(Error::NodeConfigLoad { path: node_config_path, err }),
    };
    // Assert it's of the correct type
    let worker: WorkerConfig = match node.node {
        NodeSpecificConfig::Worker(worker) => worker,
        other => return Err(Error::NodeConfigIncompatible { path: node_config_path, got: other.variant().to_string() }),
    };

    // Then see if we need to resolve the token
    let token: String = if let Some(token) = token {
        token
    } else {
        // Attempt to generate a new token based on the secret in the `node.yml` file
        match specifications::policy::generate_policy_token(
            names::three::lowercase::rand(),
            "branectl",
            Duration::from_secs(60),
            &worker.paths.policy_expert_secret,
        ) {
            Ok(token) => token,
            Err(err) => return Err(Error::TokenGenerate { secret: worker.paths.policy_expert_secret, err }),
        }
    };

    // Next, construct a request for the checker
    let req: AddPolicyPostModel = AddPolicyPostModel { version_description: "".into(), description: None, content: vec![] };

    // Done!
    Ok(())
}
