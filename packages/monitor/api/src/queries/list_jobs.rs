use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{types::job::{JobState, JobType}, updates::add_job::JobId};

#[derive(Serialize, Deserialize, CandidType)]
pub struct ListJobsArgs {
    pub offset: u32,
    pub size: u32,
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct Job {
    pub id: JobId,
    pub ty: JobType,
    pub output_template: String,
    pub interval: u32,
    pub state: JobState,
}

pub type ListJobsResult = Result<Vec<Job>, String>;