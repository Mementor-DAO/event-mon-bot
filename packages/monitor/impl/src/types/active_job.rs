use serde::{Deserialize, Serialize};
use super::{job::Job, scheduler::Schedulable};

#[derive(Clone, Serialize, Deserialize)]
pub struct ActiveJob {
    pub interval: u32,
}

impl Schedulable<ActiveJob> for ActiveJob {
    fn repeat(
        &self
    ) -> bool {
        true
    }

    fn interval(
        &self
    ) -> u64 {
        (self.interval as u64) * 1_000
    }
}

impl From<Job> for ActiveJob {
    fn from(
        value: Job
    ) -> Self {
        Self {
            interval: value.interval
        }
    }
}

