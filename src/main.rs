use clap::{Parser, Subcommand};
use subcommands::{
    block_commit::BlockCommitArguments, block_job_complete::BlockJobCompleteArguments,
    query_block_stats::QueryBlockStatsArguments,
};
use tokio::io;
use tracing::{debug, error};
use tracing_subscriber;
mod subcommands;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Arguments {
    #[command(subcommand)]
    command: Subcommands,
    unixsocket: String,
}

#[derive(Subcommand, Debug, Clone)]
enum Subcommands {
    /// Live commit of data from overlay image nodes into backing nodes - i.e., writes
    /// data between ‘top’ and ‘base’ into ‘base’.
    /// If top == base, that is an error. If top has no overlays on top of it, or if it
    /// is in use by a writer, the job will not be completed by itself. The user needs to
    /// complete the job with the block-job-complete command after getting the ready event.
    /// If the base image is smaller than top, then the base image will be resized to
    /// be the same size as top. If top is smaller than  the base image, the base will
    /// not be truncated. If you want the base image size to match the size of the smaller
    /// top, you can safely truncate it yourself once the commit operation successfully
    /// dscompletes.
    BlockCommit(BlockCommitArguments),
    /// Query the BlockStats for all virtual block devices
    QueryBlockStats(QueryBlockStatsArguments),
    QueryBlockJobs,
    BlockJobComplete(BlockJobCompleteArguments),
}

macro_rules! handle_errors {
    ($result:expr, $errormessage:expr) => {
        match $result {
            Ok(a) => a,
            Err(error) => {
                error!($errormessage, error);
                std::process::exit(1);
            }
        }
    };
    ($result:expr, $errormessage:expr, $($additionals:tt)*) => {
        match $result {
            Ok(a) => a,
            Err(error) => {
                error!($errormessage, error, ($($additionals)*));
                std::process::exit(1);
            }
        }
    };
}

#[tokio::main]
pub async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Arguments::parse();

    debug!("Connecting to {}", args.unixsocket.clone());
    let stream = handle_errors!(
        qapi::futures::QmpStreamTokio::open_uds(args.unixsocket.clone()).await,
        "Failed to connect to {}: {}",
        args.unixsocket.clone()
    );
    debug!("Stream capabilities: {:#?}", stream.capabilities);

    debug!("Negotiating QMP protocol");
    let stream = handle_errors!(
        stream.negotiate().await,
        "Failed to negotiate QMP protocol: {}"
    );
    debug!("Finished negotiating QMP protocol");

    let (qmp, handle) = stream.spawn_tokio();
    debug!("Spawned QMP stream");

    match args.command {
        Subcommands::BlockCommit(args) => {
            let qmp_onerror: Option<qapi::qmp::BlockdevOnError> = match args.on_error {
                Some(on_error) => Some(on_error.into()),
                None => None,
            };
            let query = qapi::qmp::block_commit {
                device: args.device,
                job_id: args.job_id,
                base_node: args.base_node,
                base: args.base,
                top_node: args.top_node,
                top: args.top,
                backing_file: args.backing_file,
                speed: args.speed,
                on_error: qmp_onerror,
                filter_node_name: args.filter_node_name,
                auto_finalize: args.auto_finalize,
                auto_dismiss: args.auto_dismiss,
            };
            debug!("Executing query: {:#?}", query);
            let response = qmp.execute(query).await?;
            println!("{:#?}", response);
        }
        Subcommands::QueryBlockStats(args) => {
            let query = qapi::qmp::query_blockstats {
                query_nodes: args.query_nodes,
            };
            debug!("Executing query: {:#?}", query);
            let response = qmp.execute(query).await?;
            println!("{:#?}", response);
        }
        Subcommands::QueryBlockJobs => {
            let query = qapi::qmp::query_block_jobs {};
            debug!("Executing query: {:#?}", query);
            let response = qmp.execute(query).await?;
            println!("{:#?}", response);
        }
        Subcommands::BlockJobComplete(args) => {
            let query = qapi::qmp::block_job_complete {
                device: args.device,
            };
            debug!("Executing query: {:#?}", query);
            let response = qmp.execute(query).await?;
            println!("{:#?}", response);
        }
    }
    {
        drop(qmp);
        handle.await?;
    }
    Ok(())
}
