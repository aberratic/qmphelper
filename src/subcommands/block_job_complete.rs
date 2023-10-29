use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct BlockJobCompleteArguments {
    #[arg(long)]
    pub device: String,
}
