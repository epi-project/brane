//  FRONTEND.rs
//    by Lut99
// 
//  Created:
//    28 Jun 2023, 09:15:04
//  Last edited:
//    28 Jun 2023, 09:23:03
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the variation of the data info that the user will see.
// 

use std::collections::HashSet;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use brane_shr::info::YamlInfo;

use super::common::DataMetadata;
use super::backend::{self, DataSpecificInfo};


/***** LIBRARY *****/
/// Defines the `data.yml` file's layout used by the users.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataInfo {
    /// Anything common is also for internal usage, so we defer to the [`DataMetadata`] struct.
    #[serde(flatten)]
    pub metadata : DataMetadata,
    /// The rest is kind-specific
    #[serde(alias = "implementation", alias = "contents")]
    pub layout   : DataSpecificInfo,
}
impl YamlInfo for DataInfo {}

impl From<DataInfo> for backend::DataInfo {
    fn from(value: DataInfo) -> Self {
        backend::DataInfo {
            metadata  : value.metadata,
            created   : Utc::now(),
            locations : HashSet::new(),

            layout : value.layout,
        }
    }
}
