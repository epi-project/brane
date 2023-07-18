//  LIB.rs
//    by Lut99
// 
//  Created:
//    14 Jun 2023, 17:38:09
//  Last edited:
//    18 Jul 2023, 09:32:12
//  Auto updated?
//    Yes
// 
//  Description:
//!   Wrapper around `brane-cli` that provides C-bindings for interacting with
//!   a remote backend. This allows C-programs to act as a BRANE client.
//!   
//!   The basics of how to do this are followed from:
//!   http://blog.asleson.org/2021/02/23/how-to-writing-a-c-shared-library-in-rust/
// 

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Arc, Once};
use std::time::Instant;

use humanlog::{DebugMode, HumanLogger};
use log::{debug, error, info, trace, warn};
use parking_lot::{Mutex, MutexGuard};
use tokio::runtime::{Builder, Runtime};

use brane_ast::{CompileResult, Error as AstError, ParserOptions, Warning as AstWarning};
use brane_ast::ast::Workflow;
use brane_ast::state::CompileState;
use brane_ast::traversals::print::ast;
use brane_cli::data::download_data;
use brane_cli::run::{initialize_instance, run_instance, InstanceVmState};
use brane_exe::FullValue;
use brane_tsk::api::{get_data_index, get_package_index};
use specifications::data::{AccessKind, DataIndex};
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
pub struct SourceError<'f, 's> {
    /// The filename of the file we are referencing.
    file   : &'f str,
    /// The complete source we attempted to parse.
    source : &'s str,

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
/// 
/// # Panics
/// This function can panic if the given `serr` is a NULL-pointer.
#[no_mangle]
pub unsafe extern "C" fn serror_print_swarns(serr: *const SourceError) {
    // Unwrap the pointer
    let serr: &SourceError = match serr.as_ref() {
        Some(err) => err,
        None => { panic!("Given SourceError is a NULL-pointer"); },
    };

    // Iterate over the warnings to print them
    for warn in &serr.warns {
        warn.prettyprint(serr.file, serr.source);
    }
}

/// Prints the source errors in this error to stderr.
/// 
/// Note that there may be zero or more errors at once. To discover if there are any, check [`serror_has_serrs()`].
/// 
/// # Arguments
/// - `serr`: The [`SourceError`] to print the source errors of.
/// 
/// # Panics
/// This function can panic if the given `serr` is a NULL-pointer.
#[no_mangle]
pub unsafe extern "C" fn serror_print_serrs(serr: *const SourceError) {
    // Unwrap the pointer
    let serr: &SourceError = match serr.as_ref() {
        Some(serr) => serr,
        None => { panic!("Given SourceError is a NULL-pointer"); },
    };

    // Iterate over the errors to print them
    for err in &serr.errs {
        err.prettyprint(serr.file, serr.source);
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





/***** LIBRARY PACKAGEINDEX *****/
/// Constructs a new [`PackageIndex`] that lists the available packages in a remote instance.
/// 
/// # Arguments
/// - `endpoint`: The remote API-endpoint to read the packages from. The path (`/graphql`) will be deduced and needn't be given, just the host and port.
/// - `pindex`: Will point to the newly created [`PackageIndex`] when done. Will be [`NULL`] if there is an error (see below).
/// 
/// # Returns
/// [`Null`] in all cases except when an error occurs. Then, an [`Error`]-struct is returned describing the error. Don't forget this has to be freed using [`error_free()`]!
/// 
/// # Panics
/// This function can panic if the given `endpoint` does not point to a valud UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn pindex_new_remote(endpoint: *const c_char, pindex: *mut *mut Arc<Mutex<PackageIndex>>) -> *const Error {
    init_logger();
    *pindex = std::ptr::null_mut();
    info!("Collecting package index...");

    // Read the input string
    let endpoint: &str = cstr_to_rust(endpoint);

    // Create a local threaded tokio context
    let runtime: Runtime = match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(e) => {
            let err: Error = Error { msg: format!("Failed to create local Tokio context: {e}") };
            return Box::into_raw(Box::new(err));
        }
    };

    // Build the package index around it
    let addr: String = format!("{endpoint}/graphql");
    let index: PackageIndex = match runtime.block_on(get_package_index(&addr)) {
        Ok(index) => index,
        Err(e) => {
            let err: Error = Error { msg: format!("Failed to read package index from '{addr}': {e}") };
            return Box::into_raw(Box::new(err));
        }
    };

    // Store it and we're done
    debug!("Found {} packages", index.packages.len());
    *pindex = Box::into_raw(Box::new(Arc::new(Mutex::new(index))));
    std::ptr::null()
}

/// Destructor for the PackageIndex.
/// 
/// SAFETY: You _must_ call this destructor yourself whenever you are done with the struct to cleanup any code. _Don't_ use any C-library free!
/// 
/// # Arguments
/// - `pindex`: The [`PackageIndex`] to free.
#[no_mangle]
pub unsafe extern "C" fn pindex_free(pindex: *mut Arc<Mutex<PackageIndex>>) {
    init_logger();
    trace!("Destroying PackageIndex...");

    // Simply capture the box, then drop
    drop(Box::from_raw(pindex))
}





/***** LIBRARY DATAINDEX *****/
/// Constructs a new [`DataIndex`] that lists the available datasets in a remote instance.
/// 
/// # Arguments
/// - `endpoint`: The remote API-endpoint to read the datasets from. The path (`/data/info`) will be deduced and needn't be given, just the host and port.
/// - `dindex`: Will point to the newly created [`DataIndex`] when done. Will be [`NULL`] if there is an error (see below).
/// 
/// # Returns
/// [`Null`] in all cases except when an error occurs. Then, an [`Error`]-struct is returned describing the error. Don't forget this has to be freed using [`error_free()`]!
/// 
/// # Panics
/// This function can panic if the given `endpoint` does not point to a valud UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn dindex_new_remote(endpoint: *const c_char, dindex: *mut *mut Arc<Mutex<DataIndex>>) -> *const Error {
    init_logger();
    *dindex = std::ptr::null_mut();
    info!("Collecting data index...");

    // Read the input string
    let endpoint: &str = cstr_to_rust(endpoint);

    // Create a local threaded tokio context
    let runtime: Runtime = match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(e) => {
            let err: Error = Error { msg: format!("Failed to create local Tokio context: {e}") };
            return Box::into_raw(Box::new(err));
        }
    };

    // Build the package index around it
    let addr: String = format!("{endpoint}/data/info");
    let index: DataIndex = match runtime.block_on(get_data_index(&addr)) {
        Ok(index) => index,
        Err(e) => {
            let err: Error = Error { msg: format!("Failed to read data index from '{addr}': {e}") };
            return Box::into_raw(Box::new(err));
        }
    };

    // Store it and we're done
    debug!("Found {} datasets", index.iter().count());
    *dindex = Box::into_raw(Box::new(Arc::new(Mutex::new(index))));
    std::ptr::null()
}

/// Destructor for the DataIndex.
/// 
/// SAFETY: You _must_ call this destructor yourself whenever you are done with the struct to cleanup any code. _Don't_ use any C-library free!
/// 
/// # Arguments
/// - `dindex`: The [`DataIndex`] to free.
#[no_mangle]
pub unsafe extern "C" fn dindex_free(dindex: *mut Arc<Mutex<DataIndex>>) {
    init_logger();
    trace!("Destroying DataIndex...");

    // Simply capture the box, then drop
    drop(Box::from_raw(dindex))
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

    // Simply capture the box, then drop
    drop(Box::from_raw(workflow))
}



/// Serializes the workflow by essentially disassembling it.
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
#[no_mangle]
pub unsafe extern "C" fn workflow_disassemble(workflow: *const Workflow, assembly: *mut *mut c_char) -> *const Error {
    // Set the output to NULL
    init_logger();
    *assembly = std::ptr::null_mut();
    info!("Generating workflow assembly...");

    // Unwrap the input workflow
    let workflow: &Workflow = match workflow.as_ref() {
        Some(wf) => wf,
        None => { panic!("Given Workflow is a NULL-pointer"); },
    };

    // Run the compiler traversal to serialize it
    let mut result: Vec<u8> = Vec::new();
    if let Err(e) = ast::do_traversal(workflow, &mut result) {
        let err: Error = Error { msg: format!("Failed to print given workflow: {}", e[0]) };
        return Box::into_raw(Box::new(err));
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
    pindex : Arc<Mutex<PackageIndex>>,
    /// The data index to use for compilation.
    dindex : Arc<Mutex<DataIndex>>,

    /// The additional, total collected source that we are working with
    source : String,
    /// The compile state to use in between snippets.
    state  : CompileState,
}



/// Constructor for the Compiler.
/// 
/// # Arguments
/// - `pindex`: The [`PackageIndex`] to resolve package references in the snippets with.
/// - `dindex`: The [`DataIndex`] to resolve dataset references in the snippets with.
/// - `compiler`: Will point to the newly created [`Compiler`] when done. Will be [`NULL`] if there is an error (see below).
/// 
/// # Returns
/// [`Null`] in all cases except when an error occurs. Then, an [`Error`]-struct is returned describing the error. Don't forget this has to be freed using [`error_free()`]!
/// 
/// # Panics
/// This function can panic if the given `pindex` or `dindex` points to NULL.
#[no_mangle]
pub unsafe extern "C" fn compiler_new(pindex: *const Arc<Mutex<PackageIndex>>, dindex: *const Arc<Mutex<DataIndex>>, compiler: *mut *mut Compiler) -> *const Error {
    init_logger();
    *compiler = std::ptr::null_mut();
    info!("Constructing BraneScript compiler v{}...", env!("CARGO_PKG_VERSION"));

    // Read the indices
    let pindex: &Arc<Mutex<PackageIndex>> = match pindex.as_ref() {
        Some(index) => index,
        None => { panic!("Given PackageIndex is a NULL-pointer"); },
    };
    let dindex: &Arc<Mutex<DataIndex>> = match dindex.as_ref() {
        Some(index) => index,
        None => { panic!("Given DataIndex is a NULL-pointer"); },
    };

    // Construct a new Compiler and return it as a pointer
    *compiler = Box::into_raw(Box::new(Compiler {
        pindex : pindex.clone(),
        dindex : dindex.clone(),

        source : String::new(),
        state  : CompileState::new(),
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
/// - `what`: Some string describing what we are compiling (e.g., a file, `<intern>`, a cell, etc.)
/// - `raw`: The raw BraneScript snippet to parse.
/// - `workflow`: Will point to the compiled AST. Will be [`NULL`] if there is an error (see below).
/// 
/// # Returns
/// A [`SourceError`]-struct describing the error, if any, and source warnings/errors.
/// 
/// ## SAFETY
/// Be aware that the returned [`SourceError`] refers the the given `compiler` and `what`. Freeing any of those two and then using the [`SourceError`] _will_ lead to undefined behaviour.
/// 
/// You _must_ free this [`SourceError`] using [`serror_free()`], since its allocated using Rust internals and cannot be deallocated directly using `malloc`. Note, however, that it's safe to call [`serror_free()`] _after_ freeing `compiler` or `what` (but that's the only function).
/// 
/// # Panics
/// This function can panic if the given `compiler` points to NULL, or `what`/`raw` does not point to a valid UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn compiler_compile(compiler: *mut Compiler, what: *const c_char, raw: *const c_char, workflow: *mut *mut Workflow) -> *const SourceError<'static, 'static> {
    // Initialize the logger if we hadn't already
    init_logger();
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
    let what: &str = cstr_to_rust(what);
    let raw: &str = cstr_to_rust(raw);

    // Create the error already
    let mut serr: Box<SourceError> = Box::new(SourceError { file: what, source: raw, warns: vec![], errs: vec![], msg: None });



    /* COMPILE */
    debug!("Compiling snippet...");

    // Append the source we keep track of
    compiler.source.push_str(raw);
    compiler.source.push('\n');
    serr.source = &compiler.source;

    // Compile that using `brane-ast`
    let wf: Workflow = {
        // Acquire locks on the indices
        let pindex: MutexGuard<PackageIndex> = compiler.pindex.lock();
        let dindex: MutexGuard<DataIndex> = compiler.dindex.lock();

        // Run the snippet
        match brane_ast::compile_snippet(&mut compiler.state, compiler.source.as_bytes(), &*pindex, &*dindex, &ParserOptions::bscript()) {
            CompileResult::Workflow(workflow, warns) => {
                serr.warns = warns;
                workflow
            },

            CompileResult::Eof(e) => {
                serr.errs = vec![ e ];
                return Box::into_raw(serr);
            },
            CompileResult::Err(errs) => {
                serr.errs = errs;
                return Box::into_raw(serr);
            },

            CompileResult::Program(_, _)    |
            CompileResult::Unresolved(_, _) => { unreachable!(); },
        }
    };

    // Write the workflow to the output
    *workflow = Box::into_raw(Box::new(wf));

    // OK, return the error struct!
    debug!("Compilation success");
    Box::into_raw(serr)
}





/***** FULL VALUE *****/
/// Destructor for the FullValue.
/// 
/// SAFETY: You _must_ call this destructor yourself whenever you are done with the struct to cleanup any code. _Don't_ use any C-library free!
/// 
/// # Arguments
/// - `fvalue`: The [`FullValue`] to free.
#[no_mangle]
pub unsafe extern "C" fn fvalue_free(fvalue: *mut FullValue) {
    init_logger();
    trace!("Destroying FullValue...");

    // Take ownership of the value and then drop it to destroy
    drop(Box::from_raw(fvalue));
}





/***** VIRTUAL MACHINE *****/
/// Defines a BRANE instance virtual machine.
/// 
/// This can run a compiled workflow on a running instance.
pub struct VirtualMachine {
    /// The endpoint to connect to for downloading registries
    api_endpoint : String,
    /// The endpoint to connect to when running.
    drv_endpoint : String,
    /// The dataset to download directories to.
    data_dir     : String,
    /// The state of everything we need to know about the virtual machine
    state        : InstanceVmState,
}



/// Constructor for the VirtualMachine.
/// 
/// # Arguments
/// - `api_endpoint`: The Brane API endpoint to connect to to download available registries and all that.
/// - `drv_endpoint`: The BRANE driver endpoint to connect to to execute stuff.
/// - `data_dir`: The directory to download resulting datasets to.
/// - `pindex`: The [`PackageIndex`] to resolve package references in the snippets with.
/// - `dindex`: The [`DataIndex`] to resolve dataset references in the snippets with.
/// - `virtual_machine`: Will point to the newly created [`VirtualMachine`] when done. Will be [`NULL`] if there is an error (see below).
/// 
/// # Returns
/// An [`Error`]-struct that contains the error occurred, or [`NULL`] otherwise.
/// 
/// # Panics
/// This function can panic if the given `pindex` or `dindex` are NULL, or if the given `api_endpoint`, `drv_endpoint` or `data_dir` do not point to a valid UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn vm_new(api_endpoint: *const c_char, drv_endpoint: *const c_char, data_dir: *const c_char, pindex: *const Arc<Mutex<PackageIndex>>, dindex: *const Arc<Mutex<DataIndex>>, vm: *mut *mut VirtualMachine) -> *const Error {
    init_logger();
    *vm = std::ptr::null_mut();
    info!("Constructing BraneScript virtual machine v{}...", env!("CARGO_PKG_VERSION"));

    // Read the endpoints
    let api_endpoint: &str = cstr_to_rust(api_endpoint);
    let drv_endpoint: &str = cstr_to_rust(drv_endpoint);
    let data_dir: &str = cstr_to_rust(data_dir);

    // Read the indices
    let pindex: &Arc<Mutex<PackageIndex>> = match pindex.as_ref() {
        Some(index) => index,
        None => { panic!("Given PackageIndex is a NULL-pointer"); },
    };
    let dindex: &Arc<Mutex<DataIndex>> = match dindex.as_ref() {
        Some(index) => index,
        None => { panic!("Given DataIndex is a NULL-pointer"); },
    };

    // Prepare a tokio environment
    let runtime: Runtime = match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(e) => {
            let err: Error = Error { msg: format!("Failed to create local Tokio context: {e}") };
            return Box::into_raw(Box::new(err));
        }
    };

    // Prepare the state
    let state: InstanceVmState = match runtime.block_on(initialize_instance(drv_endpoint, pindex.clone(), dindex.clone(), None, ParserOptions::bscript())) {
        Ok(state) => state,
        Err(e)  => {
            let err: Error = Error { msg: format!("Failed to create new InstanceVmState: {e}") };
            return Box::into_raw(Box::new(err));
        },
    };

    // OK, return the new thing
    *vm = Box::into_raw(Box::new(VirtualMachine {
        api_endpoint : api_endpoint.into(),
        drv_endpoint : drv_endpoint.into(),
        data_dir     : data_dir.into(),
        state,
    }));
    debug!("Virtual machine created");
    std::ptr::null()
}

/// Destructor for the VirtualMachine.
/// 
/// SAFETY: You _must_ call this destructor yourself whenever you are done with the struct to cleanup any code. _Don't_ use any C-library free!
/// 
/// # Arguments
/// - `vm`: The [`VirtualMachine`] to free.
#[no_mangle]
pub unsafe extern "C" fn vm_free(vm: *mut VirtualMachine) {
    init_logger();
    trace!("Destroying VirtualMachine...");

    // Take ownership of the VM and then drop it to destroy
    drop(Box::from_raw(vm));
}



/// Runs the given code snippet on the backend instance.
/// 
/// # Arguments
/// - `vm`: The [`VirtualMachine`] that we execute with. This determines which backend to use.
/// - `workflow`: The compiled workflow to execute.
/// - `result`: A [`FullValue`] which represents the return value of the workflow. Will be [`NULL`] if there is an error (see below).
/// 
/// # Returns
/// An [`Error`]-struct that contains the error occurred, or [`NULL`] otherwise.
/// 
/// # Panics
/// This function may panic if the input `vm` or `workflow` pointed to a NULL-pointer.
#[no_mangle]
pub unsafe extern "C" fn vm_run(vm: *mut VirtualMachine, workflow: *const Workflow, result: *mut *mut FullValue) -> *const Error {
    init_logger();
    *result = std::ptr::null_mut();
    info!("Executing workflow on virtual machine...");
    let start: Instant = Instant::now();

    // Unwrap the VM
    let vm: &mut VirtualMachine = match vm.as_mut() {
        Some(vm) => vm,
        None => { panic!("Given VirtualMachine is a NULL-pointer"); },
    };
    // Unwrap the workflow
    let workflow: &Workflow = match workflow.as_ref() {
        Some(workflow) => workflow,
        None => { panic!("Given Workflow is a NULL-pointer"); },
    };

    // Prepare a tokio environment
    let runtime: Runtime = match Builder::new_current_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(e) => {
            let err: Box<Error> = Box::new(Error { msg: format!("Failed to create local Tokio context: {e}") });
            return Box::into_raw(err);
        }
    };

    // Run the state
    debug!("Executing snippet...");
    let value: FullValue = match runtime.block_on(run_instance(&vm.drv_endpoint, &mut vm.state, workflow, false)) {
        Ok(value) => value,
        Err(e) => {
            let err: Box<Error> = Box::new(Error { msg: format!("Failed to run workflow on '{}': {}", vm.drv_endpoint, e) });
            return Box::into_raw(err);
        },
    };

    // If the value is a dataset, then download the data on top of it
    if let FullValue::Data(d) = &value {
        debug!("Downloading dataset...");

        // Refresh the data index and get the access list for this dataset
        let access: HashMap<String, AccessKind> = {
            // Get a mutable lock to do so
            let mut dindex: MutexGuard<DataIndex> = vm.state.dindex.lock();

            // Simply load it again
            *dindex = match runtime.block_on(get_data_index(&vm.api_endpoint)) {
                Ok(index) => index,
                Err(e) => {
                    let err: Box<Error> = Box::new(Error { msg: format!("Failed to refresh data index: {e}") });
                    return Box::into_raw(err);
                },
            };

            // Fetch the correct map
            match dindex.get(d) {
                Some(info) => info.access.clone(),
                None => {
                    let err: Box<Error> = Box::new(Error { msg: format!("Resulting dataset '{d}' is not at any location") });
                    return Box::into_raw(err);
                },
            }
        };

        // Run the process funtion
        let res: Option<AccessKind> = match runtime.block_on(download_data(&vm.api_endpoint, &None, &vm.data_dir, d, &access)) {
            Ok(res) => res,
            Err(e) => {
                let err: Box<Error> = Box::new(Error { msg: format!("Failed to download resulting data from '{}': {}", vm.api_endpoint, e) });
                return Box::into_raw(err);
            },
        };
        if let Some(AccessKind::File { path }) = res {
            info!("Downloaded dataset to '{}'", path.display());
        }

    } else if matches!(value, FullValue::IntermediateResult(_)) {
        // Emit a warning
        warn!("Cannot download intermediate result");
    }

    // Store it and we're done!
    debug!("Done (execution took {:.2}s)", start.elapsed().as_secs_f32());
    *result = Box::into_raw(Box::new(value));
    std::ptr::null()
}
