//  LIB.rs
//    by Lut99
//
//  Created:
//    26 Sep 2022, 15:12:09
//  Last edited:
//    07 Feb 2024, 13:41:08
//  Auto updated?
//    Yes
//
//  Description:
//!   The `brane-reg` service implements a domain-local registry for both
//!   containers and datasets.
//

// Declare the modules
pub mod check;
pub mod data;
pub mod errors;
pub mod health;
pub mod infra;
pub mod server;
pub mod spec;
pub mod store;
pub mod version;
