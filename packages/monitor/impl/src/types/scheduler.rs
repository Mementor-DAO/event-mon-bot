use std::collections::{BTreeSet, HashMap};
use serde::{Deserialize, Serialize};

pub type JobId = u64;

pub trait Schedulable<T> {
    fn repeat(&self) -> bool;
    fn interval(&self) -> u64;
}

#[derive(Serialize, Deserialize)]
pub struct Scheduler<T> {
    pub jobs: HashMap<JobId, T>,
    pub ordered: BTreeSet<(u64, JobId)>,
    pub next_id: JobId,
}