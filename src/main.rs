use subcommands::query_block_stats::QueryBlockStatsArguments;
use tokio::io;
use clap::{Parser, Subcommand, Args};
use futures::StreamExt;

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

#[tokio::main]
pub async fn main() -> io::Result<()> {
    let args = Arguments::parse();
    let stream = 
        qapi::futures::QmpStreamTokio::open_uds(args.unixsocket).await?;
    let stream = stream.negotiate().await?;
    let (qmp, mut events) = stream.into_parts();

    match args.command {
        Subcommands::QueryBlockStats(args) => {
            let query = qapi::qmp::query_blockstats {
                query_nodes: args.query_nodes,
                
            };
            let response = qmp.execute(query).await?;
            println!("{:#?}", response);
        }
    }
    Ok(())
}
