//  LIB.rs
//    by Lut99
// 
//  Created:
//    07 Jun 2023, 16:22:04
//  Last edited:
//    08 Jun 2023, 15:34:20
//  Auto updated?
//    Yes
// 
//  Description:
//!   The `specifications` crate defines not just public interfaces and
//!   structs, but parts of the specification that are "extra-Rust"; e.g.,
//!   things that relate to command-line parameters, file layouts, network
//!   messages, etc.
// 

// Declare submodules
pub mod address;
pub mod arch;
pub mod common;
pub mod container;
pub mod data;
pub mod driving;
pub mod errors;
pub mod package;
pub mod packages_new;
pub mod planning;
pub mod profiling;
pub mod version;
pub mod working;
