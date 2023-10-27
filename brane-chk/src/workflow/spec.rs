//  SPEC.rs
//    by Lut99
// 
//  Created:
//    27 Oct 2023, 15:56:55
//  Last edited:
//    27 Oct 2023, 16:39:43
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines the checker workflow itself.
// 

use std::cell::RefCell;
use std::rc::Rc;

use enum_debug::EnumDebug;

use brane_ast::MergeStrategy;
use brane_ast::locations::Location;
use specifications::version::Version;


/***** HELPER MACROS *****/
/// Implements all the boolean checks for the [`NextElem`]-variants.
/// 
/// # Variants
/// - `next_elem_checks_impl($name:ident)`
///   - `$name`: The name of the type for which to implement them.
/// - `next_elem_checks_impl($($l:lifetime),+), $name:ident)`
///   - `$l`: A list of lifetimes for this type.
///   - `$name`: The name of the type for which to implement them.
macro_rules! next_elem_checks_impl {
    ($name:ident) => {
        impl $name {
            next_elem_checks_impl!(body_impl $name);
        }
    };
    ($($l:lifetime),+, $name:ident) => {
        impl<$($l),+> $name<$($l),+> {
            next_elem_checks_impl!(body_impl $name);
        }
    };



    // Private
    (body_impl $name:ident) => {
        #[doc = concat!("Checks if there is a next node or not.\n\nAlias for `Self::is_elem()`.\n\n# Returns\nTrue if we are [Self::Elem](", stringify!($name), "::Elem), or false otherwise.")]
        #[inline]
        pub fn is_some(&self) -> bool { self.is_elem() }
        #[doc = concat!("Checks if a terminator has been reached or not.\n\n# Returns\nTrue if we are [Self::Next](", stringify!($name), "::Next) or [Self::Stop](", stringify!($name), "::Stop), or false otherwise.")]
        #[inline]
        pub fn is_term(&self) -> bool { self.is_next() || self.is_stop() }

        #[doc = concat!("Checks if there is a next node or not.\n\n# Returns\nTrue if we are [Self::Elem](", stringify!($name), "::Elem), or false otherwise.")]
        #[inline]
        pub fn is_elem(&self) -> bool { matches!(self, Self::Elem(_)) }
        #[doc = concat!("Checks if a `Next`-terminator has been reached.\n\n# Returns\nTrue if we are [Self::Next](", stringify!($name), "::Next), or false otherwise.")]
        #[inline]
        pub fn is_next(&self) -> bool { matches!(self, Self::Next) }
        #[doc = concat!("Checks if a `Stop`-terminator has been reached.\n\n# Returns\nTrue if we are [Self::Stop](", stringify!($name), "::Stop), or false otherwise.")]
        #[inline]
        pub fn is_stop(&self) -> bool { matches!(self, Self::Stop) }
    };
}





/***** AUXILLARY *****/
/// Describes the next node from the current one; which is either the node or a particular terminator that was reached.
/// 
/// This version provides ownership of the next element. See [`NextElemRef`] for a shared reference, or [`NextElemMut`] for a mutable reference.
#[derive(Clone, Debug, EnumDebug)]
pub enum NextElem {
    /// An element is next.
    Elem(Elem),
    /// An [`Elem::Next`]-terminator was encountered.
    Next,
    /// An [`Elem::Stop`]-terminator was encountered.
    Stop,
}
next_elem_checks_impl!(NextElem);

/// Describes the next node from the current one; which is either the node or a particular terminator that was reached.
/// 
/// This version provides a shared reference of the next element. See [`NextElemRef`] for ownership, or [`NextElemMut`] for a mutable reference.
#[derive(Clone, Copy, Debug, EnumDebug)]
pub enum NextElemRef<'e> {
    /// An element is next.
    Elem(&'e Elem),
    /// An [`Elem::Next`]-terminator was encountered.
    Next,
    /// An [`Elem::Stop`]-terminator was encountered.
    Stop,
}
next_elem_checks_impl!('a, NextElemRef);

/// Describes the next node from the current one; which is either the node or a particular terminator that was reached.
/// 
/// This version provides a mutable reference of the next element. See [`NextElemRef`] for ownership, or [`NextElemRef`] for a shared reference.
#[derive(Debug, EnumDebug)]
pub enum NextElemMut<'e> {
    /// An element is next.
    Elem(&'e mut Elem),
    /// An [`Elem::Next`]-terminator was encountered.
    Next,
    /// An [`Elem::Stop`]-terminator was encountered.
    Stop,
}
next_elem_checks_impl!('a, NextElemMut);





/***** AUXILLARY DATA *****/
/// Defines how a user looks like.
#[derive(Clone, Debug)]
pub struct User {
    /// The name of the user.
    pub name     : String,
    /// Any metadata attached to the user. Note: may need to be populated by the checker!
    pub metadata : Vec<Metadata>,
}

/// Defines a representation of a dataset.
#[derive(Clone, Debug)]
pub struct Dataset {
    /// The name of the dataset.
    pub name     : String,
    /// The place that we get it from. No transfer is necessary if this is the place of task execution.
    pub from     : Option<Location>,
    /// Any metadata attached to the dataset. Note: may need to be populated by the checker!
    pub metadata : Vec<Metadata>,
}

/// Represents a "tag" and everything we need to know.
#[derive(Clone, Debug)]
pub struct Metadata {
    /// The tag itself.
    pub tag             : String,
    /// The namespace where the tag may be found. Represents the "owner", or the "definer" of the tag.
    pub namespace       : String,
    /// The signature verifying this metadata. Represents the "assigner", or the "user" of the tag.
    pub signature       : String,
    /// A flag stating whether the signature is valid. If [`None`], means this hasn't been validated yet.
    pub signature_valid : Option<bool>,
}





/***** LIBRARY *****/
/// Defines the workflow's toplevel view.
#[derive(Clone, Debug)]
pub struct Workflow {
    /// Defines the first node in the workflow.
    pub start : Elem,

    /// The user instigating this workflow (and getting the result, if any).
    pub user      : User,
    /// The metadata associated with this workflow as a whole.
    pub metadata  : Vec<Metadata>,
    /// The signature verifying this workflow. Is this needed???.
    pub signature : String,
}



/// Defines an element in the graph. This is either a _Node_, which defines a task execution, or an _Edge_, which defines how next tasks may be reached.
#[derive(Clone, Debug, EnumDebug)]
pub enum Elem {
    // Nodes
    /// Defines a task, which is like a [linear edge](Elem::Linear) but with a task to execute.
    Task(ElemTask),

    // Edges
    /// Defines an edge that linearly connects to the next Elem. Can be thought of as a Task without the Task-part.
    Linear(ElemLinear),
    /// Defines an edge that connects to multiple next graph-branches of which only _one_ must be taken. Note that, because we don't include dynamic control flow information, we don't know _which_ will be taken.
    Branch(ElemBranch),
    /// Defines an edge that connects to multiple next graph-branches of which _all_ must be taken _concurrently_.
    Parallel(ElemParallel),
    /// Defines an edge that repeats a particular branch an unknown amount of times.
    Loop(ElemLoop),
    /// Calls another stream of edges, then continues onwards.
    Call(ElemCall),

    // Terminators
    /// Defines that the next element to execute is given by the parent `next`-field.
    Next,
    /// Defines that no more execution takes place.
    Stop,
}
impl Elem {
    /// Retrieves the `next` element of ourselves.
    /// 
    /// If this Elem is a terminating element, then it returns which of the ones is reached.
    /// 
    /// # Returns
    /// A [`NextElemRef`]-enum that either gives the next element in [`NextElemRef::Elem`], or a terminator as [`NextElemRef::TermNext`] or [`NextElemRef::TermStop`].
    pub fn next(&self) -> NextElemRef {
        todo!();
        NextElem::Next.is_some();
    }
}

/// Defines the only node in the graph consisting of [`Elem`]s.
/// 
/// Yeah so basically represents a task execution, with all checker-relevant information.
#[derive(Clone, Debug)]
pub struct ElemTask {
    /// The name of the task to execute
    pub name    : String,
    /// The name of the package in which to find the task.
    pub package : String,
    /// The version number of the package in which to find the task.
    pub version : Version,
    /// The hash of the container, specifically.
    pub hash    : Option<String>,

    /// Any input datasets used by the task.
    pub input  : Vec<Dataset>,
    /// If there is an output dataset produced by this task, this names it.
    pub output : Option<Dataset>,

    /// The location where the task is planned to be executed, if any.
    pub location  : Option<Location>,
    /// The list of metadata belonging to this task. Note: may need to be populated by the checker!
    pub metadata  : Vec<Metadata>,
    /// The signature verifying this container.
    pub signature : String,

    /// The next graph element that this task connects to.
    pub next : Box<Elem>,
}

/// Defines a linear connection between two graph [`Elem`]ents.
#[derive(Clone, Debug)]
pub struct ElemLinear {
    /// The next graph element that this linear edge connects to.
    pub next : Box<Elem>,
}

/// Defines a branching connection between graph [`Elem`]ents.
/// 
/// Or rather, defines a linear connection between two nodes, with a set of branches in between them.
#[derive(Clone, Debug)]
pub struct ElemBranch {
    /// The branches of which one _must_ be taken, but we don't know which one.
    pub branches : Vec<Elem>,
    /// The next graph element that this branching edge connects to.
    pub next : Box<Elem>,
}

/// Defines a parallel connection between graph [`Elem`]ents.
/// 
/// Is like a [branch](ElemBranch), except that _all_ branches are taken _concurrently_ instead of only one.
#[derive(Clone, Debug)]
pub struct ElemParallel {
    /// The branches, _all_ of which but be taken _concurrently_.
    pub branches : Vec<Elem>,
    /// The method of joining the branches.
    pub merge    : MergeStrategy,
    /// The next graph element that this parallel edge connects to.
    pub next     : Box<Elem>,
}

/// Defines a looping connection between graph [`Elem`]ents.
/// 
/// Simply defines a branch that is taken repeatedly. Any condition that was there is embedded in the branching part, since that's how the branch is dynamically taken and we can't know how often any of them is taken anyway.
#[derive(Clone, Debug)]
pub struct ElemLoop {
    /// The branch elements to take.
    pub body : Box<Elem>,
    /// The next graph element that this parallel edge connects to.
    pub next : Box<Elem>,
}

/// Defines a calling connection between graph [`Elem`]ents.
/// 
/// Refers (not defines) to a shared-ownership branch of elements that is executed before the `next`` element is continued with.
#[derive(Clone, Debug)]
pub struct ElemCall {
    /// The set of elements to call.
    pub func : Rc<RefCell<Elem>>,
    /// The next graph element that this calling edge connects to.
    pub next : Box<Elem>,
}
