//  WIZARD.rs
//    by Lut99
// 
//  Created:
//    01 Jun 2023, 12:43:20
//  Last edited:
//    06 Jun 2023, 19:04:36
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements a CLI wizard for setting up nodes, making the process
//!   _even_ easier.
// 

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::PathBuf;

use console::style;
use enum_debug::EnumDebug as _;
use log::{debug, info};

use brane_cfg::node::NodeKind;
use brane_shr::input::{input_path, select};


/***** ERRORS *****/
/// Defines errors that relate to the wizard.
#[derive(Debug)]
pub enum Error {
    /// Failed the query the user for input.
    /// 
    /// The `what` should fill in: `Failed to query the user for ...`
    Input { what: &'static str, err: brane_shr::input::Error },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            Input { what, .. } => write!(f, "Failed to query the user for {what}"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn 'static + error::Error)> {
        use Error::*;
        match self {
            Input { err, .. } => Some(err),
        }
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
    info!("Running wizard to setup a new node...");

    // Do an intro prompt
    println!();
    println!("{}{}{}", style("Welcome to ").bold(), style("Node Setup Wizard").bold().green(), style(format!(" for BRANE v{}", env!("CARGO_PKG_VERSION"))).bold());
    println!();
    println!("This wizard will guide you through the process of setting up a node interactively.");
    println!("Simply answer the questions, and the required configuration files will be generated as you go.");
    println!();

    // Select the path where we will go to
    let path: PathBuf = match input_path("1. Select the location of the node configuration files", "./") {
        Ok(path) => path,
        Err(err) => { return Err(Error::Input { what: "config path", err }); },
    };

    // Let us query the user for the type of node
    let kind: NodeKind = match select("1. Select the type of node to generate", [ NodeKind::Central, NodeKind::Worker, NodeKind::Proxy ]) {
        Ok(kind) => kind,
        Err(err) => { return Err(Error::Input { what: "node kind", err }); },
    };
    debug!("Building for node kind '{}'", kind.variant());
    println!();

    // The rest is node-dependent
    

    // Done!
    Ok(())
}
