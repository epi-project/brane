//  CONTEXT.rs
//    by Lut99
//
//  Created:
//    08 Feb 2024, 15:24:59
//  Last edited:
//    25 Nov 2024, 09:47:53
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


/***** TYPES *****/
/// Defines a single "workflow" state of the planner.
///
/// This is used in the [`Context::state`], which represents sessions of per-snippet workflow
/// planning. This is used when REPL'ing a workflow, as every snippet needs to be planned
/// individually but relies on where any intermediate results are generated in previous snippets.
/// [`Context::state`] keeps track of the locations of these intermediate results, and this type
/// defines every such session.
///
/// A session is simply a pair of the last time it was accessed (stale sessions g et removed by the
/// garbage collector) and a map of intermediate results names to the domain where they are found.
pub type Session = (Instant, HashMap<String, String>);





/***** LIBRARY *****/
/// The shared context for all paths in the planner server.
#[derive(Debug)]
pub struct Context {
    /// The path to the node config file.
    pub node_config_path: PathBuf,
    /// The proxy client through which to send API requests.
    pub proxy: ProxyClient,

    /// A map of previously planned snippets.
    pub state: Mutex<HashMap<String, Session>>,
}
