//  REGISTERING.rs
//    by Lut99
//
//  Created:
//    15 Jan 2024, 14:32:30
//  Last edited:
//    15 Jan 2024, 14:35:45
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines networks structs for communicating with the (local)
//!   registry.
//

use serde::{Deserialize, Serialize};
use serde_json::Value;


/***** LIBRARY *****/
/// Defines the input in the body of a request to download an asset.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DownloadAssetRequest {
    /// The workflow that we're checking.
    ///
    /// Note that we leave it open, as this would require importing `brane-ast` (and that would be a cycling dependency).
    pub workflow: Value,
    /// The task within the workflow that acts as the context in which the download occurs. If omitted, then it should be interpreted as the data being accessed to download the final result of the workflow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task:     Option<(usize, usize)>,
}
