//  NETWORK.rs
//    by Lut99
//
//  Created:
//    28 Sep 2022, 10:33:37
//  Last edited:
//    08 Feb 2024, 17:24:07
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines Kafka network messages used by `brane-drv` <-> `brane-job`
//!   <-> `brane-plr` interaction.
//

use serde::{Deserialize, Serialize};
use serde_json::Value;


/***** NETWORKING *****/
/// Defines a message that carries an _unplanned_ workflow. It is destined to be intercepted by the planner.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanningRequest {
    /// Defines the app (=workflow) ID that matches this snippet to a global workflow.
    pub app_id:   String,
    /// The raw workflow, as JSON, that is sent around. It may be expected that there is usually at least one task that does not have a location annotated.
    ///
    /// Note that, to avoid cyclic dependency on `brane-ast`, we define it as an abstract JSON [`Value`].
    pub workflow: Value,
}

/// Defines the reply of the planning request in the happy path.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanningReply {
    /// The workflow after planning.
    ///
    /// Note that, to avoid cyclic dependency on `brane-ast`, we define it as an abstract JSON [`Value`].
    pub plan: Value,
}

/// Defines the reply of the planner if a checker denied the request.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlanningDeniedReply {
    /// The domain that denied.
    pub domain:  String,
    /// A list of reasons given by the domain. May be empty.
    pub reasons: Vec<String>,
}
