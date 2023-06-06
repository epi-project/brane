//  INPUT.rs
//    by Lut99
// 
//  Created:
//    06 Jun 2023, 18:38:50
//  Last edited:
//    06 Jun 2023, 19:18:53
//  Auto updated?
//    Yes
// 
//  Description:
//!   Contains functions for prompting the user in the various user-facing
//!   executables.
// 

use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::path::PathBuf;

// use dialoguer::{Input, Select};
// use dialoguer::theme::ColorfulTheme;
use inquire::{Select, Text};
use inquire::autocompletion::{Autocomplete, Replacement};


/***** ERRORS *****/
/// Defines the errors that may occur when running any of the input functions.
#[derive(Debug)]
pub enum Error {
    /// Failed to run a select prompt
    Select { n_opts: usize, err: inquire::InquireError },
    /// Failed to run a text prompt
    Text { err: inquire::InquireError },
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

impl Autocomplete for FileAutocompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
        
    }

    fn get_completion(&mut self, input: &str, highlighted_suggestion: Option<String>) -> Result<Replacement, inquire::CustomUserError> {
        
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
    // // Construct the prompt
    // let theme: ColorfulTheme = ColorfulTheme::default();
    // let mut input: Input = Input::with_theme(&theme)
    //     .with_prompt(prompt.to_string())
    //     .

    // Construct the prompt
    let mut text: Text = Text::new(&prompt.to_string())
        .with_autocomplete(FileAutocompleter);
    if let Some(default) = default {
        text = text.with_default(default.into().to_string_lossy().as_ref());
    }

    // Run it
    match text.prompt() {
        Ok(result) => Ok(result.into()),
        Err(err)   => Err(Error::Text { err }),
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
    let options: Vec<D> = options.into_iter().collect();
    let n_opts: usize = options.len();

    // // Construct the prompt
    // let theme: ColorfulTheme = ColorfulTheme::default();
    // let mut input: Select = Select::with_theme(&theme);
    // input.with_prompt(prompt.to_string())
    //     .default(0)
    //     .items(&options)
    //     .report(true);

    // // Run it
    // match input.interact() {
    //     Ok(index) => Ok(options.swap_remove(index)),
    //     Err(err)  => Err(Error::Select { n_opts: options.len(), err }),
    // }

    // Construct the prompt
    let select: Select<D> = Select::new(&prompt.to_string(), options.into_iter().collect());

    // Run it!
    match select.prompt() {
        Ok(result) => Ok(result),
        Err(err)   => Err(Error::Select { n_opts, err }),
    }
}
