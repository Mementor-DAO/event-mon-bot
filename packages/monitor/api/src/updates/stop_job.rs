use candid::CandidType;
use serde::{Deserialize, Serialize};
use super::add_job::JobId;

#[derive(Serialize, Deserialize, CandidType)]
pub struct StopJobArgs {
    pub job_id: JobId, 
}

pub type StopJobResult = Result<(), String>;