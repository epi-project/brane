//  POLICIES.rs
//    by Lut99
//
//  Created:
//    10 Jan 2024, 15:57:54
//  Last edited:
//    11 Jan 2024, 13:57:03
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements handlers for subcommands to `branectl policies ...`
//

use std::error;
use std::ffi::OsStr;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::PathBuf;
use std::time::Duration;

use brane_cfg::info::Info;
use brane_cfg::node::{NodeConfig, NodeSpecificConfig, WorkerConfig};
use brane_shr::formatters::BlockFormatter;
use console::style;
use enum_debug::EnumDebug;
use error_trace::trace;
use log::{debug, info};
use rand::distributions::Alphanumeric;
use rand::Rng;
use reqwest::{Client, Request, Response, StatusCode};
use serde_json::value::RawValue;
use srv::models::{AddPolicyPostModel, PolicyContentPostModel};
use tokio::fs::{self as tfs, File as TFile};

use crate::spec::PolicyInputLanguage;


/***** ERRORS *****/
/// Defines errors that may originate in `branectl policies ...` subcommands.
#[derive(Debug)]
pub enum Error {
    /// Failed to deserialize the read input file as JSON.
    InputDeserialize { path: PathBuf, raw: String, err: serde_json::Error },
    /// Failed to read the input file.
    InputRead { path: PathBuf, err: std::io::Error },
    /// Failed to compile the input file to eFLINT JSON.
    InputToJson { path: PathBuf, err: eflint_to_json::Error },
    /// A policy language was attempted to derive from a path without extension.
    MissingExtension { path: PathBuf },
    /// The given node config file was not a worker config file.
    NodeConfigIncompatible { path: PathBuf, got: String },
    /// Failed to load the node configuration file for this node.
    NodeConfigLoad { path: PathBuf, err: brane_cfg::info::YamlError },
    /// Failed to build a request.
    RequestBuild { addr: String, err: reqwest::Error },
    /// A request failed for some reason.
    RequestFailure { addr: String, code: StatusCode, response: Option<String> },
    /// Failed to send a request.
    RequestSend { addr: String, err: reqwest::Error },
    /// Failed to create a temporary file.
    TempFileCreate { path: PathBuf, err: std::io::Error },
    /// Failed to write to a temporary file from stdin.
    TempFileWrite { path: PathBuf, err: std::io::Error },
    /// Failed to generate a new token.
    TokenGenerate { secret: PathBuf, err: specifications::policy::Error },
    /// A policy language was attempted to derive from the extension but we didn't know it.
    UnknownExtension { path: PathBuf, ext: String },
    /// The policy was given on stdout but no language was specified.
    UnspecifiedInputLanguage,
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            InputDeserialize { path, raw, .. } => {
                write!(f, "Failed to deserialize contents of '{}' to JSON\n\nRaw value:\n{}\n", path.display(), BlockFormatter::new(raw))
            },
            InputRead { path, .. } => write!(f, "Failed to read input file '{}'", path.display()),
            InputToJson { path, .. } => write!(f, "Failed to compile input file '{}' to eFLINT JSON", path.display()),
            MissingExtension { path } => {
                write!(f, "Cannot derive input language from '{}' that has no extension; manually specify it using '--language'", path.display())
            },
            NodeConfigIncompatible { path, got } => {
                write!(f, "Given node configuration file '{}' is for a {} node, but expected a Worker node", path.display(), got)
            },
            NodeConfigLoad { path, .. } => write!(f, "Failed to load node configuration file '{}'", path.display()),
            RequestBuild { addr, .. } => write!(f, "Failed to build new POST-request to '{addr}'"),
            RequestFailure { addr, code, response } => write!(
                f,
                "Request to '{}' failed with status {} ({}){}",
                addr,
                code.as_u16(),
                code.canonical_reason().unwrap_or("???"),
                if let Some(response) = response { format!("\n\nResponse:\n{}\n", BlockFormatter::new(response)) } else { String::new() }
            ),
            RequestSend { addr, .. } => write!(f, "Failed to send POST-request to '{addr}'"),
            TempFileCreate { path, .. } => write!(f, "Failed to create temporary file '{}'", path.display()),
            TempFileWrite { path, .. } => write!(f, "Failed to copy stdin to temporary file '{}'", path.display()),
            TokenGenerate { secret, .. } => write!(
                f,
                "Failed to generate one-time authentication token from secret file '{}' (you can manually specify a token using '--token')",
                secret.display()
            ),
            UnknownExtension { path, ext } => write!(
                f,
                "Cannot derive input language from '{}' that has unknown extension '{}'; manually specify it using '--language'",
                path.display(),
                ext
            ),
            UnspecifiedInputLanguage => write!(f, "Cannot derive input language when giving input via stdin; manually specify it using '--language'"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            InputDeserialize { err, .. } => Some(err),
            InputRead { err, .. } => Some(err),
            InputToJson { err, .. } => Some(err),
            MissingExtension { .. } => None,
            NodeConfigIncompatible { .. } => None,
            NodeConfigLoad { err, .. } => Some(err),
            RequestBuild { err, .. } => Some(err),
            RequestFailure { .. } => None,
            RequestSend { err, .. } => Some(err),
            TempFileCreate { err, .. } => Some(err),
            TempFileWrite { err, .. } => Some(err),
            TokenGenerate { err, .. } => Some(err),
            UnknownExtension { .. } => None,
            UnspecifiedInputLanguage => None,
        }
    }
}





/***** AUXILLARY *****/
/// Defines supported reasoners in the checker.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TargetReasoner {
    /// It's an eFLINT JSON Specification reasoner
    EFlintJson(EFlintJsonVersion),
}
impl TargetReasoner {
    /// Returns the string identifier of the reasoner that can be send to a checker.
    ///
    /// # Returns
    /// A [`String`] that the checker uses to verify if the sent policy matches the backend.
    pub fn id(&self) -> String {
        match self {
            Self::EFlintJson(_) => "eflint-json".into(),
        }
    }

    /// Returns the string identifier of the reasoner version that can be send to a checker.
    ///
    /// # Returns
    /// A [`String`] version that the checker uses to verify if the sent policy matches the backend.
    pub fn version(&self) -> String {
        match self {
            Self::EFlintJson(v) => v.to_string(),
        }
    }
}

/// Defines supported [`TargetReasoner::EFlintJson`] specification versions.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EFlintJsonVersion {
    /// Specification version 0.1.0 (see <https://gitlab.com/eflint/json-specification/-/releases/v0.1.0>).
    V0_1_0,
}
impl Display for EFlintJsonVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        match self {
            Self::V0_1_0 => write!(f, "0.1.0"),
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
        debug!("Using given token '{token}'");
        token
    } else {
        // Attempt to generate a new token based on the secret in the `node.yml` file
        match specifications::policy::generate_policy_token(
            names::three::lowercase::rand(),
            "branectl",
            Duration::from_secs(60),
            &worker.paths.policy_expert_secret,
        ) {
            Ok(token) => {
                debug!("Using generated token '{token}'");
                token
            },
            Err(err) => return Err(Error::TokenGenerate { secret: worker.paths.policy_expert_secret, err }),
        }
    };

    // Next stop: resolve the input to a path to read from
    let (input, from_stdin): (PathBuf, bool) = if input == "-" {
        // Create a temporary file to write stdin to
        let id: String = rand::thread_rng().sample_iter(Alphanumeric).take(4).map(char::from).collect::<String>();
        let temp_path: PathBuf = std::env::temp_dir().join(format!("branectl-stdin-{id}.txt"));
        debug!("Writing stdin to temporary file '{}'...", temp_path.display());
        let mut temp: TFile = match TFile::create(&temp_path).await {
            Ok(temp) => temp,
            Err(err) => return Err(Error::TempFileCreate { path: temp_path, err }),
        };

        // Perform the write
        if let Err(err) = tokio::io::copy(&mut tokio::io::stdin(), &mut temp).await {
            return Err(Error::TempFileWrite { path: temp_path, err });
        }

        // Done
        (temp_path, true)
    } else {
        (input.into(), false)
    };

    // If the language is not given, resolve it from the file extension
    let language: PolicyInputLanguage = if let Some(language) = language {
        debug!("Interpreting input as {language}");
        language
    } else if let Some(ext) = input.extension() {
        debug!("Attempting to derive input language from extension '{}' (part of '{}')", ext.to_string_lossy(), input.display());

        // Else, attempt to resolve from the extension
        if ext == OsStr::new("eflint") {
            PolicyInputLanguage::EFlint
        } else if ext == OsStr::new("json") {
            PolicyInputLanguage::EFlintJson
        } else if from_stdin {
            return Err(Error::UnspecifiedInputLanguage);
        } else {
            let ext: String = ext.to_string_lossy().into();
            return Err(Error::UnknownExtension { path: input, ext });
        }
    } else if from_stdin {
        return Err(Error::UnspecifiedInputLanguage);
    } else {
        return Err(Error::MissingExtension { path: input });
    };

    // Read the input file
    let (json, target_reasoner): (String, TargetReasoner) = match language {
        PolicyInputLanguage::EFlint => {
            // We read it as eFLINT to JSON
            debug!("Compiling eFLINT input file '{}' to eFLINT JSON", input.display());
            let mut json: Vec<u8> = Vec::new();
            if let Err(err) = eflint_to_json::compile_async(&input, &mut json, None).await {
                return Err(Error::InputToJson { path: input, err });
            }

            // Serialize it to a string
            match String::from_utf8(json) {
                Ok(json) => (json, TargetReasoner::EFlintJson(EFlintJsonVersion::V0_1_0)),
                Err(err) => panic!("{}", trace!(("eflint_to_json::compile_async() did not return valid UTF-8"), err)),
            }
        },
        PolicyInputLanguage::EFlintJson => {
            // Read the file in one go
            debug!("Reading eFLINT JSON input file '{}'", input.display());
            match tfs::read_to_string(&input).await {
                Ok(json) => (json, TargetReasoner::EFlintJson(EFlintJsonVersion::V0_1_0)),
                Err(err) => return Err(Error::InputRead { path: input, err }),
            }
        },
    };

    // Ensure it is JSON
    debug!("Deserializing input as JSON...");
    let json: Box<RawValue> = match serde_json::from_str(&json) {
        Ok(json) => json,
        Err(err) => return Err(Error::InputDeserialize { path: input, raw: json, err }),
    };

    // Finally, construct a request for the checker
    let url: String = format!("{}/v1/policies", worker.services.chk.address);
    debug!("Building POST-request to '{url}'...");
    let client: Client = Client::new();
    let contents: AddPolicyPostModel = AddPolicyPostModel {
        version_description: "".into(),
        description: None,
        content: vec![PolicyContentPostModel { reasoner: target_reasoner.id(), reasoner_version: target_reasoner.version(), content: json }],
    };
    let req: Request = match client.post(&url).bearer_auth(token).json(&contents).build() {
        Ok(req) => req,
        Err(err) => return Err(Error::RequestBuild { addr: url, err }),
    };

    // Now send it!
    debug!("Sending request to '{url}'...");
    let res: Response = match client.execute(req).await {
        Ok(res) => res,
        Err(err) => return Err(Error::RequestSend { addr: url, err }),
    };
    debug!("Server responded with {}", res.status());
    if !res.status().is_success() {
        return Err(Error::RequestFailure { addr: url, code: res.status(), response: res.text().await.ok() });
    }

    // Log the response body
    if let Ok(res) = res.text().await {
        debug!("Response:\n{}\n", BlockFormatter::new(res));
    }

    // Done!
    println!(
        "Successfully added policy {} to checker of node {}.",
        style(if from_stdin { "<stdin>".into() } else { input.display().to_string() }).bold().green(),
        style(worker.name).bold().green()
    );
    Ok(())
}
