use std::fs::File;
use std::net::IpAddr;
use std::path::PathBuf;

use brane_cfg::proxy::ProxyProtocol;
use brane_ctl::spec::{
    DownloadServicesSubcommand, GenerateBackendSubcommand, GenerateCertsSubcommand, GenerateNodeSubcommand, InclusiveRange, Pair,
    PolicyInputLanguage, ResolvableNodeKind, StartSubcommand, VersionFix, API_DEFAULT_VERSION,
};
use brane_tsk::docker::ClientVersion;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::generate;
use clap_complete::shells::Shell::*;
use humantime::Duration as HumanDuration;
use jsonwebtoken::jwk::KeyAlgorithm;
// use log::error;
use specifications::address::{Address, AddressOpt};
use specifications::arch::Arch;
use specifications::package::Capability;
use specifications::version::Version;

include!("../cli.rs");

fn main() {
    let mut command = Arguments::command();

    for (filename, shell) in [("branectl.bash", Bash), ("branectl.fish", Fish), ("branectl.zsh", Zsh)] {
        let mut file = File::create(filename).expect("Could not open/create completions file");
        generate(shell, &mut command, "branectl", &mut file);
    }
}
