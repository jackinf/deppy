use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    ToDeploy(CommandToDeployArgs),
    Doctor(CommandDoctorArgs),
}

#[derive(Args)]
pub struct CommandToDeployArgs {
    #[arg(short, long)]
    pub owner: String,

    #[arg(short, long)]
    pub project: String,

    #[arg(short, long)]
    pub author: Option<String>,

    #[arg(short, long)]
    pub env: String,

    #[arg(short, long)]
    pub cluster: Option<String>,

    #[arg(short, long)]
    pub deploy_version: Option<String>,
}

#[derive(Args)]
pub struct CommandDoctorArgs {
    #[arg(short, long)]
    pub project: String,

    #[arg(short, long)]
    pub author: String,
}
