//  INPUT.rs
//    by Lut99
// 
//  Created:
//    06 Jun 2023, 18:38:50
//  Last edited:
//    07 Jun 2023, 09:07:07
//  Auto updated?
//    Yes
// 
//  Description:
//!   Contains functions for prompting the user in the various user-facing
//!   executables.
// 

use std::borrow::Cow;
use std::error;
use std::ffi::OsString;
use std::fmt::{Display, Formatter, Result as FResult};
use std::fs::{self, DirEntry, ReadDir};
use std::path::PathBuf;

use dialoguer::{Completion, Input, Select};
use dialoguer::theme::ColorfulTheme;
use log::warn;


/***** ERRORS *****/
/// Defines the errors that may occur when running any of the input functions.
#[derive(Debug)]
pub enum Error {
    /// Failed to run a select prompt
    Select { n_opts: usize, err: std::io::Error },
    /// Failed to run a text prompt
    Text { err: std::io::Error },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            Select { n_opts, .. } => write!(f, "Failed to prompt the user (you!) to select one of {n_opts} options"),
            Text { .. }           => write!(f, "Failed to prompt the user (you!) for a string input"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use Error::*;
        match self {
            Select { err, .. } => Some(err),
            Text { err, .. }   => Some(err),
        }
    }
}





/***** AUTOCOMPLETERS *****/
/// Autocompletes files
#[derive(Clone, Debug)]
pub struct FileAutocompleter;
impl Completion for FileAutocompleter {
    fn get(&self, input: &str) -> Option<String> {
        // Get the input as a directory and some filter in that directory
        let (dir, filter): (&str, &str) = match input.rfind('/') {
            Some(pos) => (&input[..pos + 1], &input[pos + 1..]),
            None      => ("./", input),
        };

        // Attempt to find all entries that are allowed by the filter in that directory
        let mut targets: Vec<String> = vec![];
        let entries: ReadDir = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(err) => {
                warn!("Failed to read directory '{dir}': {err}");
                return None;
            },
        };
        for (i, entry) in entries.enumerate() {
            // Unwrap the entry
            let entry: DirEntry = match entry {
                Ok(entry) => entry,
                Err(err) => {
                    warn!("Failed to unwrap directory '{dir}' entry {i}: {err}");
                    return None;
                },
            };

            // Filter the entry
            let entry: OsString = entry.file_name();
            let entry: Cow<str> = entry.to_string_lossy();
            if entry.len() < filter.len() || &entry[..filter.len()] != filter { continue; }

            // Otherwise, add it as possibility
            targets.push(entry.into());
        }

        // We now only complete if there is exactly one entry
        if targets.len() == 1 {
            Some(targets.swap_remove(0))
        } else {
            None
        }
    }
}





/***** LIBRARY *****/
/// Prompts the user for an input path.
/// 
/// # Arguments
/// - `prompt`: The prompt to display to the user.
/// - `default`: Any default path to give, or else [`None`].
/// 
/// # Returns
/// The user's chosen path.
/// 
/// # Errors
/// This function errors if we failed to interact with the user.
pub fn input_path(prompt: impl Display, default: Option<impl Into<PathBuf>>) -> Result<PathBuf, Error> {
    // Construct the prompt
    let theme: ColorfulTheme = ColorfulTheme::default();
    let mut input: Input<String> = Input::with_theme(&theme);
    input.with_prompt(prompt.to_string())
        .completion_with(&FileAutocompleter);
    if let Some(default) = default {
        input.default(default.into().to_string_lossy().into());
    }

    // Run the prompt
    match input.interact() {
        Ok(path) => Ok(path.into()),
        Err(err) => Err(Error::Text { err }),
    }
}



/// Prompts the user to select on the given values.
/// 
/// # Arguments
/// - `prompt`: The prompt to display to the user.
/// - `options`: A list of options to select from.
/// 
/// # Returns
/// The selected option.
/// 
/// # Errors
/// This function errors if we failed to interact with the user.
pub fn select<D: Display>(prompt: impl Display, options: impl IntoIterator<Item = D>) -> Result<D, Error> {
    // Collect the options
    let mut options: Vec<D> = options.into_iter().collect();

    // Construct the prompt
    let theme: ColorfulTheme = ColorfulTheme::default();
    let mut input: Select = Select::with_theme(&theme);
    input.with_prompt(prompt.to_string())
        .default(0)
        .items(&options)
        .report(true);

    // Run it
    match input.interact() {
        Ok(index) => Ok(options.swap_remove(index)),
        Err(err)  => Err(Error::Select { n_opts: options.len(), err }),
    }
}