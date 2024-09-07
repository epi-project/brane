//  WIZARD.rs
//    by Lut99
//
//  Created:
//    01 Jun 2023, 12:43:20
//  Last edited:
//    07 Mar 2024, 09:54:57
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements a CLI wizard for setting up nodes, making the process
//!   _even_ easier.
//

use std::borrow::Cow;
use std::collections::HashMap;
use std::error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::fs::{self, canonicalize, File};
use std::io::Write as _;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::{Path, PathBuf};

use brane_cfg::info::Info;
use brane_cfg::infra::InfraLocation;
use brane_cfg::node::{self, CentralConfig, CentralPaths, CentralServices, ExternalService, NodeConfig, NodeKind, NodeSpecificConfig, PrivateOrExternalService, PrivateService, PublicService};
use brane_cfg::proxy::{ForwardConfig, ProxyConfig, ProxyProtocol};
use brane_shr::input::{confirm, input, input_map, input_path, select, select_enum, FileHistory};
use console::style;
use dirs_2::config_dir;
use enum_debug::EnumDebug as _;
use log::{debug, info};
use specifications::address::{Address, Host};
use validator::{FromStrValidator, MapValidator, PortValidator, RangeValidator};

pub mod validator;

use crate::spec::InclusiveRange;

type PortRangeValidator = RangeValidator<PortValidator>;
type AddressValidator = FromStrValidator<Address>;
type HostValidator = FromStrValidator<Host>;
type PortMapValidator = MapValidator<PortValidator, AddressValidator>;

type LocationId = String;
type LocationIdValidator = FromStrValidator<LocationId>;
type LocationMapValidator = MapValidator<LocationIdValidator, HostValidator>;

static REG_PORT: u16 = 50151;
static JOB_PORT: u16 = 50152;

/***** HELPER MACROS *****/
/// Generates a FileHistory that points to some branectl-specific directory in the [`config_dir()`].
macro_rules! hist {
    ($name:literal) => {{
        let hist = FileHistory::new(config_dir().unwrap().join("branectl").join("history").join($name));
        debug!("{hist:?}");
        hist
    }};
}

/// Writes a few lines that generate a directory, with logging statements.
///
/// # Arguments
/// - `[$name, $value]`: The name and subsequent value of the variable that contains the given path.
macro_rules! generate_dir {
    ($value:ident) => {
        if !$value.exists() {
            debug!("Generating '{}'...", $value.display());
            if let Err(err) = fs::create_dir(&$value) {
                return Err(Error::GenerateDir { path: $value, err });
            }
        }
    };

    ($name:ident, $value:expr) => {
        let $name: PathBuf = $value;
        generate_dir!($name);
    };
}





/***** ERRORS *****/
/// Defines errors that relate to the wizard.
#[derive(Debug)]
pub enum Error {
    /// Failed to query the user for the node config file.
    NodeConfigQuery {
        err: Box<Self>,
    },
    /// Failed to write the node config file.
    NodeConfigWrite {
        err: Box<Self>,
    },
    /// Failed to query the user for the proxy config file.
    ProxyConfigQuery {
        err: Box<Self>,
    },
    /// Failed to write the proxy config file.
    ProxyConfigWrite {
        err: Box<Self>,
    },
    /// Failed to write the proxy config file.
    ProxyConfigRead {
        err: brane_cfg::info::YamlError,
    },

    /// Failed to create a new file!().
    ConfigCreate {
        path: PathBuf,
        err:  std::io::Error,
    },
    /// Failed to generate a configuration file.
    ConfigSerialize {
        path: PathBuf,
        err:  brane_cfg::info::YamlError,
    },
    /// Failed to write to the config file.
    ConfigWrite {
        path: PathBuf,
        err:  std::io::Error,
    },
    /// Failed to generate a directory.
    GenerateDir {
        path: PathBuf,
        err:  std::io::Error,
    },
    /// Failed the query the user for input.
    ///
    /// The `what` should fill in: `Failed to query the user for ...`
    Input {
        what: &'static str,
        err:  brane_shr::input::Error,
    },
    InfraConfigWrite {
        err: Box<Error>,
    },
    PathCanonicalize {
        what: &'static str,
        path: PathBuf,
        err:  std::io::Error,
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            NodeConfigQuery { .. } => write!(f, "Failed to query node configuration"),
            NodeConfigWrite { .. } => write!(f, "Failed to write node config file"),
            ProxyConfigQuery { .. } => write!(f, "Failed to query proxy service configuration"),
            ProxyConfigWrite { .. } => write!(f, "Failed to write proxy service config file"),
            // TODO: Maybe we should elaborate on this error. I realise that it is easy to
            // misinterpret this error as writing the error.
            ProxyConfigRead { .. } => write!(f, "Failed to read proxy service config file"),
            InfraConfigWrite { .. } => write!(f, "Failed to write infra config file"),

            ConfigCreate { path, .. } => write!(f, "Failed to create config file '{}'", path.display()),
            ConfigSerialize { path, .. } => write!(f, "Failed to serialize config to '{}'", path.display()),
            ConfigWrite { path, .. } => write!(f, "Failed to write to config file '{}'", path.display()),
            GenerateDir { path, .. } => write!(f, "Failed to generate directory '{}'", path.display()),
            Input { what, .. } => write!(f, "Failed to query the user for {what}"),
            PathCanonicalize { what, path, .. } => write!(f, "Failed to canonicalize the {what} path: {}", path.display())
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn 'static + error::Error)> {
        use Error::*;
        match self {
            NodeConfigQuery { err } => Some(err),
            NodeConfigWrite { err } => Some(err),

            ProxyConfigQuery { err } => Some(err),
            ProxyConfigWrite { err } => Some(err),
            ProxyConfigRead { err } => Some(err),

            InfraConfigWrite { err } => Some(err),

            ConfigCreate { err, .. } => Some(err),
            ConfigSerialize { err, .. } => Some(err),
            ConfigWrite { err, .. } => Some(err),
            GenerateDir { err, .. } => Some(err),
            Input { err, .. } => Some(err),
            PathCanonicalize { err, .. } => Some(err),
        }
    }
}





/***** HELPER FUNCTIONS *****/
/// Writes a given [`Config`] to disk.
///
/// This wraps the default [`Config::to_path()`] function to also include a nice header.
///
/// # Arguments
/// - `config`: The [`Config`]-file to write.
/// - `path`: The path to write the file to.
/// - `url`: The wiki-URL to write in the file.
///
/// # Errors
/// This function may error if we failed to write any of this.
///
/// # Panics
/// This function may panic if the given path has no filename.
fn write_config<C>(config: C, path: impl AsRef<Path>, url: impl AsRef<str>) -> Result<(), Error>
where
    C: Info<Error = serde_yaml::Error>,
{
    let path: &Path = path.as_ref();
    let url: &str = url.as_ref();
    debug!("Generating config file '{}'...", path.display());

    // Deduce the filename
    let filename: Cow<str> = match path.file_name() {
        Some(filename) => filename.to_string_lossy(),
        None => {
            panic!("No filename found in '{}'", path.display());
        },
    };

    // Convert the filename to nice header
    let mut header_name: String = String::with_capacity(filename.len());
    let mut saw_lowercase: bool = false;
    let mut ext: bool = false;
    for c in filename.chars() {
        if !ext && c == '.' {
            // Move to extension mode
            header_name.push('.');
            ext = true;
        } else if !ext && (c == ' ' || c == '-' || c == '_') {
            // Write it as a space
            header_name.push(' ');
        } else if !ext && saw_lowercase && c.is_ascii_uppercase() {
            // Write is with a space, since we assume it's a word boundary in camelCase
            header_name.push(' ');
            header_name.push(c);
        } else if !ext && c.is_ascii_lowercase() {
            // Capitalize it
            header_name.push((c as u8 - b'a' + b'A') as char);
        } else {
            // The rest is pushed as-is
            header_name.push(c);
        }

        // Update whether we saw a lowercase last step
        saw_lowercase = c.is_ascii_lowercase();
    }

    // Create a file, now
    let mut handle: File = match File::create(path) {
        Ok(handle) => handle,
        Err(err) => {
            return Err(Error::ConfigCreate { path: path.into(), err });
        },
    };

    // Write the header to a string
    if let Err(err) = writeln!(handle, "# {header_name}") {
        return Err(Error::ConfigWrite { path: path.into(), err });
    };
    if let Err(err) = writeln!(handle, "#   by branectl") {
        return Err(Error::ConfigWrite { path: path.into(), err });
    };
    if let Err(err) = writeln!(handle, "# ") {
        return Err(Error::ConfigWrite { path: path.into(), err });
    };
    if let Err(err) = writeln!(handle, "# This file has been generated using the `branectl wizard` subcommand. You can") {
        return Err(Error::ConfigWrite { path: path.into(), err });
    };
    if let Err(err) = writeln!(handle, "# manually change this file after generation; it is just a normal YAML file.") {
        return Err(Error::ConfigWrite { path: path.into(), err });
    };
    if let Err(err) = writeln!(handle, "# Documentation for how to do so can be found here:") {
        return Err(Error::ConfigWrite { path: path.into(), err });
    };
    if let Err(err) = writeln!(handle, "# {url}") {
        return Err(Error::ConfigWrite { path: path.into(), err });
    };
    if let Err(err) = writeln!(handle, "# ") {
        return Err(Error::ConfigWrite { path: path.into(), err });
    };
    if let Err(err) = writeln!(handle) {
        return Err(Error::ConfigWrite { path: path.into(), err });
    };

    // Write the remainder of the file
    if let Err(err) = config.to_writer(handle, true) {
        return Err(Error::ConfigSerialize { path: path.into(), err });
    }
    Ok(())
}

/***** QUERY FUNCTIONS *****/
/// Queries the user for the proxy services configuration.
///
/// # Returns
/// A new [`ProxyConfig`] that reflects the user's choices.
///
/// # Errors
/// This function may error if we failed to query the user.
pub fn query_proxy_config() -> Result<ProxyConfig, Error> {
    // Query the user for the range
    let range: InclusiveRange<u16> = match input(
        "port range",
        "P1. Enter the range of ports allocated for outgoing connections",
        Some(InclusiveRange::new(4200, 4299)),
        Some(PortRangeValidator::default()),
        Some(hist!("prx-outgoing_range.hist")),
    ) {
        Ok(range) => range,
        Err(err) => {
            return Err(Error::Input { what: "outgoing range", err });
        },
    };
    debug!("Outgoing range: [{}, {}]", range.0.start(), range.0.end());
    println!();

    // Read the map of incoming ports
    let incoming: HashMap<u16, Address> = match input_map(
        "port",
        "address",
        "P2.1. Enter an incoming port map as '<incoming port>:<destination address>:<destination port>' (or leave empty to specify none)",
        "P2.%I. Enter an additional incoming port map as '<port>:<destination address>' (or leave empty to finish)",
        ":",
        // None::<NoValidator>,
        Some(PortMapValidator { allow_empty: true, ..Default::default() }),
        Some(hist!("prx-incoming.hist")),
    ) {
        Ok(incoming) => incoming,
        Err(err) => {
            return Err(Error::Input { what: "outgoing range", err });
        },
    };
    debug!("Incoming ports map:\n{:#?}", incoming);
    println!();

    // Finally, read any proxy
    let to_proxy_or_not_to_proxy: bool = match confirm("P3. Do you want to route outgoing traffic through a SOCKS proxy?", Some(false)) {
        Ok(yesno) => yesno,
        Err(err) => {
            return Err(Error::Input { what: "proxy confirmation", err });
        },
    };
    let forward: Option<ForwardConfig> = if to_proxy_or_not_to_proxy {
        // Query the address
        let address: Address = match input(
            "address",
            "P3a. Enter the target address (including port) to route the traffic to",
            None::<Address>,
            Some(AddressValidator::default()),
            Some(hist!("prx-forward-address.hist")),
        ) {
            Ok(address) => address,
            Err(err) => {
                return Err(Error::Input { what: "forwarding address", err });
            },
        };

        // Query the protocol
        let protocol: ProxyProtocol =
            match select("P3b. Enter the protocol to use to route traffic", vec![ProxyProtocol::Socks5, ProxyProtocol::Socks6], Some(0)) {
                Ok(prot) => prot,
                Err(err) => {
                    return Err(Error::Input { what: "forwarding protocol", err });
                },
            };

        // Construct the config
        Some(ForwardConfig { address, protocol })
    } else {
        None
    };
    debug!("Using forward config: {:?}", forward);
    println!();

    // Construct the ProxyConfig to return it
    Ok(ProxyConfig { outgoing_range: range.0, incoming, forward })
}

/// Queries the user for the node file configuration.
///
/// # Returns
/// A new [`NodeConfig`] that reflects the user's choices.
///
/// # Errors
/// This function may error if we failed to query the user.
pub fn query_proxy_node_config() -> Result<NodeConfig, Error> {
    // Construct the ProxyConfig to return it
    Ok(NodeConfig {
        hostnames: HashMap::new(),
        namespace: String::new(),
        node:      NodeSpecificConfig::Proxy(node::ProxyConfig {
            paths:    node::ProxyPaths { certs: "".into(), proxy: "".into() },
            services: node::ProxyServices {
                prx: node::PublicService {
                    name: "brane-prx".into(),
                    address: Address::Hostname("test.com".into(), 42),
                    bind: std::net::SocketAddr::V4(std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(0, 0, 0, 0), 0)),
                    external_address: Address::Hostname("test.com".into(), 42),
                },
            },
        }),
    })
}

#[derive(PartialEq)]
enum ProxyConfigSource {
    Default,
    ExistingFile,
    Prompt,
}

impl From<&ProxyConfigSource> for &'static str {
    fn from(value: &ProxyConfigSource) -> Self {
        match value {
            ProxyConfigSource::Default => "Use the default configuration (often recommended)",
            ProxyConfigSource::ExistingFile => "Use existing proxy.yml on your filesystem",
            ProxyConfigSource::Prompt => "Configure it right now.",
        }
    }
}

impl Display for ProxyConfigSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "{}", Into::<&str>::into(self)) }
}

/***** LIBRARY *****/
/// Main handler for the `branectl wizard setup` (or `branectl wizard node`) subcommand.
///
/// # Arguments
///
/// # Errors
/// This function may error if any of the wizard steps fail.
pub fn setup() -> Result<(), Error> {
    info!("Running wizard to setup a new node...");

    // Let us setup the history structure
    generate_dir!(_path, config_dir().unwrap().join("branectl"));

    // FIXME: I think XDG_CACHE_HOME is a better fit
    generate_dir!(_path, config_dir().unwrap().join("branectl").join("history"));

    // Do an intro prompt
    println!();
    println!(
        "{}{}{}",
        style("Welcome to ").bold(),
        style("Node Setup Wizard").bold().green(),
        style(format!(" for BRANE v{}", env!("CARGO_PKG_VERSION"))).bold()
    );
    println!();
    println!("This wizard will guide you through the process of setting up a node interactively.");
    println!("Simply answer the questions, and the required configuration files will be generated as you go.");
    println!();
    println!("You can abort the wizard at any time by pressing {}.", style("Ctrl+C").bold().green());
    println!();

    // Select the path where we will go to
    let mut prompt: Cow<str> = Cow::Borrowed("1. Select the location of the node configuration files");
    let path: PathBuf = loop {
        // Query the path
        let path: PathBuf = match input_path(prompt, Some("./"), Some(hist!("output_path.hist"))) {
            Ok(path) => path,
            Err(err) => {
                return Err(Error::Input { what: "config path", err });
            },
        };

        // Ask to create it if it does not exist
        if !path.exists() {
            // Do the question
            let ok: bool = match confirm("Directory '{}' does not exist. Create it?", Some(true)) {
                Ok(ok) => ok,
                Err(err) => {
                    return Err(Error::Input { what: "directory creation confirmation", err });
                },
            };

            // Create it, lest continue and try again
            if ok {
                generate_dir!(path);
            }
        }

        // Assert it's a directory
        if path.is_dir() {
            break path;
        }
        prompt = Cow::Owned(format!("Path '{}' does not point to a directory; specify another", path.display()));
    };
    debug!("Configuration directory: '{}'", path.display());
    println!();

    // Generate the configuration directories already
    generate_dir!(config_dir, path.join("config"));
    generate_dir!(certs_dir, config_dir.join("certs"));

    // Let us query the user for the type of node
    let kind: NodeKind = match select("2. Select the type of node to generate", [NodeKind::Central, NodeKind::Worker, NodeKind::Proxy], None) {
        Ok(kind) => kind,
        Err(err) => {
            return Err(Error::Input { what: "node kind", err });
        },
    };
    debug!("Building for node kind '{}'", kind.variant());
    println!();

    // Do a small intermittent text, which will be finished by node-specific contexts
    println!("You have selected to create a new {} node.", style(kind).bold().green());
    println!("For this node type, the following configuration files have to be generated:");

    // The rest is node-dependent
    match kind {
        NodeKind::Central => {
            println!(" - {}", style(config_dir.join("infra.yml").display()).bold());
            println!(" - {}", style(config_dir.join("proxy.yml").display()).bold());
            println!(" - {}", style(config_dir.join("node.yml").display()).bold());
            println!();

            println!("{}", style("=== infra.yml ===").bold());

            let location_id: String = input(
                "<Location ID>",
                "Insert a location ID for the central node",
                Some("central"),
                Some(LocationIdValidator::default()),
                Some(hist!("location_id")),
            )
            .map_err(|err| Error::Input { what: "location id", err })?;

            let central_node_dir = path.join(&location_id);
            generate_dir!(central_node_dir);

            // Read the map of incoming ports
            let _worker_mapping: HashMap<LocationId, Host> = input_map(
                "<Location ID>",
                "<Address>",
                "P2.1. Enter an worker mapping as: '<Location ID>:<Host>' (or leave empty to specify none)",
                "P2.%I. Enter an additional worker mapping as '<Location ID>:<Host>' (or leave empty to finish)",
                ":",
                // None::<NoValidator>,
                Some(LocationMapValidator { allow_empty: true, ..Default::default() }),
                Some(hist!("location-map.hist")),
            )
            .map_err(|err| Error::Input { what: "outgoing range", err })?;

            let infra_locations = _worker_mapping
                .into_iter()
                .map(|(location_id, host)| {
                    (location_id.clone(), InfraLocation {
                        // TODO: Prompt for the human readable name
                        name:     location_id,
                        delegate: (host.clone(), JOB_PORT).into(),
                        registry: (host, REG_PORT).into(),
})
                })
                .collect::<HashMap<_, _>>();

            let infra_file = brane_cfg::infra::InfraFile::new(infra_locations);

            println!("One can set the ports for all services on the worker in case these are different from the defaults.");
            println!(
                "This however is not yet supported in the generator. If you need this behaviour. It is recommended you use `branectl generate` \
                 instead."
            );
            let infra_path = central_node_dir.join("infra.yml");

            write_config(
                infra_file,
                &infra_path,
                "https://wiki.enablingpersonalizedinterventions.nl/user-guide/config/admins/infra.html",
            )
            .map_err(|err| Error::InfraConfigWrite { err: Box::new(err) })?;

            println!("{}", style("=== proxy.yml ===").bold());
            let proxy_config_source = select_enum::<ProxyConfigSource>(
                "How do you prefer to configure proxy.yml?",
                // TODO: Maybe use strum or something to get 'm all
                [ProxyConfigSource::Default, ProxyConfigSource::ExistingFile, ProxyConfigSource::Prompt],
                None,
            )
            .map_err(|err| Error::Input { what: "config source", err })?;

            let proxy = match proxy_config_source {
                ProxyConfigSource::Default => ProxyConfig::default(),
                ProxyConfigSource::ExistingFile => {
                    let path = input_path("Select the existing proxy.yml on your system", None::<PathBuf>, Some(hist!("proxy-path.hist")))
                        .map_err(|err| Error::Input { what: "proxy.yml path", err })?;
                    println!("Using proxy.yml from: `{}`", path.display());
                    println!("Note: that this will make a copy of this file. So changing it afterwards will have no effect.");
                    ProxyConfig::from_path(path).map_err(|err| Error::ProxyConfigRead { err })?
                },
                ProxyConfigSource::Prompt => query_proxy_config()?,
            };

            let proxy_path = central_node_dir.join("proxy.yml");
            write_config(proxy, &proxy_path, "https://wiki.enablingpersonalizedinterventions.nl/user-guide/config/admins/proxy.html")
                .map_err(|err| Error::ProxyConfigWrite { err: Box::new(err) })?;

            println!("{}", style("=== node.yml ===").bold());

            println!("The default settings for node.yml are listed below:");
            let node_defaults = confirm("Do you wish to use these defaults?", Some(true)).map_err(|err| Error::Input { what: "default central node", err })?;
            let node = if node_defaults {
                // FIXME: These need to become constants in the specification crate
                let prx_port = 123;
                let plr_port = 123;
                let api_port = 123;
                let drv_port = 123;

                let api_name = String::from("");
                let drv_name = String::from("");
                let plr_name = String::from("");
                let prx_name = String::from("");

                let certs_path = "";
                let packages_path = "";
                let external_proxy = None;

                // TODO: We need to figure this out, maybe prompt it or something
                let hosts = Default::default();
                let hostname = "";

                NodeConfig {
                    hostnames: hosts,
                    namespace: "brane-central".into(),

                    node: NodeSpecificConfig::Central(CentralConfig {
                        paths: CentralPaths {
                            certs:    canonicalize(&certs_path).map_err(|err| Error::PathCanonicalize { what: "cert path", path: certs_path.into(), err })?,
                            packages: canonicalize(&packages_path).map_err(|err| Error::PathCanonicalize { what: "packages path", path: packages_path.into(), err })?,

                            infra: canonicalize(&infra_path).map_err(|err| Error::PathCanonicalize { what: "infra configuration path", path: infra_path, err })?,
                            proxy: if external_proxy.is_some() { None } else { Some(canonicalize(&proxy_path).map_err(|err| Error::PathCanonicalize { what: "proxy configuration path", path: proxy_path, err })?) },
                        },

                        services: CentralServices {
                            api: PublicService {
                                name:    api_name.clone(),
                                bind:    SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), api_port).into(),
                                address: Address::Hostname(format!("http://{api_name}"), api_port),

                                external_address: Address::Hostname(format!("http://{hostname}"), api_port),
                            },
                            drv: PublicService {
                                name:    drv_name.clone(),
                                bind:    SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), drv_port).into(),
                                address: Address::Hostname(format!("grpc://{drv_name}"), drv_port),

                                external_address: Address::Hostname(format!("grpc://{hostname}"), drv_port),
                            },
                            plr: PrivateService {
                                name:    plr_name.clone(),
                                bind:    SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), plr_port).into(),
                                address: Address::Hostname(format!("http://{plr_name}"), plr_port),
                            },
                            prx: if let Some(address) = external_proxy {
                                PrivateOrExternalService::External(ExternalService { address })
                            } else {
                                PrivateOrExternalService::Private(PrivateService {
                                    name:    prx_name.clone(),
                                    bind:    SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), prx_port).into(),
                                    address: Address::Hostname(format!("http://{prx_name}"), prx_port),
                                })
                            },

                            aux_scylla: PrivateService {
                                name:    "aux-scylla".into(),
                                bind:    SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9042).into(),
                                address: Address::Hostname("aux-scylla".into(), 9042),
                            },
                        },
                    })
                }
            } else {
                // TODO: Prompt last parameters
                todo!("Overriding the defaults is not yet supported in the wizard");
            };

            let node_path = central_node_dir.join("node.yml");

            write_config(node, &node_path, "https://wiki.enablingpersonalizedinterventions.nl/user-guide/config/admins/node.html")
                .map_err(|err| Error::NodeConfigWrite { err: Box::new(err) })?;

            // q  -i, --infra <INFRA> The location of the 'infra.yml' file. Use '$CONFIG' to reference the value given by '--config-path'. [default: $CONFIG/infra.yml]
            // -P, --proxy <PROXY> The location of the 'proxy.yml' file. Use '$CONFIG' to reference the value given by '--config-path'. [default: $CONFIG/proxy.yml]
            //     --trace If given, prints the largest amount of debug information as possible.
            // -c, --certs <CERTS> The location of the certificate directory. Use '$CONFIG' to reference the value given by '--config-path'. [default: $CONFIG/certs]
            // -n, --node-config <NODE_CONFIG> The 'node.yml' file that describes properties about the node itself (i.e., the location identifier, where to find directories, which ports to use, ...) [default: ./node.yml]
            //     --packages <PACKAGES> The location of the package directory. [default: ./packages]
            //     --external-proxy <EXTERNAL_PROXY> If given, will use a proxy service running on the external address instead of one in this Docker service. This will mean that it will _not_ be spawned when running 'branectl start'. --api-name <API_NAME> The name of the API service's container. [default: brane-api]
            //     --drv-name <DRV_NAME> The name of the driver service's container. [default: brane-drv]
            //     --plr-name <PLR_NAME> The name of the planner service's container. [default: brane-plr]
            //     --prx-name <PRX_NAME> The name of the proxy service's container. [default: brane-prx]
            //     --api-port <API_PORT> The port on which the API service is available. [default: 50051]
            //     --plr-port <PLR_PORT> The port on which the planner service is available. [default: 50052]
            //     --drv-port <DRV_PORT> The port on which the driver service is available. [default: 50053]
            //     --prx-port <PRX_PORT> The port on which the proxy service is available. [default: 50050]uestions for the node.yml


            // TODO: Write node.yml
        },

        NodeKind::Worker => {
            println!(" - {}", style(config_dir.join("backend.yml").display()).bold());
            println!(" - {}", style(config_dir.join("proxy.yml").display()).bold());
            println!(" - {}", style(config_dir.join("node.yml").display()).bold());
            println!();

            println!("Besides configuration files, we will probably want some other files as well:");
            println!(" - {}", style(config_dir.join("policy_deliberation_secret.json").display()).bold());
            println!(" - {}", style(config_dir.join("policy_expert_secret.yml.json").display()).bold());
            println!(" - {}", style(config_dir.join("policy_token.json").display()).bold());
            println!();

            println!("And lastly:");
            println!(" - {}", style("A 802.1X certificate").bold());
            println!();

            println!("{}", style("=== backend.yml ===").bold());
            println!("{}", style("=== proxy.yml ===").bold());
            println!("{}", style("=== node.yml ===").bold());

            println!("{}", style("=== policy_deliberation_secret.json ===").bold());
            // TODO: Confirm
            println!("{}", style("=== policy_expert_secret.json ===").bold());
            // TODO: Confirm
            println!("{}", style("=== policy_token.json ===").bold());
            // TODO: Confirm
            // TODO: Ask name
            println!("{}", style("=== policies.db ===").bold());
            let create_policy_database = confirm("Do you wish to create a policy database (policies.db)", Some(true));
            println!("{}", style("=== 801.1X certificate ===").bold());
            let install = confirm("Do you wish to install this certificate on the central node", Some(true));

            // branectl generate backend -f -p ./config/backend.yml local
            // branectl generate proxy -f -p ./config/proxy.yml
            // branectl generate policy_secret -f -p ./config/policy_deliberation_secret.json
            // branectl generate policy_secret -f -p ./config/policy_expert_secret.json
            // branectl generate policy_db -f -p ./policies.db
            //
            // branectl generate policy_token "dan" "${WORKER_NAME}" 1y -s ./config/policy_expert_secret.json -p ./policy_token.json
            //
            // # Note that we are using amys own registry in --use-cases (Are we?)
            // branectl generate node -f worker "${CENTRAL_ADDRESS}" "${WORKER_NAME}" \
            //     --use-cases "central=http://${CENTRAL_HOSTNAME}:50051" \
            //     --reg-port 50151 --job-port 50152 --chk-port 50153 --prx-port 50150
            //
            // branectl generate certs -f -p ./config/certs server "${WORKER_NAME}" --hostname "${WORKER_ADDRESS}"
            // mkdir "../central/config/certs/${WORKER_NAME}"
            // cp ./config/certs/ca.pem "../central/config/certs/${WORKER_NAME}"
        },

        NodeKind::Proxy => {
            println!(" - {}", style(config_dir.join("proxy.yml").display()).bold());
            println!();

            // Note: we don't check if the user wants a custom config, since they very likely want it if they are setting up a proxy node
            // For the proxy, we only need to read the proxy config
            println!("{}", style("=== proxy.yml ===").bold());
            let cfg: ProxyConfig = match query_proxy_config() {
                Ok(cfg) => cfg,
                Err(err) => {
                    return Err(Error::ProxyConfigQuery { err: Box::new(err) });
                },
            };
            let proxy_path: PathBuf = config_dir.join("proxy.yml");
            if let Err(err) = write_config(cfg, proxy_path, "https://wiki.enablingpersonalizedinterventions.nl/user-guide/config/admins/proxy.html") {
                return Err(Error::ProxyConfigWrite { err: Box::new(err) });
            }

            // Now we generate the node.yml file
            println!("=== node.yml ===");
            let node: NodeConfig = match query_proxy_node_config() {
                Ok(node) => node,
                Err(err) => {
                    return Err(Error::NodeConfigQuery { err: Box::new(err) });
                },
            };
            let node_path: PathBuf = path.join("node.yml");
            if let Err(err) = write_config(node, node_path, "https://wiki.enablingpersonalizedinterventions.nl/user-guide/config/admins/node.html") {
                return Err(Error::NodeConfigWrite { err: Box::new(err) });
            }
        },
    }

    // Done!
    Ok(())
}
