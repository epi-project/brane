//  LIB.rs
//    by Lut99
// 
//  Created:
//    14 Jun 2023, 17:38:09
//  Last edited:
//    29 Jun 2023, 13:52:06
//  Auto updated?
//    Yes
// 
//  Description:
//!   Wrapper around `brane-tsk` that provides C-bindings. This allows
//!   C-programs to act as a BRANE client.
//!   
//!   The basics of how to do this are followed from:
//!   http://blog.asleson.org/2021/02/23/how-to-writing-a-c-shared-library-in-rust/
// 


/***** LIBRARY *****/
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Once;

use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info, trace};
use tokio::runtime::{Builder, Runtime};

use brane_ast::{CompileResult, Error as AstError, ParserOptions, Warning as AstWarning};
use brane_ast::ast::Workflow;
use brane_ast::state::CompileState;
use brane_tsk::api::{get_data_index, get_package_index};
use specifications::data::DataIndex;
use specifications::package::PackageIndex;


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

/// Reads a C-string as a Rust string (or at least, attempts to).
/// 
/// # Arguments
/// - `cstr`: The [`*const c_char`](c_char) that we attempt to read as a Rust-string.
/// 
/// # Returns
/// The converted [`str`].
/// 
/// # Errors
/// This function may error if the given `cstr` was not valid unicode.
#[inline]
unsafe fn cstr_to_rust<'s>(cstr: *const c_char) -> Result<&'s str, *const Error> {
    let cstr: &CStr = CStr::from_ptr(cstr);
    match cstr.to_str() {
        Ok(cstr) => Ok(cstr),
        Err(err) => Err(Box::into_raw(Box::new(Error {
            msg   : Some(format!("Input string is not valid UTF-8: {err}")),
            errs  : vec![],
            warns : vec![],
        }))),
    }
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



/// Destructor for the Error type.
///
/// SAFETY: You _must_ call this destructor yourself whenever you are done with the struct to cleanup any code. _Don't_ use any C-library free!
///
/// # Arguments
/// - `err`: The [`Error`] to deallocate.
#[no_mangle]
pub unsafe extern "C" fn error_free(err: *mut Error) {
    init_logger();
    trace!("Destroying Error...");

    // Simply captute the box, then drop
    drop(Box::from_raw(err))
}



/// Returns if this error contains a message to display or not (and thus whether something went wrong).
/// 
/// # Arguments
/// - `err`: The [`Error`] to check the message status of.
/// 
/// # Returns
/// True if [`error_print_warns()`] will print anything, or false otherwise.
#[no_mangle]
pub unsafe extern "C" fn error_warn_occurred(err: *const Error) -> bool {
    !(*err).warns.is_empty()
}

/// Returns if this error contains a message to display or not (and thus whether something went wrong).
/// 
/// # Arguments
/// - `err`: The [`Error`] to check the message status of.
/// 
/// # Returns
/// True if [`error_print_errs()`] will print anything, or false otherwise.
#[no_mangle]
pub unsafe extern "C" fn error_err_occurred(err: *const Error) -> bool {
    !(*err).errs.is_empty()
}

/// Returns if this error contains a message to display or not (and thus whether something went wrong).
/// 
/// # Arguments
/// - `err`: The [`Error`] to check the message status of.
/// 
/// # Returns
/// True if [`error_print_msg()`] will print anything, or false otherwise.
#[no_mangle]
pub unsafe extern "C" fn error_msg_occurred(err: *const Error) -> bool {
    (*err).msg.is_some()
}



/// Prints the warnings in this error to stderr.
/// 
/// The error struct may contain multiple errors if the source code contained those.
/// 
/// # Arguments
/// - `err`: The [`Error`] to check the message status of.
/// - `file`: Some string describing the source/filename of the source text.
/// - `source`: The physical source text, as parsed.
/// 
/// # Returns
/// It may be that parsing the given strings as valid UTF-8 fails. In that case, the returned [`Error`] will be non-NULL and describe the error.
#[no_mangle]
pub unsafe extern "C" fn error_print_warns(err: *const Error, file: *const c_char, source: *const c_char) -> *const Error {
    // Read the file & source strings
    let file: &str = match cstr_to_rust(file) {
        Ok(file) => file,
        Err(err) => { return err; },
    };
    let source: &str = match cstr_to_rust(source) {
        Ok(source) => source,
        Err(err)   => { return err; },
    };

    // Iterate over the warnings to print them
    for warn in &(*err).warns {
        warn.prettyprint(file, source);
    }
    std::ptr::null()
}

/// Prints the errors in this error to stderr.
/// 
/// The error struct may contain multiple errors if the source code contained those.
/// 
/// # Arguments
/// - `err`: The [`Error`] to check the message status of.
/// - `file`: Some string describing the source/filename of the source text.
/// - `source`: The physical source text, as parsed.
/// 
/// # Errors
/// Note that this function may fail to parse the given `file` and `source` strings as valid UTF-8. In that case, it will not print any source errors, but the fact that it failed to do so instead.
#[no_mangle]
pub unsafe extern "C" fn error_print_errs(err: *const Error, file: *const c_char, source: *const c_char) {
    // Read the file & source strings
    let file: &str = match cstr_to_rust(file) {
        Ok(file) => file,
        Err(err) => { error_print_msg(err); return; },
    };
    let source: &str = match cstr_to_rust(source) {
        Ok(source) => source,
        Err(err)   => { error_print_msg(err); return; },
    };

    // Iterate over the errors to print them
    for err in &(*err).errs {
        err.prettyprint(file, source);
    }
}

/// Prints the non-source related error to stderr.
/// 
/// This usually indicates a "harder error" that the user did not do with the input source text.
/// 
/// # Arguments
/// - `err`: The [`Error`] to print the message of.
#[no_mangle]
pub unsafe extern "C" fn error_print_msg(err: *const Error) {
    // Simply write as a log error
    if let Some(msg) = &(*err).msg {
        init_logger();
        error!("{msg}");
    }
}





/***** LIBRARY COMPILER *****/
#[derive(Debug)]
pub struct Compiler {
    /// The package index to use for compilation.
    pindex : PackageIndex,
    /// The data index to use for compilation.
    dindex : DataIndex,
    /// The compile state to use in between snippets.
    state  : CompileState,
}



/// Constructor for the Compiler.
/// 
/// # Arguments
/// - `endpoint`: The endpoint (as an address) to read the package & data index from. This is the address of a `brane-api` instance.
/// - `compiler`: Will point to the newly created [`Compiler`] when done. **Note**: Has to be manually [`free()`](libc::free())ed.
/// 
/// # Returns
/// An [`Error`]-struct that may or may not contain any generated errors. If [`error_err_occurred()`] is true, though, then `compiler` will point to [`NULL`].
#[no_mangle]
pub unsafe extern "C" fn compiler_new(endpoint: *const c_char, compiler: *mut *mut Compiler) -> *const Error {
    init_logger();
    let mut err: Box<Error> = Box::new(Error { msg: None, warns: vec![], errs: vec![] });
    *compiler = std::ptr::null_mut();
    debug!("Constructing BraneScript compiler v{}...", env!("CARGO_PKG_VERSION"));

    // Read the endpoint string
    let endpoint: &str = match cstr_to_rust(endpoint) {
        Ok(endpoint) => endpoint,
        Err(err)     => { return err; }
    };

    // Create a local threaded tokio context
    let runtime: Runtime = match Builder::new_current_thread().enable_time().enable_io().build() {
        Ok(runtime) => runtime,
        Err(e) => {
            err.msg = Some(format!("Failed to create local Tokio context: {e}"));
            return Box::into_raw(err);
        }
    };

    // Load the package index
    debug!("Loading package index from '{endpoint}'...");
    let pindex: PackageIndex = match runtime.block_on(get_package_index(format!("{endpoint}/graphql"))) {
        Ok(index) => index,
        Err(e) => {
            err.msg = Some(format!("Failed to get package index: {e}"));
            return Box::into_raw(err);
        },
    };

    // Load the data index
    debug!("Loading package index from '{endpoint}'...");
    let dindex: DataIndex = match runtime.block_on(get_data_index(format!("{endpoint}/data/info"))) {
        Ok(index) => index,
        Err(e) => {
            err.msg = Some(format!("Failed to get data index: {e}"));
            return Box::into_raw(err);
        },
    };

    // Construct a new Compiler and return it as a pointer
    *compiler = Box::into_raw(Box::new(Compiler {
        pindex,
        dindex,
        state : CompileState::new(),
    }));
    debug!("Compiler created");
    std::ptr::null()
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
    trace!("Destroying BraneScript compiler...");

    // Take ownership of the compiler and then drop it to destroy
    drop(Box::from_raw(compiler));
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
    *wr = std::ptr::null_mut();
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



    /* COMPILE */
    // Compile that using `brane-ast`
    debug!("Compiling snippet...");
    let workflow: Workflow = match brane_ast::compile_snippet(&mut compiler.state, bs.as_bytes(), &compiler.pindex, &compiler.dindex, &ParserOptions::bscript()) {
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
