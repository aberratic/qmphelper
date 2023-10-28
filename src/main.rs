use clap::{Args, Parser, Subcommand};
use futures::StreamExt;
use subcommands::query_block_stats::QueryBlockStatsArguments;
use tokio::io;
use tracing::{debug, error, info, instrument, span, trace, warn, Level};
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
    /// Query the BlockStats for all virtual block devices
    QueryBlockStats(QueryBlockStatsArguments),
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
        Subcommands::QueryBlockStats(args) => {
            let query = qapi::qmp::query_blockstats {
                query_nodes: args.query_nodes,
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
