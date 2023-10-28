use clap::Args;


#[derive(Args, Debug, Clone)]
pub struct QueryBlockStatsArguments {
    #[arg(default_value="false")]
    /// If true, the command will query all the block nodes that have a node name, 
    /// in a list which will include “parent” information, but not “backing”. If 
    /// false or omitted, the behavior is as before - query all the device backends, 
    /// recursively including their “parent” and “backing”. Filter nodes that were 
    /// created implicitly are skipped over in this mode.
    pub query_nodes: Option<bool>,
}
