use candid::CandidType;
use serde::Deserialize;

#[derive(Deserialize, CandidType)]
pub struct UpdateMonitorArgs {
    pub wasm: Vec<u8>,
}

pub type UpdateMonitorResponse = Result<(), String>;
