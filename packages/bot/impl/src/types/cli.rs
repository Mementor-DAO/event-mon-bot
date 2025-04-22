use clap::{Parser, Subcommand};
use monitor_api::updates::add_job::JobId;

#[derive(Parser, Debug)]
#[command(
    name = "",
    version, 
    about = "Events Monitor Bot posts real-time updates from any canister to your channel!", 
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Deploy a monitor canister for this channel/group")]
    Deploy,
    #[command(about = "Create a new job subcommands", subcommand)]
    Create(CreateSubcommand),
    #[command(about = "List jobs")]
    List {
        #[arg(default_value_t = 1, help = "Optional page number (default = 1)")]
        page: u32,
    },
    #[command(about = "Start a job")]
    Start {
        #[arg(help = "Job id")]
        id: JobId
    },
    #[command(about = "Stop a job")]
    Stop {
        #[arg(help = "Job id")]
        id: JobId
    },
    #[command(about = "Delete a job")]
    Delete {
        #[arg(help = "Job id")]
        id: JobId
    },
}

#[derive(Subcommand, Debug)]
pub enum CreateSubcommand {
    #[command(about = "Create a new job to monitor a canister method")]
    Canister {
        #[arg(help = "Canister id")]
        canister_id: String,
        #[arg(help = "Method name")]
        method_name: String,
        #[arg(help = "Output template")]
        output_template: String,
        #[arg(help = "Interval, in seconds, to poll the canister")]
        interval: u32,
    },
}