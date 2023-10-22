//  CAPABILITIES.rs
//    by Lut99
// 
//  Created:
//    19 Jun 2023, 09:50:46
//  Last edited:
//    19 Jun 2023, 09:55:44
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the capabilities supported by BRANE backends.
// 

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::str::FromStr;

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};


/***** ERRORS *****/
/// Lists the error for parsing a Capability from a string.
#[derive(Debug)]
pub enum ParseError {
    /// An unknown capability was given.
    UnknownCapability{ raw: String },
}
impl Display for ParseError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ParseError::*;
        match self {
            UnknownCapability{ raw } => write!(f, "Unknown capability '{raw}'"),
        }
    }
}
impl Error for ParseError {}





/***** LIBRARY *****/
/// Defines if the package has any additional requirements on the system it will run.
#[derive(Clone, Copy, Deserialize, EnumDebug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// The package requires access to a CUDA GPU
    CudaGpu,
}

impl std::fmt::Debug for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Capability::*;
        match self {
            CudaGpu => write!(f, "cuda_gpu"),
        }
    }
}
impl FromStr for Capability {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cuda_gpu" | "cuda-gpu" => Ok(Self::CudaGpu),

            _ => Err(ParseError::UnknownCapability{ raw: s.into() }),
        }
    }
}

impl AsRef<Capability> for Capability {
    #[inline]
    fn as_ref(&self) -> &Self { self }
}
impl AsMut<Capability> for Capability {
    #[inline]
    fn as_mut(&mut self) -> &mut Capability { self }
}
impl From<&Capability> for Capability {
    #[inline]
    fn from(value: &Capability) -> Self { *value }
}
impl From<&mut Capability> for Capability {
    #[inline]
    fn from(value: &mut Capability) -> Self { *value }
}
