use std::collections::{BTreeSet, HashMap};
use serde::{Deserialize, Serialize};

pub type JobId = u64;

#[allow(unused)]
pub enum JobType {
    Recurring,
    Once,
}

pub trait Schedulable<T> {
    fn ty(&self) -> JobType;
    fn interval(&self) -> u64;
}

#[derive(Serialize, Deserialize)]
pub struct Scheduler<T> {
    pub jobs: HashMap<JobId, T>,
    pub ordered: BTreeSet<(u64, JobId)>,
    pub next_id: JobId,
}