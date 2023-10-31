//  MOD.rs
//    by Lut99
//
//  Created:
//    27 Oct 2023, 15:56:38
//  Last edited:
//    31 Oct 2023, 15:45:09
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the workflow internally use(d|able) by the checker.
//

// Declare the subsubmodules
pub mod compile;
pub mod optimize;
pub mod spec;
#[cfg(test)]
pub mod tests;
pub mod visualize;
