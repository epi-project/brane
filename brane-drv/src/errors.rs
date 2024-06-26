//  ERRORS.rs
//    by Lut99
//
//  Created:
//    01 Feb 2022, 16:13:53
//  Last edited:
//    08 Feb 2024, 16:49:47
//  Auto updated?
//    Yes
//
//  Description:
//!   Contains errors used within the brane-drv package only.
//

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::PathBuf;


/***** ERRORS *****/
/// Defines errors that relate to the RemoteVm.
#[derive(Debug)]
pub enum RemoteVmError {
    /// Failed to plan a workflow.
    PlanError { err: brane_tsk::errors::PlanError },
    /// Failed to run a workflow.
    ExecError { err: brane_exe::Error },

    /// The given node config was not for this type of node.
    IllegalNodeConfig { path: PathBuf, got: String },
    /// Failed to load the given infra file.
    InfraFileLoad { path: PathBuf, err: brane_cfg::info::YamlError },
    /// Failed to load the given node config file.
    NodeConfigLoad { path: PathBuf, err: brane_cfg::info::YamlError },
}

impl Display for RemoteVmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use RemoteVmError::*;
        match self {
            PlanError { .. } => write!(f, "Failed to plan workflow"),
            ExecError { .. } => write!(f, "Failed to execute workflow"),

            IllegalNodeConfig { path, got } => {
                write!(f, "Illegal node config kind in node config '{}'; expected Central, got {}", path.display(), got)
            },
            InfraFileLoad { path, .. } => write!(f, "Failed to load infra file '{}'", path.display()),
            NodeConfigLoad { path, .. } => write!(f, "Failed to load node config file '{}'", path.display()),
        }
    }
}

impl Error for RemoteVmError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use RemoteVmError::*;
        match self {
            PlanError { err } => Some(err),
            ExecError { err } => Some(err),

            IllegalNodeConfig { .. } => None,
            InfraFileLoad { err, .. } => Some(err),
            NodeConfigLoad { err, .. } => Some(err),
        }
    }
}
