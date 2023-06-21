//  UTILS.rs
//    by Lut99
// 
//  Created:
//    12 Jun 2023, 13:46:12
//  Last edited:
//    21 Jun 2023, 11:32:27
//  Auto updated?
//    Yes
// 
//  Description:
//!   Defines utilities used through out the crate. These are mostly
//!   relating to testing traversals.
// 

use std::fs::{self, DirEntry, ReadDir};
use std::future::Future;
use std::path::PathBuf;

use tokio::runtime::{Builder, Runtime};


/***** CONSTANTS *****/
/// Defines the path of the tests folder.
pub const TESTS_DIR: &str = "../tests";

/// Defines the path of the packages in the tests folder.
pub const TESTS_PACKAGES_DIR: &str = "../tests/packages";
/// Defines the path of the datasets in the tests folder.
pub const TESTS_DATASETS_DIR: &str = "../tests/data";





/***** LIBRARY *****/
/// Runs a given closure on all files in the `tests` folder (see the constant defined in this function's source file).
/// 
/// # Generic arguments
/// - `F`: The function signature of the closure. It simply accepts the path and source text of a single file, and returns nothing. Instead, it can panic if the test fails.
/// 
/// # Arguments
/// - `mode`: The mode to run in. May either be 'BraneScript' or 'Bakery'.
/// - `exec`: The closure that runs code on every file in the appropriate language's text.
/// 
/// # Panics
/// This function panics if the test failed (i.e., if the files could not be found or the closure panics).
#[inline]
pub fn test_on_dsl_files<F>(mode: &'static str, exec: F)
where
    F: Fn(PathBuf, String),
{
    // Create a runtime on this thread and then do the async version
    let runtime: Runtime = Builder::new_current_thread().build().unwrap_or_else(|err| panic!("Failed to launch Tokio runtime: {}", err));

    // Run the test_on_dsl_files_async
    runtime.block_on(test_on_dsl_files_async(mode, |path, code| {
        async { exec(path, code) }
    }))
}

/// Runs a given closure on all files in the `tests` folder (see the constant defined in this function's source file).
/// 
/// # Generic arguments
/// - `F`: The function signature of the closure. It simply accepts the path and source text of a single file, and returns a future that represents the test code. If it should cause the test to fail, that future should panic.
/// 
/// # Arguments
/// - `mode`: The mode to run in. May either be 'BraneScript' or 'Bakery'.
/// - `exec`: The closure that runs code on every file in the appropriate language's text.
/// 
/// # Panics
/// This function panics if the test failed (i.e., if the files could not be found or the closure panics).
pub async fn test_on_dsl_files_async<F, R>(mode: &'static str, exec: F)
where
    F: Fn(PathBuf, String) -> R,
    R: Future<Output = ()>,
{
    // Setup some variables and checks
    let mut tests_dir: PathBuf = PathBuf::from(TESTS_DIR);
    let exts: Vec<&'static str> = match mode {
        "BraneScript" => {
            tests_dir = tests_dir.join("branescript");
            vec![ "bs", "bscript" ]
        },
        "Bakery"      => {
            tests_dir = tests_dir.join("bakery");
            vec![ "bakery" ]
        },
        val           => { panic!("Unknown mode '{}'", val); }
    };

    // Try to open the folder
    let dir = match fs::read_dir(&tests_dir) {
        Ok(dir)  => dir,
        Err(err) => { panic!("Failed to list tests directory '{}': {}", tests_dir.display(), err); }
    };

    // Start a 'recursive' process where we run all '*.bscript' files.
    let mut todo: Vec<(PathBuf, ReadDir)> = vec![ (tests_dir, dir) ];
    let mut counter = 0;
    while !todo.is_empty() {
        // Get the next directory to search
        let (path, dir): (PathBuf, ReadDir) = todo.pop().unwrap();

        // Iterate through it
        for entry in dir {
            // Attempt to unwrap the entry
            let entry: DirEntry = match entry {
                Ok(entry) => entry,
                Err(err)  => { panic!("Failed to read entry in directory '{}': {}", path.display(), err); }
            };

            // Check whether it's a directory or not
            if entry.path().is_file() {
                // Check if it ends with '.bscript'
                if let Some(ext) = entry.path().extension() {
                    if exts.contains(&ext.to_str().unwrap_or("")) {
                        // Read the file to a buffer
                        let code: String = match fs::read_to_string(entry.path()) {
                            Ok(code) => code,
                            Err(err) => { panic!("Failed to read {} file '{}': {}", mode, entry.path().display(), err); },
                        };

                        // Run the closure on this file
                        exec(entry.path(), code).await;
                        counter += 1;
                    } else if entry.path().extension().is_some() && entry.path().extension().unwrap() != "yml" && entry.path().extension().unwrap() != "yaml" {
                        println!("Ignoring entry '{}' in '{}' (does not have extensions {})", entry.path().display(), path.display(), exts.iter().map(|e| format!("'.{e}'")).collect::<Vec<String>>().join(", "));
                    }
                } else {
                    println!("Ignoring entry '{}' in '{}' (cannot extract extension)", entry.path().display(), path.display());
                }

            } else if entry.path().is_dir() {
                // Recurse, i.e., list and add to the todo list
                let new_dir = match fs::read_dir(entry.path()) {
                    Ok(dir)  => dir,
                    Err(err) => { panic!("Failed to list nested tests directory '{}': {}", entry.path().display(), err); }
                };
                if todo.len() == todo.capacity() { todo.reserve(todo.capacity()); }
                todo.push((entry.path(), new_dir));

            } else {
                // Dunno what to do with it
                println!("Ignoring entry '{}' in '{}' (unknown entry type)", entry.path().display(), path.display());
            }
        }
    }

    // Do a finishing debug print
    if counter == 0 {
        println!("No files to run.");
    } else {
        println!("Tested {counter} files in total");
    }
}
