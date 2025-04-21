use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType)]
pub struct AddJobArgs {
    pub canister_id: Principal, 
    pub method_name: String, 
    pub output_template: String, 
    pub interval: u32
}