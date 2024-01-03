//  MAIN.rs
//    by Lut99
//
//  Created:
//    30 Sep 2022, 11:59:58
//  Last edited:
//    03 Jan 2024, 14:06:47
//  Auto updated?
//    Yes
//
//  Description:
//!   Entrypoint to the `brane-drv` service.
//

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use brane_cfg::info::Info as _;
use brane_cfg::node::{CentralConfig, NodeConfig};
use brane_drv::handler::DriverHandler;
use brane_drv::planner::InstancePlanner;
use brane_prx::client::ProxyClient;
use clap::Parser;
use dotenvy::dotenv;
use error_trace::trace;
use log::{debug, error, info, warn, LevelFilter};
use specifications::driving::DriverServiceServer;
use tokio::signal::unix::{signal, Signal, SignalKind};
use tonic::transport::Server;


/***** ARGUMENTS *****/
/// Defines the arguments that may be given to the service.
#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    /// Print debug info
    #[clap(short, long, action, help = "If given, prints additional logging information.", env = "DEBUG")]
    debug:    bool,
    /// Consumer group id
    #[clap(short, long, default_value = "brane-drv", help = "The group ID of this service's consumer")]
    group_id: String,

    /// Node environment metadata store.
    #[clap(
        short,
        long,
        default_value = "/node.yml",
        help = "The path to the node environment configuration. This defines things such as where local services may be found or where to store \
                files, as wel as this service's service address.",
        env = "NODE_CONFIG_PATH"
    )]
    node_config_path: PathBuf,
}





/***** ENTRY POINT *****/
#[tokio::main]
async fn main() {
    dotenv().ok();
    let opts = Opts::parse();

    // Configure logger.
    let mut logger = env_logger::builder();
    logger.format_module_path(false);
    if opts.debug {
        logger.filter_level(LevelFilter::Debug).init();
    } else {
        logger.filter_level(LevelFilter::Info).init();
    }
    info!("Initializing brane-drv v{}...", env!("CARGO_PKG_VERSION"));

    // Load the config, making sure it's a central config
    debug!("Loading node.yml file '{}'...", opts.node_config_path.display());
    let node_config: NodeConfig = match NodeConfig::from_path(&opts.node_config_path) {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to load NodeConfig file: {}", err);
            std::process::exit(1);
        },
    };
    let central: CentralConfig = match node_config.node.try_into_central() {
        Some(central) => central,
        None => {
            error!("Given NodeConfig file '{}' does not have properties for a central node.", opts.node_config_path.display());
            std::process::exit(1);
        },
    };

    // Create our side of the planner, and launch its event monitor
    let planner: Arc<InstancePlanner> = match InstancePlanner::new(central.clone()) {
        Ok(planner) => Arc::new(planner),
        Err(err) => {
            error!("Failed to create InstancePlanner: {}", err);
            std::process::exit(1);
        },
    };
    if let Err(err) = planner.start_event_monitor(&opts.group_id).await {
        error!("Failed to start InstancePlanner event monitor: {}", err);
        std::process::exit(1);
    }

    // Start the DriverHandler
    let handler = DriverHandler::new(&opts.node_config_path, Arc::new(ProxyClient::new(central.services.prx.address())), planner.clone());

    // Start gRPC server with callback service.
    debug!("gRPC server ready to serve on '{}'", central.services.drv.bind);
    if let Err(err) = Server::builder()
        .add_service(DriverServiceServer::new(handler))
        .serve_with_shutdown(central.services.drv.bind, async {
            // Register a SIGTERM handler to be Docker-friendly
            let mut handler: Signal = match signal(SignalKind::terminate()) {
                Ok(handler) => handler,
                Err(err) => {
                    error!("{}", trace!(("Failed to register SIGTERM signal handler"), err));
                    warn!("Service will NOT shutdown gracefully on SIGTERM");
                    loop {
                        tokio::time::sleep(Duration::from_secs(24 * 3600)).await;
                    }
                },
            };

            // Wait until we receive such a signal after which we terminate the server
            handler.recv().await;
            info!("Received SIGTERM, shutting down gracefully...");
        })
        .await
    {
        error!("Failed to start gRPC server: {}", err);
        std::process::exit(1);
    }
}
