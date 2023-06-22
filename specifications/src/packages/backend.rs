//  BACKEND.rs
//    by Lut99
// 
//  Created:
//    20 Jun 2023, 17:11:50
//  Last edited:
//    22 Jun 2023, 08:48:56
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the version of the package info that is back-facing. Most
//!   importantly, it doesn't need any information about how to build the
//!   container, since this has already been done; instead, it contains
//!   information on how to run it.
// 

use std::collections::{HashMap, HashSet};

use chrono::DateTime;
use chrono_tz::Local;
use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};

use brane_shr::info::JsonInfo;
use brane_shr::serialize::Identifier;

use crate::capabilities::Capability;
use super::common::{Class, Function, PackageKind, PackageMetadata};


/***** LIBRARY *****/
/// Defines the PackageInfo alternative for the backend.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageInfo {
    /// What we know about this package that is implementation-agnostic (e.g., name, version, ...)
    #[serde(flatten)]
    pub metadata : PackageMetadata,
    /// Defines when this package was created.
    pub created  : DateTime<Local>,

    /// Defines the functions, each of which define kind-specific implementation details that we need to know to launch the package.
    pub functions : HashMap<Identifier, Function<FunctionImplementation>>,
    /// Defines the functions, each of which define kind-specific implementation details that we need to know to launch the package.
    pub classes   : HashMap<Identifier, Class<FunctionImplementation>>,
}
impl JsonInfo for PackageInfo {}



/// Defines what we need to know per package type public-facing.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FunctionImplementation {
    /// It's a container.
    Ecu(FunctionEcu),
    /// It's a BraneScript/Bakery package.
    Dsl(FunctionDsl),
    /// It's a CWL package.
    Cwl(FunctionCwl),
}
impl FunctionImplementation {
    /// Returns an enum that can be used to represent the kind of this info.
    /// 
    /// # Returns
    /// A [`PackageKind`] that represents the kind of this info.
    #[inline]
    pub fn kind(&self) -> PackageKind {
        use FunctionImplementation::*;
        match self {
            Ecu(_) => PackageKind::Ecu,
            Dsl(_) => PackageKind::Dsl,
            Cwl(_) => PackageKind::Cwl,
        }
    }

    /// Returns if this PackageSpecificInfo is a [`PackageSpecificInfo::Ecu`].
    /// 
    /// # Returns
    /// True if it is, or false otherwise.
    #[inline]
    pub fn is_ecu(&self) -> bool { matches!(self, Self::Ecu(_)) }
    /// Provides quick immutable access to the internal info as if this was a [`PackageSpecificInfo::Ecu`].
    /// 
    /// # Returns
    /// A reference to the internal [`FunctionEcu`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Ecu`].
    #[track_caller]
    #[inline]
    pub fn ecu(&self) -> &FunctionEcu { if let Self::Ecu(ecu) = self { ecu } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Ecu", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`PackageSpecificInfo::Ecu`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`FunctionEcu`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Ecu`].
    #[track_caller]
    #[inline]
    pub fn ecu_mut(&mut self) -> &mut FunctionEcu { if let Self::Ecu(ecu) = self { ecu } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Ecu", self.variant()); } }
    /// Returns the internal info as if this was a [`PackageSpecificInfo::Ecu`].
    /// 
    /// # Returns
    /// The internal [`FunctionEcu`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Ecu`].
    #[track_caller]
    #[inline]
    pub fn into_ecu(self) -> FunctionEcu { if let Self::Ecu(ecu) = self { ecu } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Ecu", self.variant()); } }

    /// Returns if this PackageSpecificInfo is a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// True if it is, or false otherwise.
    #[inline]
    pub fn is_dsl(&self) -> bool { matches!(self, Self::Dsl(_)) }
    /// Provides quick immutable access to the internal info as if this was a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// A reference to the internal [`FunctionDsl`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Dsl`].
    #[track_caller]
    #[inline]
    pub fn dsl(&self) -> &FunctionDsl { if let Self::Dsl(dsl) = self { dsl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Dsl", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`FunctionDsl`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Dsl`].
    #[track_caller]
    #[inline]
    pub fn dsl_mut(&mut self) -> &mut FunctionDsl { if let Self::Dsl(dsl) = self { dsl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Dsl", self.variant()); } }
    /// Returns the internal info as if this was a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// The internal [`FunctionDsl`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Dsl`].
    #[track_caller]
    #[inline]
    pub fn into_dsl(self) -> FunctionDsl { if let Self::Dsl(dsl) = self { dsl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Dsl", self.variant()); } }

    /// Returns if this PackageSpecificInfo is a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// True if it is, or false otherwise.
    #[inline]
    pub fn is_cwl(&self) -> bool { matches!(self, Self::Cwl(_)) }
    /// Provides quick immutable access to the internal info as if this was a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// A reference to the internal [`FunctionCwl`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Cwl`].
    #[track_caller]
    #[inline]
    pub fn cwl(&self) -> &FunctionCwl { if let Self::Cwl(cwl) = self { cwl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Cwl", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`FunctionCwl`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Cwl`].
    #[track_caller]
    #[inline]
    pub fn cwl_mut(&mut self) -> &mut FunctionCwl { if let Self::Cwl(cwl) = self { cwl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Cwl", self.variant()); } }
    /// Returns the internal info as if this was a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// The internal [`FunctionCwl`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Cwl`].
    #[track_caller]
    #[inline]
    pub fn into_cwl(self) -> FunctionCwl { if let Self::Cwl(cwl) = self { cwl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Cwl", self.variant()); } }
}



/// Defines what we need to know for ECU packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FunctionEcu {
    /// Capabilities required by this package.
    #[serde(alias = "required", alias = "required_capabilities", default = "HashSet::new")]
    pub capabilities : HashSet<Capability>,
}



/// Defines what we need to know for BraneScript/Bakery packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FunctionDsl {}



/// Defines what we need to know for CWL packages.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FunctionCwl {}
