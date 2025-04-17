use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, CandidType)]
pub struct UpdateMonitorArgs {
    pub image: Vec<u8>,
}

#[derive(Serialize, CandidType)]
pub struct UpdateMonitorResponse {
}