//  WIZARD.rs
//    by Lut99
// 
//  Created:
//    01 Jun 2023, 12:43:20
//  Last edited:
//    01 Jun 2023, 12:52:40
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements a CLI wizard for setting up nodes, making the process
//!   _even_ easier.
// 

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};


/***** ERRORS *****/
/// Defines errors that relate to the wizard.
#[derive(Debug)]
pub enum Error {
    
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        // use Error::*;
        // match self {

        // }
        Ok(())
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn 'static + error::Error)> {
        // use Error::*;
        // match self {

        // }
        None
    }
}





/***** LIBRARY *****/
/// Main handler for the `branectl wizard setup` (or `branectl wizard node`) subcommand.
/// 
/// # Arguments
/// 
/// # Errors
/// This function may error if any of the wizard steps fail.
pub fn setup() -> Result<(), Error> {
    Ok(())
}
