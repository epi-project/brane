//  COMPILE.rs
//    by Lut99
// 
//  Created:
//    27 Oct 2023, 17:39:59
//  Last edited:
//    27 Oct 2023, 18:17:31
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines conversion functions between the
//!   [Checker Workflow](Workflow) and the [WIR](ast::Workflow).
// 

use std::collections::HashMap;
use std::convert::TryFrom;
use std::error;
use std::fmt::{Display, Formatter, Result as FResult};

use brane_ast::MergeStrategy;
use brane_ast::ast;
use brane_ast::state::VirtualSymTable;

use super::spec::Workflow;


/***** ERRORS *****/
/// Defines errors that may occur when compiling an [`ast::Workflow`] to a [`Workflow`].
#[derive(Debug)]
pub enum Error {
    /// Unknown task given.
    UnknownTask { id: usize },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            UnknownTask { id } => write!(f, "Encountered unknown task ID {id} in Node"),
        }
    }
}
impl error::Error for Error {}





/***** HELPER FUNCTIONS *****/
/// Analyses the edges in an [`ast::Workflow`] to resolve function calls to the ID of the functions they call.
/// 
/// # Arguments
/// - `wir`: The [`ast::Workflow`] to analyse.
/// - `table`: A running [`VirtualSymTable`] that determines the current types in scope.
/// - `stack_id`: The function ID currently known to be on the stack. Is [`None`] if we don't know this.
/// - `pc`: The program-counter-index of the edge to analyse. These are pairs of `(function, edge_idx)`, where main is referred to by [`usize::MAX`](usize).
/// - `breakpoint`: An optional program-counter-index that, if given, will not analyse that edge onwards (excluding it too).
/// 
/// # Returns
/// A tuple with a [`HashMap`] that maps call indices (as program-counter-indices) to function IDs and an optional top call ID currently on the stack.
/// 
/// Note that, if a call ID occurs in the map but has [`None`] as function ID, it means it does not map to a body (e.g., a builtin).
/// 
/// # Errors
/// This function may error if we failed to statically discover the function IDs.
fn resolve_calls(wir: &ast::Workflow, table: &VirtualSymTable, pc: (usize, usize), stack_id: Option<usize>, breakpoint: Option<(usize, usize)>) -> Result<(HashMap<(usize, usize), Option<usize>>, Option<usize>), Error> {
    // Quit if we're at the breakpoint
    if let Some(breakpoint) = breakpoint {
        if pc == breakpoint { return Ok((HashMap::new(), None)); }
    }

    // Get the edge in the workflow
    let edge: &ast::Edge = if pc.0 == usize::MAX {
        match wir.graph.get(pc.1) {
            Some(edge) => edge,
            None       => { return Ok((HashMap::new(), None)); },
        }
    } else {
        match wir.funcs.get(&pc.0) {
            Some(edges) => match edges.get(pc.1) {
                Some(edge) => edge,
                None       => { return Ok((HashMap::new(), None)); },
            },
            None => { return Ok((HashMap::new(), None)); },
        }
    };

    // Match to recursively process it
    match edge {
        ast::Edge::Node { task, next, .. } => {
            // Attempt to discover the return type of the Node.
            let def: &ast::TaskDef = match std::panic::catch_unwind(|| table.task(*task)) {
                Ok(def) => def,
                Err(_)  => { return Err(Error::UnknownTask { id: *task }) },
            };

            // Alright, recurse with the next instruction
            resolve_calls(wir, table, (pc.0, *next), if def.func().ret.is_void() { stack_id } else { None },  breakpoint)
        },

        ast::Edge::Linear { instrs, next } => {
            // Analyse the instructions to find out if we can deduce a new `stack_id`
            // TODO

            // Analyse the next one
            resolve_calls(wir, table, (pc.0, *next), stack_id, breakpoint)
        },

        ast::Edge::Stop {} => Ok((HashMap::new(), None)),

        ast::Edge::Branch { true_next, false_next, merge } => {
            // First, analyse the branches
            let (mut calls, mut stack_id): (HashMap<_, _>, Option<usize>) = resolve_calls(wir, table, (pc.0, *true_next), stack_id, merge.map(|merge| (pc.0, merge)))?;
            if let Some(false_next) = false_next {
                let (false_calls, false_stack) = resolve_calls(wir, table, (pc.0, *false_next), stack_id, merge.map(|merge| (pc.0, merge)))?;
                calls.extend(false_calls);
                if stack_id != false_stack { stack_id = None; }
            }

            // Analyse the remaining part next
            if let Some(merge) = merge {
                let (merge_calls, merge_stack) = resolve_calls(wir, table, (pc.0, *merge), stack_id, breakpoint)?;
                calls.extend(merge_calls);
                stack_id = merge_stack;
            }

            // Alright, return the found results
            Ok((calls, stack_id))
        },

        ast::Edge::Parallel { branches, merge } => {
            // Simply analyse all branches first. No need to worry about their return values and such, since that's not until the `Join`.
            let mut calls: HashMap<_, _> = HashMap::new();
            for branch in branches {
                calls.extend(resolve_calls(wir, table, (pc.0, *branch), stack_id, breakpoint)?.0);
            }

            // OK, then analyse the rest assuming the stack is unchanged (we can do that because the parallel's branches get clones)
            let (new_calls, stack_id): (HashMap<_, _>, Option<usize>) = resolve_calls(wir, table, (pc.0, *merge), stack_id, breakpoint)?;
            calls.extend(new_calls);
            Ok((calls, stack_id))
        },

        ast::Edge::Join { merge, next } => {
            // Simply do the next, only _not_ resetting the stack ID if no value is returned.
            resolve_calls(wir, table, (pc.0, *next), if *merge == MergeStrategy::None { stack_id } else { None }, breakpoint)
        },

        ast::Edge::Loop { cond, body, next } => {
            // 
        },
    }
}





/***** LIBRARY *****/
impl TryFrom<ast::Workflow> for Workflow {
    type Error = Error;

    #[inline]
    fn try_from(value: ast::Workflow) -> Result<Self, Self::Error> {
        // First, let's convert all the edges
        

        todo!();
    }
}
