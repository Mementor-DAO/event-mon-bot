mod types;
mod state;
mod utils;
mod guards;
mod lifecycle;
mod updates;
mod memory;

use monitor_api::lifecycle::init::InitOrUpgradeArgs;

ic_cdk::export_candid!();