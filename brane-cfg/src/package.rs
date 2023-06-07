//  PACKAGE.rs
//    by Lut99
// 
//  Created:
//    07 Jun 2023, 16:23:43
//  Last edited:
//    07 Jun 2023, 16:25:39
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the `package.yml` file layouts.
//!   
//!   This is just the user-facing version. For internally used counterparts,
//!   check the [`metadata`](crate::metadata) submodule.
// 

use specifications::version::Version;

use crate::info::YamlInfo;


/***** LIBRARY *****/
// /// Defines the `package.yml` file's layout.
// pub struct PackageInfo {
//     /// The name/programming ID of this package.
//     pub name        : String,
//     /// The version of this package.
//     pub version     : Version,
//     /// The kind of this package.
//     pub kind        : PackageKind,
//     /// The list of owners of this package.
//     pub owners      : Option<Vec<String>>,
//     /// A short description of the package.
//     pub description : Option<String>,

//     /// The functions that this package supports.
//     pub actions    : Map<Action>,
//     /// The entrypoint of the image
//     pub entrypoint : Entrypoint,
//     /// The types that this package adds.
//     pub types      : Option<Map<Type>>,

//     /// The base image to use for the package image.
//     pub base         : Option<String>,
//     /// The dependencies, as install commands for sudo apt-get install -y <...>
//     pub dependencies : Option<Vec<String>>,
//     /// Any environment variables that the user wants to be set
//     pub environment  : Option<Map<String>>,
//     /// The list of additional files to copy to the image
//     pub files        : Option<Vec<String>>,
//     /// An extra script to run to initialize the working directory
//     pub initialize   : Option<Vec<String>>,
//     /// An extra set of commands that will be run _before_ the workspace is copied over. Useful for non-standard general dependencies.
//     pub install      : Option<Vec<String>>,
//     /// An extra set of commands that will be run _after_ the workspace is copied over. Useful for preprocessing or unpacking things.
//     #[serde(alias = "postinstall", alias = "post-install", alias = "post_install")]
//     pub unpack       : Option<Vec<String>>,
// }

// impl<'de> YamlInfo<'de> for PackageInfo {}
