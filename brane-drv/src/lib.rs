//  LIB.rs
//    by Lut99
//
//  Created:
//    26 Sep 2022, 12:00:46
//  Last edited:
//    06 Feb 2024, 11:46:27
//  Auto updated?
//    Yes
//
//  Description:
//!   The `brane-drv` crate implements the 'user delegate' in the central
//!   compute node. To be more precise, it takes user workflows and runs
//!   them, scheduling and orchestrating external function calls (tasks)
//!   where necessary.
//

// Declare the modules
pub mod check;
pub mod errors;
pub mod gc;
pub mod handler;
pub mod planner;
pub mod spec;
pub mod vm;
