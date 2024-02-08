//  CONTEXT.rs
//    by Lut99
//
//  Created:
//    08 Feb 2024, 15:24:59
//  Last edited:
//    08 Feb 2024, 15:28:54
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the shared context for all paths in the server.
//

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use brane_prx::client::ProxyClient;
use parking_lot::Mutex;


/***** LIBRARY *****/
/// The shared context for all paths in the planner server.
#[derive(Debug)]
pub struct Context {
    /// The path to the node config file.
    pub node_config_path: PathBuf,
    /// The proxy client through which to send API requests.
    pub proxy: ProxyClient,

    /// A map of previously planned snippets.
    pub state: Mutex<HashMap<String, (Instant, HashMap<String, String>)>>,
}
