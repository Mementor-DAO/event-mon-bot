use clap::{Parser, Subcommand};
use monitor_api::updates::add_job::JobId;

#[derive(Parser, Debug)]
#[command(
    name = "",
    version, 
    about = "Event Monitor posts updates from any canister to your channel!", 
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Deploy a event monitor canister for this channel/group")]
    Deploy,
    #[command(about = "Print the status of this channel/group's event monitor canister")]
    Status,
    #[command(subcommand, about = "Job sub-commands")]
    Job (Job),
    #[command(subcommand, about = "EventMon Wallet sub-commands")]
    Wallet (Wallet),
}

#[derive(Subcommand, Debug)]
pub enum Job {
    #[command(about = "Create new job subcommands", subcommand)]
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

#[derive(Subcommand, Debug)]
pub enum Wallet {
    #[command(about = "Display your ICP balance in the EventMon Wallet")]
    Balance,
    #[command(about = "Display your ICP address in the EventMon Wallet")]
    Address,
    #[command(about = "Withdraw ICP from your account in the EventMon Wallet")]
    Withdraw {
        #[arg(help = "Amount to withdraw in decimal format (ie: 1.25)")]
        amount: f32,
        #[arg(help = "Optional destination account address in hex format (default: your OC wallet)")]
        to: Option<String>,
    },
    #[command(about = "Display logs of ICP transactions")]
    Logs {
        #[arg(default_value_t = 1, help = "Optional page number (default = 1)")]
        page: usize,
    },
}