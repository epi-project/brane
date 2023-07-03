//  LIB.rs
//    by Lut99
// 
//  Created:
//    14 Jun 2023, 17:38:09
//  Last edited:
//    03 Jul 2023, 11:36:33
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

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Once;

use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info, trace};
use tokio::runtime::{Builder, Runtime};

use brane_ast::{CompileResult, Error as AstError, ParserOptions, SymTable, Warning as AstWarning};
use brane_ast::ast::Workflow;
use brane_ast::state::CompileState;
use brane_ast::traversals::print::ast;
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
#[track_caller]
unsafe fn cstr_to_rust<'s>(cstr: *const c_char) -> &'s str {
    let cstr: &CStr = CStr::from_ptr(cstr);
    match cstr.to_str() {
        Ok(cstr) => cstr,
        Err(err) => { panic!("Given char-pointer does point to valid UTF-8 string: {err}"); },
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
    /// The message to print.
    msg : String,
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

/// Prints the error message in this error to stderr.
/// 
/// # Arguments
/// - `err`: The [`Error`] to print.
/// 
/// # Panics
/// This function can panic if the given `err` is a NULL-pointer.
#[no_mangle]
pub unsafe extern "C" fn error_print_err(err: *const Error) {
    init_logger();

    // Read the pointer
    let err: &Error = match err.as_ref() {
        Some(err) => err,
        None => { panic!("Given Error is a NULL-pointer"); },
    };

    // Simply log it as an error
    error!("{}", err.msg);
}





/***** LIBRARY SOURCE ERROR *****/
/// Defines the error type returned by this library.
#[derive(Debug)]
pub struct SourceError {
    /// The warning messages to print.
    warns : Vec<AstWarning>,
    /// The error messages to print.
    errs  : Vec<AstError>,
    /// Any custom error message to print that is not from the compiler itself.
    msg   : Option<String>,
}



/// Destructor for the Error type.
///
/// SAFETY: You _must_ call this destructor yourself whenever you are done with the struct to cleanup any code. _Don't_ use any C-library free!
///
/// # Arguments
/// - `serr`: The [`SourceError`] to deallocate.
#[no_mangle]
pub unsafe extern "C" fn serror_free(serr: *mut SourceError) {
    init_logger();
    trace!("Destroying SourceError...");

    // Simply captute the box, then drop
    drop(Box::from_raw(serr))
}



/// Returns if a source warning has occurred.
/// 
/// # Arguments
/// - `serr`: The [`SourceError`] struct to inspect.
/// 
/// # Returns
/// True if [`serr_print_swarns`] would print anything, or false otherwise.
/// 
/// # Panics
/// This function can panic if the given `serr` is a NULL-pointer.
#[no_mangle]
pub unsafe extern "C" fn serror_has_swarns(serr: *const SourceError) -> bool {
    // Unwrap the pointer
    let serr: &SourceError = match serr.as_ref() {
        Some(err) => err,
        None => { panic!("Given SourceError is a NULL-pointer"); },
    };

    // Now return if there are any warnings
    !(*serr).warns.is_empty()
}

/// Returns if a source error has occurred.
/// 
/// # Arguments
/// - `serr`: The [`SourceError`] struct to inspect.
/// 
/// # Returns
/// True if [`serr_print_serrs`] would print anything, or false otherwise.
/// 
/// # Panics
/// This function can panic if the given `err` is a NULL-pointer.
#[no_mangle]
pub unsafe extern "C" fn serror_has_serrs(serr: *const SourceError) -> bool {
    // Unwrap the pointer
    let serr: &SourceError = match serr.as_ref() {
        Some(err) => err,
        None => { panic!("Given SourceError is a NULL-pointer"); },
    };

    // Now return if there are any errors
    !(*serr).errs.is_empty()
}

/// Returns if a program error has occurred.
/// 
/// # Arguments
/// - `serr`: The [`SourceError`] struct to inspect.
/// 
/// # Returns
/// True if [`serr_print_err`] would print anything, or false otherwise.
/// 
/// # Panics
/// This function can panic if the given `err` is a NULL-pointer.
#[no_mangle]
pub unsafe extern "C" fn serror_has_err(serr: *const SourceError) -> bool {
    // Unwrap the pointer
    let serr: &SourceError = match serr.as_ref() {
        Some(err) => err,
        None => { panic!("Given SourceError is a NULL-pointer"); },
    };

    // Now return if there is a message
    (*serr).msg.is_some()
}



/// Prints the source warnings in this error to stderr.
/// 
/// Note that there may be zero or more warnings at once. To discover if there are any, check [`serror_has_swarns()`].
/// 
/// # Arguments
/// - `serr`: The [`SourceError`] to print the source warnings of.
/// - `file`: Some string describing the source/filename of the source text.
/// - `source`: The physical source text, as parsed.
/// 
/// # Panics
/// This function can panic if the given `serr` is a NULL-pointer, or if `file` or `source` do not point to valid UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn serror_print_swarns(serr: *const SourceError, file: *const c_char, source: *const c_char) {
    // Unwrap the pointer
    let serr: &SourceError = match serr.as_ref() {
        Some(err) => err,
        None => { panic!("Given SourceError is a NULL-pointer"); },
    };

    // Read the file & source strings
    let file: &str = cstr_to_rust(file);
    let source: &str = cstr_to_rust(source);

    // Iterate over the warnings to print them
    for warn in &serr.warns {
        warn.prettyprint(file, source);
    }
}

/// Prints the source errors in this error to stderr.
/// 
/// Note that there may be zero or more errors at once. To discover if there are any, check [`serror_has_serrs()`].
/// 
/// # Arguments
/// - `serr`: The [`SourceError`] to print the source errors of.
/// - `file`: Some string describing the source/filename of the source text.
/// - `source`: The physical source text, as parsed.
/// 
/// # Panics
/// This function can panic if the given `serr` is a NULL-pointer, or if `file` or `source` do not point to valid UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn serror_print_serrs(serr: *const SourceError, file: *const c_char, source: *const c_char) {
    // Unwrap the pointer
    let serr: &SourceError = match serr.as_ref() {
        Some(serr) => serr,
        None => { panic!("Given SourceError is a NULL-pointer"); },
    };

    // Read the file & source strings
    let file: &str = cstr_to_rust(file);
    let source: &str = cstr_to_rust(source);

    // Iterate over the errors to print them
    for err in &serr.errs {
        err.prettyprint(file, source);
    }
}

/// Prints the error message in this error to stderr.
/// 
/// Note that there may be no error, but only source warnings- or errors. To discover if there is any, check [`serror_has_err()`].
/// 
/// # Arguments
/// - `serr`: The [`SourceError`] to print the error of.
/// 
/// # Panics
/// This function can panic if the given `serr` is a NULL-pointer.
#[no_mangle]
pub unsafe extern "C" fn serror_print_err(serr: *const SourceError) {
    init_logger();

    // Unwrap the pointer
    let serr: &SourceError = match serr.as_ref() {
        Some(err) => err,
        None => { panic!("Given SourceError is a NULL-pointer"); },
    };

    // Print the message as an error-log, if any
    if let Some(msg) = &serr.msg {
        error!("{msg}");
    }
}





/***** LIBRARY WORKFLOW *****/
/// Destructor for the Workflow.
/// 
/// SAFETY: You _must_ call this destructor yourself whenever you are done with the struct to cleanup any code. _Don't_ use any C-library free!
/// 
/// # Arguments
/// - `workflow`: The [`Workflow`] to free.
#[no_mangle]
pub unsafe extern "C" fn workflow_free(workflow: *mut Workflow) {
    init_logger();
    trace!("Destroying Workflow...");

    // Simply captute the box, then drop
    drop(Box::from_raw(workflow))
}



/// Serializes the workflow by essentially disassembling it.
/// 
/// NOTE: The given workflow is actually mutated during this call - although it is guaranteed to _not_ mutate when done (weird, no)? Anyway, this functions is read-only for all purposes except when considering multi-threaded access to `workflow`.
/// 
/// # Arguments
/// - `workflow`: The [`Workflow`] to disassemble.
/// - `assembly`: The serialized assembly of the same workflow, as a string. Don't forget to free it! Will be [`NULL`] if there is an error (see below).
/// 
/// # Returns
/// [`Null`] in all cases except when an error occurs. Then, an [`Error`]-struct is returned describing the error. Don't forget this has to be freed using [`error_free()`]!
/// 
/// # Panics
/// This function can panic if the given `workflow` is a NULL-pointer.
pub unsafe extern "C" fn workflow_disassemble(workflow: *mut Workflow, assembly: *mut *mut c_char) -> *const Error {
    // Set the output to NULL
    *assembly = std::ptr::null_mut();

    // Unwrap the input workflow
    let workflow: &mut Workflow = match workflow.as_mut() {
        Some(wf) => wf,
        None => { panic!("Given Workflow is a NULL-pointer"); },
    };

    // Take ownership of the workflow real quick
    let mut wf: Workflow = Workflow::new(SymTable::new(), vec![], HashMap::new());
    std::mem::swap(&mut wf, workflow);

    // Run the compiler traversal to serialize it
    let mut result: Vec<u8> = Vec::new();
    *workflow = match ast::do_traversal(wf, &mut result) {
        Ok(workflow) => workflow,
        Err(e) => {
            let err: Error = Error { msg: format!("Failed to print given workflow: {}", e[0]) };
            return Box::into_raw(Box::new(err));
        },
    };

    // Convert the resulting string to a C-string.
    let result: CString = match CString::new(result) {
        Ok(result) => result,
        Err(e) => {
            let err: Error = Error { msg: format!("Failed to convert disassembly to a C-compatible string: {e}") };
            return Box::into_raw(Box::new(err));
        },
    };

    // Write that in a malloc-allocated area (so C can free it), and then set it in the output
    let n_chars: usize = libc::strlen(result.as_ptr());
    let target: *mut c_char = libc::malloc(n_chars + 1) as *mut c_char;
    libc::strncpy(target, result.as_ptr(), n_chars);
    std::slice::from_raw_parts_mut(target, n_chars + 1)[n_chars] = '\0' as i8;
    *assembly = target;

    // Done, return that no error occurred
    std::ptr::null()
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
/// - `compiler`: Will point to the newly created [`Compiler`] when done. Will be [`NULL`] if there is an error (see below).
/// 
/// # Returns
/// [`Null`] in all cases except when an error occurs. Then, an [`Error`]-struct is returned describing the error. Don't forget this has to be freed using [`error_free()`]!
#[no_mangle]
pub unsafe extern "C" fn compiler_new(endpoint: *const c_char, compiler: *mut *mut Compiler) -> *const Error {
    init_logger();
    *compiler = std::ptr::null_mut();
    info!("Constructing BraneScript compiler v{}...", env!("CARGO_PKG_VERSION"));

    // Read the endpoint string
    let endpoint: &str = cstr_to_rust(endpoint);

    // Create a local threaded tokio context
    let runtime: Runtime = match Builder::new_current_thread().enable_time().enable_io().build() {
        Ok(runtime) => runtime,
        Err(e) => {
            let err: Error = Error { msg: format!("Failed to create local Tokio context: {e}") };
            return Box::into_raw(Box::new(err));
        }
    };

    // Load the package index
    let package_endpoint: String = format!("{endpoint}/graphql");
    debug!("Loading package index from '{package_endpoint}'...");
    let pindex: PackageIndex = match runtime.block_on(get_package_index(package_endpoint)) {
        Ok(index) => index,
        Err(e) => {
            let err: Error = Error { msg: format!("Failed to get package index: {e}") };
            return Box::into_raw(Box::new(err));
        },
    };

    // Load the data index
    let data_endpoint: String = format!("{endpoint}/data/info");
    debug!("Loading data index from '{data_endpoint}'...");
    let dindex: DataIndex = match runtime.block_on(get_data_index(data_endpoint)) {
        Ok(index) => index,
        Err(e) => {
            let err: Error = Error { msg: format!("Failed to get data index: {e}") };
            return Box::into_raw(Box::new(err));
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
/// SAFETY: You _must_ call this destructor yourself whenever you are done with the struct to cleanup any code. _Don't_ use any C-library free!
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
/// Note that this function changes the `compiler`'s state.
/// 
/// # Arguments
/// - `compiler`: The [`Compiler`] to compile with. Essentially this determines which previous compile state to use.
/// - `raw`: The raw BraneScript snippet to parse.
/// - `workflow`: Will point to the compiled AST. Will be [`NULL`] if there is an error (see below).
/// 
/// # Returns
/// A [`SourceError`]-struct describing the error, if any, and source warnings/errors. Don't forget this has to be freed using [`serror_free()`]!
/// 
/// # Panics
/// This function can panic if the given `compiler` points to NULL, or `endpoint` does not point to a valid UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn compiler_compile(compiler: *mut Compiler, raw: *const c_char, workflow: *mut *mut Workflow) -> *const SourceError {
    // Initialize the logger if we hadn't already
    init_logger();
    let mut err: Box<SourceError> = Box::new(SourceError { warns: vec![], errs: vec![], msg: None });
    *workflow = std::ptr::null_mut();
    info!("Compiling snippet...");



    /* INPUT */
    // Cast the Compiler pointer to a Compiler reference
    debug!("Reading compiler input...");
    let compiler: &mut Compiler = match compiler.as_mut() {
        Some(compiler) => compiler,
        None => { panic!("Given Compiler is a NULL-pointer"); },
    };

    // Get the input as a Rust string
    let raw: &str = cstr_to_rust(raw);



    /* COMPILE */
    // Compile that using `brane-ast`
    debug!("Compiling snippet...");
    let wf: Workflow = match brane_ast::compile_snippet(&mut compiler.state, raw.as_bytes(), &compiler.pindex, &compiler.dindex, &ParserOptions::bscript()) {
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

    // Write the workflow to the output
    *workflow = Box::into_raw(Box::new(wf));

    // OK, return the error struct!
    debug!("Compilation success");
    Box::into_raw(err)
}
