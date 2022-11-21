//  PLANNER.rs
//    by Lut99
// 
//  Created:
//    25 Oct 2022, 11:35:00
//  Last edited:
//    21 Nov 2022, 17:10:51
//  Auto updated?
//    Yes
// 
//  Description:
//!   Implements a planner for the instance use-case.
// 


/***** LIBRARY *****/
use std::collections::HashMap;
use std::future::Future;
use std::mem;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, SystemTime};

use dashmap::DashMap;
use futures_util::TryStreamExt;
use log::{debug, info, error};
use prost::Message as _;
use rand::seq::IteratorRandom;
use rdkafka::{ClientConfig, Message};
use rdkafka::consumer::{CommitMode, Consumer};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

use brane_ast::Workflow;
use brane_ast::locations::Locations;
use brane_ast::ast::{DataName, Edge, SymTable};
use brane_cfg::{InfraFile, InfraPath};
use brane_cfg::node::NodeConfig;
use brane_shr::kafka::{ensure_topics, restore_committed_offsets};
use specifications::data::{AccessKind, AvailabilityKind, DataIndex, PreprocessKind};
use specifications::planning::{PlanningStatus, PlanningStatusKind, PlanningUpdate};

use crate::errors::PlanError;
use crate::spec::{Planner, TaskId};
use crate::api::get_data_index;


/***** CONSTANTS *****/
/// Defines the default timeout for the initial planning feedback (in ms)
const DEFAULT_PLANNING_STARTED_TIMEOUT: u128 = 30 * 1000;





/***** FUTURES *****/
/// Waits until the given plan has been planned.
struct WaitUntilPlanned {
    /// The correlation ID of the plan we're waiting for.
    correlation_id : String,
    /// The event-monitor updated list of states we use to check the plan's status.
    updates        : Arc<DashMap<String, PlanningStatus>>,
    /// The waker used by the event monitor.
    waker          : Arc<Mutex<Option<Waker>>>,

    /// Whether or not planning has started.
    started       : bool,
    /// The timeout to wait until the planning should have started.
    timeout       : u128,
    /// The time since the last check.
    timeout_start : SystemTime,
}

impl Future for WaitUntilPlanned {
    type Output = Option<PlanningStatus>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // If not started yet, possibly timeout
        if !self.started {
            // Compute the elapsed time
            let elapsed = match SystemTime::now().duration_since(self.timeout_start) {
                Ok(elapsed) => elapsed,
                Err(err)    => { panic!("The time since we started planning is later than the current time (by {:?}); this should never happen!", err.duration()); }
            };

            // Timeout if the time has passed
            if elapsed.as_millis() >= self.timeout {
                debug!("Workflow '{}' has timed out", self.correlation_id);
                return Poll::Ready(None);
            }
        }

        // Otherwise, if not timed out, switch on the state
        if let Some((_, status)) = self.updates.remove(&self.correlation_id) {
            match status {
                // The planning was started
                PlanningStatus::Started(name) => {
                    // Log it
                    debug!("Planning of workflow '{}' started{}",
                        self.correlation_id,
                        if let Some(name) = name {
                            format!(" by planner '{}'", name)
                        } else {
                            String::new()
                        }
                    );

                    // Set the internal result, then continue being pending
                    self.started = true;
                }

                // The planning was finished
                PlanningStatus::Success(_) |
                PlanningStatus::Failed(_)  |
                PlanningStatus::Error(_)   => {
                    debug!("Planning of workflow '{}' completed", self.correlation_id);
                    return Poll::Ready(Some(status));
                },

                // The rest means pending
                _ => {},
            }
        }

        // We have to update the internal waker too
        {
            let mut state: MutexGuard<Option<Waker>> = self.waker.lock().unwrap();
            *state = Some(cx.waker().clone());
        }
        Poll::Pending
    }
}





/***** HELPER FUNCTIONS *****/
/// Helper function that sends an update event over Kafka.
/// 
/// # Arguments
/// - `producer`: The Kafka producer to send with.
/// - `topic`: The Kafka topic to send on.
/// - `id`: The planning session ID to correlation this update with.
/// - `status`: The PlanningStatus to update with.
/// 
/// # Errors
/// This function errors if we failed to send the update somehow.
async fn send_update(producer: Arc<FutureProducer>, topic: impl AsRef<str>, correlation_id: impl AsRef<str>, status: PlanningStatus) -> Result<(), PlanError> {
    let topic          : &str = topic.as_ref();
    let correlation_id : &str = correlation_id.as_ref();
    debug!("Sending update '{:?}' on topic '{}' for workflow '{}'", status, topic, correlation_id);

    // Translate the status into a (kind, string) pair.
    let (kind, result): (PlanningStatusKind, Option<String>) = match status {
        PlanningStatus::Started(result) => (PlanningStatusKind::Started, result),

        PlanningStatus::Success(result) => (PlanningStatusKind::Success, Some(result)),
        PlanningStatus::Failed(result)  => (PlanningStatusKind::Failed, result),
        PlanningStatus::Error(result)   => (PlanningStatusKind::Error, Some(result)),

        PlanningStatus::None => { panic!("Cannot update the client on `PlanningStatus::None`"); },
    };
    let result_len: usize = result.as_ref().map(|r| r.len()).unwrap_or(0);

    // Create a planning update
    let update : PlanningUpdate = PlanningUpdate{
        id   : correlation_id.into(),
        kind : kind.into(),
        result,
    };

    // Encode it
    let mut payload : Vec<u8> = Vec::with_capacity(64 + result_len);
    if let Err(err) = update.encode(&mut payload) {
        return Err(PlanError::UpdateEncodeError{ correlation_id: correlation_id.into(), kind, err });
    };

    // Construct the future record that contains the to-be-planned workflow from this
    let scorr: String = correlation_id.into();
    let message: FutureRecord<String, [u8]> = FutureRecord::to(topic)
        .key(&scorr)
        .payload(&payload);

    // Send the message with the appropriate timeout
    let timeout: Timeout = Timeout::After(Duration::from_secs(5));
    if let Err((err, _)) = producer.send(message, timeout).await {
        return Err(PlanError::KafkaSendError{ correlation_id: correlation_id.into(), topic: topic.into(), err });
    }

    // Done
    Ok(())
}



/// Helper function that plans the given list of edges.
/// 
/// # Arguments
/// - `table`: The SymbolTable where this edge lives in.
/// - `edges`: The given list to plan.
/// - `dindex`: The DataIndex we use to resolve data references.
/// - `infra`: The infrastructure to resolve locations.
/// 
/// # Returns
/// Nothing, but does change the given list.
/// 
/// # Errors
/// This function may error if the given list of edges was malformed (usually due to unknown or inaccessible datasets or results).
fn plan_edges(table: &mut SymTable, edges: &mut [Edge], dindex: &DataIndex, infra: &InfraFile) -> Result<(), PlanError> {
    for (i, e) in edges.iter_mut().enumerate() {
        if let Edge::Node{ task, locs, at, input, result, .. } = e {
            debug!("Planning task '{}' (edge {})...", table.tasks[*task].name(), i);

            // If everything is allowed, we make it one easier for the planner by checking we happen to find only one occurrance based on the datasets
            if locs.is_all() {
                // Search all of the input to collect a list of possible locations
                let mut data_locs: Vec<&String> = vec![];
                for (d, _) in input.iter() {
                    // We only take data into account (for now, at least)
                    if let DataName::Data(name) = d {
                        // Attempt to find it
                        if let Some(info) = dindex.get(name) {
                            // Simply add all locations where it lives
                            data_locs.append(&mut info.access.keys().collect::<Vec<&String>>());
                        } else {
                            return Err(PlanError::UnknownDataset{ name: name.clone() });
                        }
                    }
                }

                // If there is only one location, then we override locs
                if data_locs.len() == 1 {
                    *locs = Locations::Restricted(vec![ data_locs[0].clone() ]);
                }
            }

            // We resolve all locations by collapsing them to the only possibility indicated by the user. More or less than zero? Error!
            if !locs.is_restrictive() || locs.restricted().len() != 1 { return Err(PlanError::AmbigiousLocationError{ name: table.tasks[*task].name().into(), locs: locs.clone() }); }
            let location: &str = &locs.restricted()[0];
            *at = Some(location.into());
            debug!("Task '{}' planned at '{}'", table.tasks[*task].name(), location);

            // For all dataset/intermediate result inputs, we check if these are available on the planned location.
            for (name, avail) in input {
                match name {
                    DataName::Data(name) => {
                        if let Some(info) = dindex.get(name) {
                            // Check if it is local or remote
                            if let Some(access) = info.access.get(location) {
                                debug!("Input dataset '{}' is locally available", name);
                                *avail = Some(AvailabilityKind::Available { how: access.clone() });
                            } else {
                                // Select one of the other locations it's available (for now, random?)
                                if info.access.is_empty() { return Err(PlanError::DatasetUnavailable { name: name.clone(), locs: vec![] }); }
                                let mut rng = rand::thread_rng();
                                let location: &str = info.access.keys().choose(&mut rng).unwrap();

                                // Get the registry of that location
                                let registry : &String = &infra.get(location).unwrap_or_else(|| panic!("DataIndex advertises location '{}', but that location is unknown", location)).registry;
                                let address  : String  = format!("{}/data/download/{}", registry, name);
                                debug!("Input dataset '{}' will be transferred in from '{}'", name, address);

                                // That's the location where to pull the dataset from
                                *avail = Some(AvailabilityKind::Unavailable{ how: PreprocessKind::TransferRegistryTar{ location: location.into(), address } });
                            }
                        } else {
                            return Err(PlanError::UnknownDataset{ name: name.clone() });
                        }
                    },

                    DataName::IntermediateResult(name) => {
                        // It has to be declared before
                        if let Some(loc) = table.results.get(name) {
                            // Match on whether it is available locally or not
                            if location == loc {
                                debug!("Input intermediate result '{}' is locally available", name);
                                *avail = Some(AvailabilityKind::Available { how: AccessKind::File{ path: PathBuf::from(name) } });
                            } else {
                                // Find the remote location in the infra file
                                let registry: &String = &infra.get(loc).unwrap_or_else(|| panic!("IntermediateResult advertises location '{}', but that location is unknown", loc)).registry;

                                // Compute the registry access method
                                let address: String = format!("{}/results/download/{}", registry, name);
                                debug!("Input intermediate result '{}' will be transferred in from '{}'", name, address);

                                // That's the location where to pull the dataset from
                                *avail = Some(AvailabilityKind::Unavailable{ how: PreprocessKind::TransferRegistryTar{ location: loc.clone(), address } });
                            }
                        } else {
                            return Err(PlanError::UnknownIntermediateResult{ name: name.clone() });
                        }
                    },
                }
            }

            // Then, we make the intermediate result available at the location where the function is being run (if there is any)
            if let Some(name) = result {
                // Insert an entry in the list detailling where to access it and how
                debug!("Making intermediate result '{}' accessible after execution of '{}' on '{}'", name, table.tasks[*task].name(), location);
                table.results.insert(name.clone(), location.into());
            }
        }
    }

    // Done
    debug!("Planning success");
    Ok(())
}



/// Blocks the current thread until the remote planner has planned the workflow with the given correlation ID.
/// 
/// This is done by using Kafka events to wait for it.
/// 
/// # Arguments
/// - `correlation_id`: The identifier for the workflow of which we are interested in the planning results.
/// - `waker`: The Waker of the event monitor that will ping us when a new message has arrived.
/// - `updates`: The event monitor-updated list of the latest status per correlation ID.
/// 
/// # Returns
/// The planned workflow as a Workflow. It being planned means its tasks and datasets are resolved.
/// 
/// # Errors
/// This function errors if we either failed to wait on Kafka, or if the remote planner failed.
async fn wait_planned(correlation_id: impl AsRef<str>, waker: Arc<Mutex<Option<Waker>>>, updates: Arc<DashMap<String, PlanningStatus>>) -> Result<Workflow, PlanError> {
    let correlation_id: &str = correlation_id.as_ref();

    // Wait until a plan result occurs
    let res: Option<PlanningStatus> = WaitUntilPlanned {
        correlation_id : correlation_id.to_string(),
        updates,
        waker,

        started       : false,
        timeout       : DEFAULT_PLANNING_STARTED_TIMEOUT,
        timeout_start : SystemTime::now(),
    }.await;

    // Match on timeouts
    let res: PlanningStatus = match res {
        Some(res) => res,
        None      => { return Err(PlanError::PlanningTimeout{ correlation_id: correlation_id.into(), timeout: DEFAULT_PLANNING_STARTED_TIMEOUT }); },
    };

    // Match the result itself
    match res {
        // The planning was done
        PlanningStatus::Success(wf) => {
            // We attempt to parse the result itself as a Workflow
            let workflow: Workflow = match serde_json::from_str(&wf) {
                Ok(workflow) => workflow,
                Err(err)     => { return Err(PlanError::PlanParseError{ correlation_id: correlation_id.into(), raw: wf, err }); }  
            };

            // Done, return
            Ok(workflow)
        },

        // Otherwise, no plan available
        PlanningStatus::Failed(reason) => Err(PlanError::PlanningFailed{ correlation_id: correlation_id.into(), reason }),
        PlanningStatus::Error(err)     => Err(PlanError::PlanningError{ correlation_id: correlation_id.into(), err }),

        // Other things should not occur; the wait covered those
        _ => { unreachable!(); }
    }
}





/***** LIBRARY *****/
/// The planner is in charge of assigning locations to tasks in a workflow. This one is very simple, assigning 'localhost' to whatever it sees.
pub struct InstancePlanner {
    /// The Kafka servers we're connecting to.
    node_config : NodeConfig,

    /// The Kafka producer with which we send commands and such.
    producer  : Arc<FutureProducer>,
    /// The waker triggered by the event monitor to trigger futures waiting for event updates.
    waker     : Arc<Mutex<Option<Waker>>>,
    /// The list of states that contains the most recently received planner updates.
    updates   : Arc<DashMap<String, PlanningStatus>>,
}

impl InstancePlanner {
    /// Constructor for the InstancePlanner.
    /// 
    /// # Arguments
    /// - `node_config`: The configuration for this node's environment. For us, mostly Kafka topics and associated broker.
    /// 
    /// # Returns
    /// A new InstancePlanner instance.
    #[inline]
    pub fn new(node_config: NodeConfig) -> Result<Self, PlanError> {
        let brokers: String = node_config.node.central().services.brokers.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(",");
        Ok(Self {
            node_config,

            producer  : match ClientConfig::new().set("bootstrap.servers", &brokers).set("message.timeout.ms", "5000").create() {
                Ok(producer) => Arc::new(producer),
                Err(err)     => { return Err(PlanError::KafkaProducerError { err }); },
            },
            waker     : Arc::new(Mutex::new(None)),
            updates   : Arc::new(DashMap::new()),
        })
    }



    /// This function hosts the actual planner, which uses an event monitor to receive plans which are then planned.
    /// 
    /// # Arguments
    /// - `node_config_path`: Path to the node.yml file that defines this node's environment configuration.
    /// - `node_config`: The configuration for this node's environment. For us, mostly Kafka topics and paths to infra.yml and (optional) secrets.yml files. This is mostly given to avoid another load, since we could've loaded it from the path too.
    /// - `group_id`: The Kafka group ID to listen on.
    /// 
    /// # Returns
    /// This function doesn't really return, unless the Kafka topic stream closes.
    /// 
    /// # Errors
    /// This function only errors if we fail to listen for events. Otherwise, errors are logged to stderr using the `error!` macro.
    pub async fn planner_server(node_config_path: impl Into<PathBuf>, node_config: NodeConfig, group_id: impl Into<String>) -> Result<(), PlanError> {
        let node_config_path : PathBuf = node_config_path.into();
        let group_id         : String  = group_id.into();

        // Ensure that the input/output topics exists.
        let topics  : Vec<&str> = vec![ &node_config.node.central().topics.planner_command, &node_config.node.central().topics.planner_results ];
        let brokers : String    = node_config.node.central().services.brokers.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(",");
        if let Err(err) = ensure_topics(topics.clone(), &brokers).await {
            return Err(PlanError::KafkaTopicError{ brokers, topics: topics.into_iter().map(|t| t.into()).collect(), err });
        };

        // Start the producer(s) and consumer(s).
        let producer: Arc<FutureProducer> = match ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("message.timeout.ms", "5000")
            .create()
        {
            Ok(producer) => Arc::new(producer),
            Err(err)     => { return Err(PlanError::KafkaProducerError{ err }); },
        };
        let consumer: StreamConsumer = match ClientConfig::new()
            .set("group.id", &group_id)
            .set("bootstrap.servers", &brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .create()
        {
            Ok(consumer) => consumer,
            Err(err)     => { return Err(PlanError::KafkaConsumerError{ err }); },
        };

        // Now restore the committed offsets
        if let Err(err) = restore_committed_offsets(&consumer, &node_config.node.central().topics.planner_command) {
            return Err(PlanError::KafkaOffsetsError{ err });
        }

        // Next, we start processing the incoming stream of messages as soon as they arrive
        match consumer.stream().try_for_each(|borrowed_message| {
            consumer.commit_message(&borrowed_message, CommitMode::Sync).unwrap();

            // Shadow with owned clones
            let owned_message     : OwnedMessage        = borrowed_message.detach();
            let producer          : Arc<FutureProducer> = producer.clone();
            let node_config_path  : PathBuf             = node_config_path.clone();

            // Do the rest in a future that takes ownership of the clones
            async move {
                // Fetch the most recent NodeConfig
                let node_config: NodeConfig = match NodeConfig::from_path(node_config_path) {
                    Ok(config) => config,
                    Err(err)   => {
                        error!("Failed to load NodeConfig file: {}", err);
                        return Ok(());
                    },
                };

                // Parse the key
                let id: String = String::from_utf8_lossy(owned_message.key().unwrap_or(&[])).to_string();
                info!("Received new plan request with ID '{}' on topic '{}'", id, node_config.node.central().topics.planner_command);

                // Parse the payload, if any
                if let Some(payload) = owned_message.payload() {
                    // Parse as UTF-8
                    debug!("Message: \"\"\"{}\"\"\"", String::from_utf8_lossy(payload));
                    let message: String = String::from_utf8_lossy(payload).to_string();

                    // Attempt to parse the workflow
                    debug!("Parsing workflow of {} characters for session '{}'", message.len(), id);
                    let mut workflow: Workflow = match serde_json::from_str(&message) {
                        Ok(workflow) => workflow,
                        Err(err)     => {
                            error!("Failed to parse incoming message workflow on topic '{}' as Workflow JSON: {}\n\nworkflow:\n{}\n{}\n{}\n", node_config.node.central().topics.planner_command, err, (0..80).map(|_| '-').collect::<String>(), message, (0..80).map(|_| '-').collect::<String>());
                            return Ok(());
                        }
                    };

                    // Send that we've started planning
                    if let Err(err) = send_update(producer.clone(), &node_config.node.central().topics.planner_results, &id, PlanningStatus::Started(None)).await { error!("Failed to update client that planning has started: {}", err); };

                    // Fetch the data index
                    let data_index_addr: String = format!("{}/data/info", node_config.node.central().services.api);
                    let dindex: DataIndex = match get_data_index(&data_index_addr).await {
                        Ok(dindex) => dindex,
                        Err(err)   => {
                            error!("Failed to fetch DataIndex from '{}': {}", data_index_addr, err);
                            return Ok(());
                        }
                    };

                    // Now we do the planning
                    {
                        // Load the infrastructure file
                        let infra: InfraFile = match InfraFile::from_path(InfraPath::new(&node_config.node.central().paths.infra, &node_config.node.central().paths.secrets)) {
                            Ok(infra) => infra,
                            Err(err)  => {
                                error!("Failed to load infrastructure file '{}': {}", node_config.node.central().paths.infra.display(), err);
                                return Ok(());
                            }
                        };

                        // Get the symbol table muteable, so we can... mutate... it
                        let mut table: Arc<SymTable> = Arc::new(SymTable::new());
                        mem::swap(&mut workflow.table, &mut table);
                        let mut table: SymTable      = Arc::try_unwrap(table).unwrap();

                        // Do the main edges first
                        {
                            // Start by getting a list of all the edges
                            let mut edges: Arc<Vec<Edge>> = Arc::new(vec![]);
                            mem::swap(&mut workflow.graph, &mut edges);
                            let mut edges: Vec<Edge>      = Arc::try_unwrap(edges).unwrap();

                            // Plan them
                            debug!("Planning main edges...");
                            if let Err(err) = plan_edges(&mut table, &mut edges, &dindex, &infra) {
                                error!("Failed to plan main edges for workflow with correlation ID '{}': {}", id, err);
                                if let Err(err) = send_update(producer.clone(), &node_config.node.central().topics.planner_results, &id, PlanningStatus::Error(format!("{}", err))).await { error!("Failed to update client that planning has failed: {}", err); }
                                return Ok(());
                            };

                            // Move the edges back
                            let mut edges: Arc<Vec<Edge>> = Arc::new(edges);
                            mem::swap(&mut edges, &mut workflow.graph);
                        }

                        // Then we do the function edges
                        {
                            // Start by getting the map
                            let mut funcs: Arc<HashMap<usize, Vec<Edge>>> = Arc::new(HashMap::new());
                            mem::swap(&mut workflow.funcs, &mut funcs);
                            let mut funcs: HashMap<usize, Vec<Edge>>      = Arc::try_unwrap(funcs).unwrap();

                            // Iterate through all of the edges
                            for (idx, edges) in &mut funcs {
                                debug!("Planning '{}' edges...", table.funcs[*idx].name);
                                if let Err(err) = plan_edges(&mut table, edges, &dindex, &infra) {
                                    error!("Failed to plan function '{}' edges for workflow with correlation ID '{}': {}", table.funcs[*idx].name, id, err);
                                    if let Err(err) = send_update(producer.clone(), &node_config.node.central().topics.planner_results, &id, PlanningStatus::Error(format!("{}", err))).await { error!("Failed to update client that planning has failed: {}", err); }
                                    return Ok(());
                                }
                            }

                            // Put the map back
                            let mut funcs: Arc<HashMap<usize, Vec<Edge>>> = Arc::new(funcs);
                            mem::swap(&mut funcs, &mut workflow.funcs);
                        }

                        // Then, put the table back
                        let mut table: Arc<SymTable> = Arc::new(table);
                        mem::swap(&mut table, &mut workflow.table);
                    }

                    // With the planning done, re-serialize
                    debug!("Serializing plan...");
                    let splan: String = match serde_json::to_string(&workflow) {
                        Ok(splan) => splan,
                        Err(err)  => {
                            error!("Failed to serialize plan: {}", err);
                            if let Err(err) = send_update(producer.clone(), &node_config.node.central().topics.planner_results, &id, PlanningStatus::Error(format!("{}", err))).await { error!("Failed to update client that planning has failed: {}", err); }
                            return Ok(());
                        },
                    };

                    // Send the result
                    if let Err(err) = send_update(producer.clone(), &node_config.node.central().topics.planner_results, &id, PlanningStatus::Success(splan)).await { error!("Failed to update client that planning has succeeded: {}", err); }
                    debug!("Planning OK");
                }

                // Done
                Ok(())
            }
        }).await {
            Ok(_)    => Ok(()),
            Err(err) => Err(PlanError::KafkaStreamError{ err }),
        }
    }



    /// Launches an event monitor in the background (using `tokio::spawn`) for planning updates.
    /// 
    /// Note that the event monitor itself is launched asynchronously. When this function returns, it has merely started (even though it's an async function itself - but that's only used during setup, I swear).
    /// 
    /// # Arguments
    /// - `group_id`: The Kafka group ID to listen on.
    /// 
    /// # Errors
    /// This function errors if we failed to start listening on the Kafka stream and create a future for that.
    pub async fn start_event_monitor(&self, group_id: impl AsRef<str>) -> Result<(), PlanError> {
        let group_id  : &str = group_id.as_ref();

        // Ensure that the to-be-listened on topic exists
        let brokers: String = self.node_config.node.central().services.brokers.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(",");
        if let Err(err) = ensure_topics(vec![ &self.node_config.node.central().topics.planner_results ], &brokers).await { return Err(PlanError::KafkaTopicError { brokers, topics: vec![ self.node_config.node.central().topics.planner_results.clone() ], err }); };

        // Create one consumer per topic that we're reading (i.e., one)
        let consumer: StreamConsumer = match ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", &brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .create()
        {
            Ok(consumer) => consumer,
            Err(err)     => { return Err(PlanError::KafkaConsumerError{ err }); }
        };

        // Restore previous offsets
        if let Err(err) = restore_committed_offsets(&consumer, &self.node_config.node.central().topics.planner_results) { return Err(PlanError::KafkaOffsetsError { err }); }

        // Run the Kafka consumer to monitor the planning events on a new thread (or at least, concurrently)
        let waker   : Arc<Mutex<Option<Waker>>>            = self.waker.clone();
        let updates : Arc<DashMap<String, PlanningStatus>> = self.updates.clone();
        tokio::spawn(async move {
            loop {
                if let Err(err) = consumer.stream().try_for_each(|borrowed_message| {
                    let owned_message = borrowed_message.detach();
                    let owned_updates = updates.clone();
                    let owned_waker   = waker.clone();

                    // The rest is returned as a future
                    async move {
                        if let Some(payload) = owned_message.payload() {
                            // Decode payload into a PlanningUpdate message.
                            let msg: PlanningUpdate = match PlanningUpdate::decode(payload) {
                                Ok(msg)  => msg,
                                Err(err) => { error!("Failed to decode incoming PlanningUpdate: {}", err); return Ok(()); },
                            };
        
                            // Match on the kind, inserting the proper states
                            match PlanningStatusKind::from_i32(msg.kind) {
                                Some(PlanningStatusKind::Started) => {
                                    debug!("Status update: Workflow '{}' is now being planned{}", msg.id, if let Some(name) = &msg.result { format!(" by planner '{}'", name) } else { String::new() });
                                    owned_updates.insert(msg.id, PlanningStatus::Started(msg.result));
                                },
        
                                Some(PlanningStatusKind::Success) => {
                                    debug!("Status update: Workflow '{}' has successfully been planned", msg.id);
        
                                    // Make sure a workflow is given
                                    let plan: String = match msg.result {
                                        Some(plan) => plan,
                                        None       => { error!("Incoming PlanningUpdate with PlanningStatusKind::Success is missing a resolved workflow"); return Ok(()); },
                                    };
        
                                    // Store it
                                    owned_updates.insert(msg.id, PlanningStatus::Success(plan));
                                },
                                Some(PlanningStatusKind::Failed) => {
                                    debug!("Status update: Workflow '{}' failed to been planned{}", msg.id, if let Some(reason) = &msg.result { format!(": {}", reason) } else { String::new() });
                                    owned_updates.insert(msg.id, PlanningStatus::Failed(msg.result));
                                },
                                Some(PlanningStatusKind::Error) => {
                                    let err: String = msg.result.unwrap_or_else(|| String::from("<unknown error>"));
                                    debug!("Status update: Workflow '{}' has caused errors to appear: {}", msg.id, err);
                                    owned_updates.insert(msg.id, PlanningStatus::Error(err));
                                },
        
                                None => { error!("Unknown PlanningStatusKind '{}'", msg.kind); return Ok(()); },
                            }
                        }
        
                        // Signal the waker, if any
                        {
                            let mut state: MutexGuard<Option<Waker>> = owned_waker.lock().unwrap();
                            if let Some(waker) = state.take() {
                                waker.wake();
                            }
                        }
                        
                        // Done
                        Ok(())
                    }
                }).await {
                    error!("Failed to run InstancePlanner event monitor: {}", err);
                    error!("Note: you will likely not get any events now. Automatically restarting in 3 seconds, but you might want to investigate the problem.");
                    tokio::time::sleep(Duration::from_millis(3000)).await;
                }
            }
        });

        // Done
        Ok(())
    }
}

#[async_trait::async_trait]
impl Planner for InstancePlanner {
    async fn plan(&self, workflow: Workflow) -> Result<Workflow, PlanError> {
        // Ensure that the to-be-send-on topic exists
        let brokers: String = self.node_config.node.central().services.brokers.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(",");
        if let Err(err) = ensure_topics(vec![ &self.node_config.node.central().topics.planner_command ], &brokers).await { return Err(PlanError::KafkaTopicError { brokers, topics: vec![ self.node_config.node.central().topics.planner_command.clone() ], err }); };

        // Serialize the workflow
        let swork: String = match serde_json::to_string(&workflow) {
            Ok(swork) => swork,
            Err(err)  => { return Err(PlanError::WorkflowSerializeError{ err }); },  
        };

        // Populate a "PlanningCommand" with that (i.e., just populate a future record with the string)
        let correlation_id: String = format!("{}", TaskId::generate());
        let message: FutureRecord<String, [u8]> = FutureRecord::to(&self.node_config.node.central().topics.planner_command)
            .key(&correlation_id)
            .payload(swork.as_bytes());

        // Send the message
        if let Err((err, _)) = self.producer.send(message, Timeout::After(Duration::from_secs(5))).await {
            return Err(PlanError::KafkaSendError { correlation_id, topic: self.node_config.node.central().topics.planner_command.clone(), err });
        }

        // Now we wait until the message has been planned.
        let plan: Workflow = wait_planned(&correlation_id, self.waker.clone(), self.updates.clone()).await?;

        // Done
        Ok(plan)
    }
}
