use std::fs::File;

use clap::CommandFactory;
use clap_complete::generate;
use clap_complete::shells::Shell::*;

mod cli;
use cli::*;

fn main() {
    let mut command = Cli::command();

    for (filename, shell) in [("brane.bash", Bash), ("brane.fish", Fish), ("brane.zsh", Zsh)] {
        let mut file = File::create(filename).expect("Could not open/create completions file");
        generate(shell, &mut command, "brane", &mut file);
    }
}
