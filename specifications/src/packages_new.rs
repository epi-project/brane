//  PACKAGE NEW.rs
//    by Lut99
// 
//  Created:
//    08 Jun 2023, 15:33:55
//  Last edited:
//    12 Jun 2023, 11:24:04
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines file structures for packages and containers, but only for
//!   internal use (i.e., not user-facing).
// 

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::version::Version;


// /***** LIBRARY *****/
// /// Defines the metadata that we need to know per-package.
// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct PackageInfo {
//     /// The name/programming ID of this package.
//     pub name        : Identifier,
//     /// The version of this package.
//     pub version     : Version,
//     /// The list of owners of this package.
//     pub owners      : Option<Vec<String>>,
//     /// A short description of the package.
//     pub description : Option<String>,

//     /// The functions that this package supports.
//     #[serde(alias = "actions")]
//     pub functions : HashMap<String, Function>,
//     /// The classes/types that this package adds.
//     #[serde(alias = "types")]
//     pub classes   : HashMap<String, Class>,
// }
// impl<'de> YamlInfo<'de> for PackageInfo {}
