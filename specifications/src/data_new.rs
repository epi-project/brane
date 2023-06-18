//  DATA NEW.rs
//    by Lut99
// 
//  Created:
//    18 Jun 2023, 18:22:18
//  Last edited:
//    18 Jun 2023, 18:28:42
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines file structures for datasets.
// 

use enum_debug::EnumDebug;
use serde::{Deserialize, Serialize};

use brane_shr::info::{JsonInfo, YamlInfo};
use brane_shr::serialize::Identifier;
use brane_shr::version::Version;


/***** LIBRARY *****/
/// Defines the `data.yml` file's layout.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataInfo {
    /// Anything common is also for internal usage, so we defer to the [`Datametadata`] struct.
    #[serde(flatten)]
    pub metadata : DataMetadata,
    /// The rest is kind-specific
    #[serde(alias = "implementation", alias = "contents")]
    pub layout   : DataSpecificInfo,
}
impl YamlInfo for DataInfo {}


/// Defines what we need to know for the backend only.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataMetadata {
    /// The name/programming ID of this package.
    pub name        : Identifier,
    /// The version of this package.
    pub version     : Version,
    /// The list of owners of this package.
    pub owners      : Option<Vec<String>>,
    /// A short description of the package.
    pub description : Option<String>,
}
impl JsonInfo for DataMetadata {}


/// Defines what we need to know per package type.
#[derive(Clone, Debug, Deserialize, EnumDebug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataSpecificInfo {
    /// It's a container.
    Ecu(EcuInfo),
    /// It's a BraneScript/Bakery package.
    Dsl(DslInfo),
    /// It's a CWL package.
    Cwl(CwlInfo),
}
impl DataSpecificInfo {
    /// Returns an enum that can be used to represent the kind of this info.
    /// 
    /// # Returns
    /// A [`PackageKind`] that represents the kind of this info.
    #[inline]
    pub fn kind(&self) -> PackageKind {
        use DataSpecificInfo::*;
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
    /// A reference to the internal [`EcuInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Ecu`].
    #[track_caller]
    #[inline]
    pub fn ecu(&self) -> &EcuInfo { if let Self::Ecu(ecu) = self { ecu } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Ecu", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`PackageSpecificInfo::Ecu`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`EcuInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Ecu`].
    #[track_caller]
    #[inline]
    pub fn ecu_mut(&mut self) -> &mut EcuInfo { if let Self::Ecu(ecu) = self { ecu } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Ecu", self.variant()); } }
    /// Returns the internal info as if this was a [`PackageSpecificInfo::Ecu`].
    /// 
    /// # Returns
    /// The internal [`EcuInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Ecu`].
    #[track_caller]
    #[inline]
    pub fn into_ecu(self) -> EcuInfo { if let Self::Ecu(ecu) = self { ecu } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Ecu", self.variant()); } }

    /// Returns if this PackageSpecificInfo is a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// True if it is, or false otherwise.
    #[inline]
    pub fn is_dsl(&self) -> bool { matches!(self, Self::Dsl(_)) }
    /// Provides quick immutable access to the internal info as if this was a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// A reference to the internal [`DslInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Dsl`].
    #[track_caller]
    #[inline]
    pub fn dsl(&self) -> &DslInfo { if let Self::Dsl(dsl) = self { dsl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Dsl", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`DslInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Dsl`].
    #[track_caller]
    #[inline]
    pub fn dsl_mut(&mut self) -> &mut DslInfo { if let Self::Dsl(dsl) = self { dsl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Dsl", self.variant()); } }
    /// Returns the internal info as if this was a [`PackageSpecificInfo::Dsl`].
    /// 
    /// # Returns
    /// The internal [`DslInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Dsl`].
    #[track_caller]
    #[inline]
    pub fn into_dsl(self) -> DslInfo { if let Self::Dsl(dsl) = self { dsl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Dsl", self.variant()); } }

    /// Returns if this PackageSpecificInfo is a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// True if it is, or false otherwise.
    #[inline]
    pub fn is_cwl(&self) -> bool { matches!(self, Self::Cwl(_)) }
    /// Provides quick immutable access to the internal info as if this was a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// A reference to the internal [`CwlInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Cwl`].
    #[track_caller]
    #[inline]
    pub fn cwl(&self) -> &CwlInfo { if let Self::Cwl(cwl) = self { cwl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Cwl", self.variant()); } }
    /// Provides quick mutable access to the internal info as if this was a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// A mutable reference to the internal [`CwlInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Cwl`].
    #[track_caller]
    #[inline]
    pub fn cwl_mut(&mut self) -> &mut CwlInfo { if let Self::Cwl(cwl) = self { cwl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Cwl", self.variant()); } }
    /// Returns the internal info as if this was a [`PackageSpecificInfo::Cwl`].
    /// 
    /// # Returns
    /// The internal [`CwlInfo`].
    /// 
    /// # Panics
    /// This function panics if we are something else than a [`PackageSpecificInfo::Cwl`].
    #[track_caller]
    #[inline]
    pub fn into_cwl(self) -> CwlInfo { if let Self::Cwl(cwl) = self { cwl } else { panic!("Cannot unwrap {:?} as a PackageSpecificInfo::Cwl", self.variant()); } }
}
