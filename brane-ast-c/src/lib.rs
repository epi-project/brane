//  LIB.rs
//    by Lut99
// 
//  Created:
//    14 Jun 2023, 11:48:13
//  Last edited:
//    15 Jun 2023, 19:33:15
//  Auto updated?
//    Yes
// 
//  Description:
//!   Wrapper around `brane-ast` that provides C-bindings. This allows the
//!   BraneScript compiler to be used in `brane-ide`.
//!   
//!   The basics of how to do this are followed from:
//!   http://blog.asleson.org/2021/02/23/how-to-writing-a-c-shared-library-in-rust/
// 

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Once;

use humanlog::{DebugMode, HumanLogger};
use log::{debug, info};

use brane_ast::{CompileResult, Error as AstError, ParserOptions, Warning as AstWarning};
use brane_ast::ast::Workflow;
use brane_ast::state::CompileState;


/***** CONSTANTS *****/
/// The version string of this package, null-terminated for C-compatibility.
static C_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");





/***** GLOBALS *****/
/// Ensures that the initialization function is run only once.
static LOG_INIT: Once = Once::new();





/***** HELPER FUNCTIONS *****/
/// Initializes the logging system if it hadn't already.
#[inline]
fn init_logger() {
    LOG_INIT.call_once(|| {
        if let Err(err) = HumanLogger::terminal(DebugMode::Debug).init() {
            eprintln!("WARNING: Failed to setup Rust logger: {err} (logging disabled for this session)");
        }
    });
}





/***** LIBRARY FUNCTIONS *****/
/// Returns the BRANE version for which this compiler is valid.
/// 
/// # Returns
/// String version that contains a major, minor and patch version separated by dots.
#[no_mangle]
pub extern "C" fn version() -> *const c_char {
    // SAFETY: We can easily do this without a care in the world, since the string is static and won't need deallocation.
    C_VERSION.as_ptr() as *const c_char
}





/***** LIBRARY ERROR *****/
/// Defines the error type returned by this library.
#[derive(Debug)]
pub struct Error {
    /// Any custom error message to print that is not from the compiler itself.
    msg   : Option<String>,
    /// The warning messages to print.
    warns : Vec<AstWarning>,
    /// The error messages to print.
    errs  : Vec<AstError>,
}





/***** LIBRARY COMPILER *****/
#[derive(Debug)]
pub struct Compiler {
    /// The compile state to use in between snippets.
    state : CompileState,
}



/// Constructor for the Compiler.
/// 
/// # Returns
/// A new [`Compiler`] instance.
#[no_mangle]
pub extern "C" fn compiler_new() -> *const Compiler {
    init_logger();
    info!("Constructing BraneScript compiler v{}...", env!("CARGO_PKG_VERSION"));

    // // Load the package index
    // debug!("Loading package index from '{}'...");
    // let pindex: PackageIndex = 

    // // Load the data index
    // debug!("Loading data index from '{}'...");
    // let dindex: DataIndex = 

    // Construct a new Compiler and return it as a pointer
    Box::into_raw(Box::new(Compiler {
        state : CompileState::new(),
    }))
}

/// Destructor for the Compiler.
/// 
/// SAFETY: You _must_ call this destructor yourself.
/// 
/// # Arguments
/// - `compiler`: The [`Compiler`] to free.
#[no_mangle]
pub unsafe extern "C" fn compiler_free(compiler: *mut Compiler) {
    init_logger();
    info!("Destroying BraneScript compiler...");

    // Take ownership of the compiler and then drop it to destroy
    Box::from_raw(compiler);
}



/// Compiles the given BraneScript snippet to the BRANE Workflow Representation.
/// 
/// Note that the representation is returned as JSON, and not really meant to be inspected from C-code.
/// Use other functions in this library instead to ensure you are compatible with the latest WR version.
/// 
/// # Arguments
/// - `compiler`: The [`Compiler`] to compile with. Essentially this determines which previous compile state to use.
/// - `bs`: The raw BraneScript snippet to parse.
/// - `wr`: Will point to the compiled JSON string when done. **Note**: Has to be manually [`free()`](libc::free())ed.
/// 
/// # Returns
/// An [`Error`]-struct that may or may not contain any generated errors. If [`error_err_occurred()`] is true, though, then `wr` will point to [`NULL`](std::ptr::null()).
#[no_mangle]
pub unsafe extern "C" fn compiler_compile(compiler: *mut Compiler, bs: *const c_char, wr: *mut *mut c_char) -> *const Error {
    // Initialize the logger if we hadn't already
    init_logger();
    let mut err: Box<Error> = Box::new(Error { msg: None, warns: vec![], errs: vec![] });
    info!("Compiling snippet...");



    /* INPUT */
    // Cast the Compiler pointer to a Compiler reference
    debug!("Reading compiler input...");
    let compiler: &mut Compiler = &mut *compiler;

    // Get the input as a Rust string
    let bs: &CStr = CStr::from_ptr(bs);
    let bs: &str = match bs.to_str() {
        Ok(bs) => bs,
        Err(e) => {
            err.msg = Some(format!("Input string is not valid UTF-8: {e}"));
            return Box::into_raw(err);
        },
    };

    // Set the output string to avoid confusion
    *wr = std::ptr::null_mut();



    /* COMPILE */
    // Compile that using `brane-ast`
    debug!("Compiling snippet...");
    let workflow: Workflow = match brane_ast::compile_snippet(&mut compiler.state, bs.as_bytes(), package_index, data_index, &ParserOptions::bscript()) {
        CompileResult::Workflow(workflow, warns) => {
            err.warns = warns;
            workflow
        },

        CompileResult::Eof(e) => {
            err.errs = vec![ e ];
            return Box::into_raw(err);
        },
        CompileResult::Err(errs) => {
            err.errs = errs;
            return Box::into_raw(err);
        },

        CompileResult::Program(_, _)    |
        CompileResult::Unresolved(_, _) => { unreachable!(); },
    };



    /* SERIALIZE */
    // Store the serialized workflow as a C-string
    debug!("Serializing workflow...");
    let workflow: String = match serde_json::to_string(&workflow) {
        Ok(workflow) => workflow,
        Err(e)       => {
            err.msg = Some(format!("Failed to serialize workflow: {e}"));
            return Box::into_raw(err);
        },
    };
    let workflow: CString = match CString::new(workflow) {
        Ok(workflow) => workflow,
        Err(e)       => {
            err.msg = Some(format!("Failed to convert serialized workflow to a C-compatible string: {e}"));
            return Box::into_raw(err);
        },
    };

    // Allocate the proper space (we do the copy a bit around-the-bend to be compatible with a C-style free).
    let n_chars: usize = libc::strlen(workflow.as_ptr());
    let target: *mut c_char = libc::malloc(n_chars + 1) as *mut c_char;

    // Write the workflow there
    libc::strncpy(target, workflow.as_ptr(), n_chars);
    *wr = target;

    // OK, return the error struct!
    debug!("Compilation success");
    Box::into_raw(err)
}
