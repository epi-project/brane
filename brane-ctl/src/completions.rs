#![allow(dead_code)]
use std::fs::File;

use clap::CommandFactory;
use clap_complete::generate;
use clap_complete::shells::Shell::*;

mod cli;
use cli::*;

fn main() {
    let mut command = Arguments::command();

    for (filename, shell) in [("branectl.bash", Bash), ("branectl.fish", Fish), ("branectl.zsh", Zsh)] {
        let mut file = File::create(filename).expect("Could not open/create completions file");
        generate(shell, &mut command, "branectl", &mut file);
    }
}
