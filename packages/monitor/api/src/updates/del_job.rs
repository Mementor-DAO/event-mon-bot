use candid::CandidType;
use serde::{Deserialize, Serialize};
use super::add_job::JobId;

#[derive(Serialize, Deserialize, CandidType)]
pub struct DelJobArgs {
    pub job_id: JobId, 
}

pub type DelJobResult = Result<(), String>;