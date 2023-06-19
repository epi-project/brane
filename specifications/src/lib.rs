//  LIB.rs
//    by Lut99
// 
//  Created:
//    07 Jun 2023, 16:22:04
//  Last edited:
//    19 Jun 2023, 09:51:30
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
pub mod arch;
pub mod capabilities;
pub mod common_old;
pub mod container;
pub mod data;
pub mod data_new;
pub mod driving;
pub mod errors;
pub mod index;
pub mod package_old;
pub mod packages;
pub mod planning;
pub mod profiling;
pub mod working;
