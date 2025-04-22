use candid::CandidType;
use serde::{Deserialize, Serialize};
use super::add_job::JobId;

#[derive(Serialize, Deserialize, CandidType)]
pub struct StartJobArgs {
    pub job_id: JobId, 
}

pub type StartJobResult = Result<(), String>;