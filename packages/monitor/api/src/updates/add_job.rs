use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

pub type JobId = u64;

#[derive(Serialize, Deserialize, CandidType)]
pub struct AddJobArgs {
    pub canister_id: Principal, 
    pub method_name: String, 
    pub interval: u32,
    pub batch_size: u32,
    pub output_template: String, 
    pub offset: u32,
}

pub type AddJobResult = Result<JobId, String>;