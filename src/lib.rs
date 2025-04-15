mod types;
mod services;
mod state;
mod memory;
mod router;
mod lifecycle;
mod http_request;
mod storage;
mod utils;

use ic_http_certification::{HttpRequest, HttpResponse};
use crate::types::init::InitOrUpgradeArgs;

ic_cdk::export_candid!();
