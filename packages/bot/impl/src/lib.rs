mod types;
mod services;
mod state;
mod memory;
mod router;
mod lifecycle;
mod http_request;
mod storage;
mod updates;
mod guards;
mod utils;

use ic_http_certification::{HttpRequest, HttpResponse};
use bot_api::{
    lifecycle::init::*,
    updates::{update_monitor::*, notify_events::*}
};

pub const DEPLOY_MONITOR_CYCLES: u128 = 1_000_000_000_000;

ic_cdk::export_candid!();
