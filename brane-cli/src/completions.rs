
include!("cli.rs");

use std::fs::File;
use std::path::PathBuf;
use brane_cli::spec::{Hostname, VersionFix, API_DEFAULT_VERSION};
use brane_tsk::docker::ClientVersion;
use brane_tsk::spec::AppId;
use clap::{Parser, CommandFactory};
use specifications::arch::Arch;
use specifications::version::Version as SemVersion;
use clap_complete::{generate, shells::Shell::*};


fn main() {
    let mut command = Cli::command();

    for (filename, shell) in [("brane.bash", Bash), ("brane.fish", Fish), ("brane.zsh", Zsh)] {
        let mut file = File::create(filename).expect("Could not open/create completions file");
        generate(shell, &mut command, "brane", &mut file);
    }
}
