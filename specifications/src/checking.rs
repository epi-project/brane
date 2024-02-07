//  CHECKING.rs
//    by Lut99
//
//  Created:
//    07 Feb 2024, 11:54:14
//  Last edited:
//    07 Feb 2024, 18:20:15
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines interface structs & constants necessary for communication
//!   with the `policy-reasoner`.
//

use reqwest::Method;


/***** CONSTANTS *****/
/// Defines the API path to fetch the checker's current list of policies.
pub const POLICY_API_LIST_POLICIES: (Method, &'static str) = (Method::GET, "v1/management/policies/versions");
/// Defines the API path to fetch the currently active version on the checker.
pub const POLICY_API_GET_ACTIVE_VERSION: (Method, &'static str) = (Method::GET, "v1/management/policies/active");
/// Defines the API path to update the currently active version on the checker.
pub const POLICY_API_SET_ACTIVE_VERSION: (Method, &'static str) = (Method::PUT, "v1/management/policies/active");
/// Defines the API path to add a new policy version to the checker.
pub const POLICY_API_ADD_VERSION: (Method, &'static str) = (Method::POST, "v1/management/policies");

/// Defines the API path to check if a workflow as a whole is permitted to be executed.
pub const DELIBERATION_API_WORKFLOW: (Method, &'static str) = (Method::POST, "v1/deliberation/execute-workflow");
/// Defines the API path to check if a task in a workflow is permitted to be executed.
pub const DELIBERATION_API_EXECUTE_TASK: (Method, &'static str) = (Method::POST, "v1/deliberation/execute-task");
/// Defines the API path to check if a dataset in a workflow is permitted to be transferred.
pub const DELIBERATION_API_TRANSFER_DATA: (Method, &'static str) = (Method::POST, "v1/deliberation/access-data");
