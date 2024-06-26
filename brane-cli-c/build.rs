//  BUILD.rs
//    by Lut99
//
//  Created:
//    04 Mar 2024, 13:01:03
//  Last edited:
//    04 Mar 2024, 13:29:09
//  Auto updated?
//    Yes
//
//  Description:
//!   Build script to generate C headers from the Rust codebase.
//!   
//!   Uses [cbindgen](https://github.com/mozilla/cbindgen) to achieve this.
//

// use std::fs;
// use std::panic::catch_unwind;
// use std::path::PathBuf;

// use cbindgen::{generate_with_config, Bindings, Config};


/***** ENTRYPOINT *****/
fn main() {
    // // Emit we only need to run this if the source changed
    // println!("cargo:rerun-if-changed={}", concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
    // println!("cargo:rerun-if-changed={}", concat!(env!("CARGO_MANIFEST_DIR"), "/cbindgen.toml"));

    // // Prepare the configuration for the cbindings
    // let crate_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // let config_path: PathBuf = crate_path.join("cbindgen.toml");
    // let config: Config = match Config::from_file(&config_path) {
    //     Ok(config) => config,
    //     Err(err) => {
    //         eprintln!("ERROR: Failed to read cbindgen config file '{}': {}", config_path.display(), err);
    //         std::process::exit(1);
    //     },
    // };

    // // Now attempt to open the file
    // let bindings: Bindings = match generate_with_config(env!("CARGO_MANIFEST_DIR"), config) {
    //     Ok(bindings) => bindings,
    //     Err(err) => {
    //         eprintln!("ERROR: Failed to generate C bindings: {err}");
    //         std::process::exit(1);
    //     },
    // };

    // // Create the output dir
    // let include_path: PathBuf = crate_path.join("include");
    // if !include_path.exists() {
    //     if let Err(err) = fs::create_dir_all(&include_path) {
    //         eprintln!("ERROR: Failed to create directory '{}': {}", include_path.display(), err);
    //         std::process::exit(1);
    //     }
    // }

    // // Write to the output
    // let out_path: PathBuf = include_path.join("brane_cli.h");
    // let out_path2: PathBuf = out_path.clone();
    // if catch_unwind(move || bindings.write_to_file(out_path)).is_err() {
    //     eprintln!("ERROR: Failed to write C bindings to '{}' (see output above)", out_path2.display());
    //     std::process::exit(1);
    // }
}
