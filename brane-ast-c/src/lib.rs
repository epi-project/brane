//  LIB.rs
//    by Lut99
// 
//  Created:
//    14 Jun 2023, 11:48:13
//  Last edited:
//    14 Jun 2023, 18:14:16
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

use brane_ast::ast::Workflow;


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





/***** ERRORS *****/
/// Defines the error type returned by this library.
#[derive(Clone, Debug)]
pub struct Error {
    /// The error message.
    msg : CString,
}





/***** LIBRARY *****/
/// Returns the BRANE version for which this compiler is valid.
/// 
/// # Returns
/// String version that contains a major, minor and patch version separated by dots.
#[no_mangle]
pub extern "C" fn version() -> *const c_char {
    // SAFETY: We can easily do this without a care in the world, since the string is static and won't need deallocation.
    C_VERSION.as_ptr() as *const c_char
}



/// Compiles the given BraneScript snippet to the BRANE Workflow Representation.
/// 
/// Note that the representation is returned as JSON, and not really meant to be inspected from C-code.
/// Use other functions in this library instead to ensure you are compatible with the latest WR version.
/// 
/// # Arguments
/// - `bs`: The raw BraneScript snippet to parse.
/// - `wr`: Will point to the compiled JSON string when done. **Note**: Has to be manually [`free()`](libc::free())ed.
/// 
/// # Returns
/// [`NULL`](std::ptr::null())
/// 
/// # Errors
/// If this function errors, typically because the given snippet is invalid BraneScript, then an [`Error`]-struct is returned instead containing information about what happened.
pub extern "C" fn compile(bs: *const c_char, wr: *mut *mut c_char) -> *const Error {
    // Initialize the logger if we hadn't already
    init_logger();
    info!("Compiling snippet with BraneScript compiler v{}", env!("CARGO_PKG_VERSION"));

    // Get the input as a Rust string
    let bs: &CStr = unsafe { CStr::from_ptr(bs) };
    let bs: &str = match bs.to_str() {
        Ok(bs)   => bs,
        Err(err) => { return Box::into_raw(Box::new(Error { msg: CString::new(format!("Input string is not valid UTF-8: {err}")).unwrap() })) },
    };

    // Compile that using `brane-ast`
    let workflow: Workflow = match brane_ast::compile_snippet(state, reader, package_index, data_index, options) {

    };

    // OK, nothing to error!
    std::ptr::null()
}
