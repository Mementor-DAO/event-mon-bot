use std::cell::RefCell;
use ic_stable_structures::BTreeMap;
use crate::{
    memory::{get_jobs_memory, Memory}, 
    types::{job::Job, scheduler::JobId}
};

pub struct JobStorage;

thread_local! {
    static JOBS: RefCell<BTreeMap<JobId, Job, Memory>> = RefCell::new(
        BTreeMap::init(
            get_jobs_memory()
        )
    );
}

impl JobStorage {
    pub fn save(
        id: JobId,
        job: Job
    ) {
        JOBS.with_borrow_mut(|jobs| {
            jobs.insert(id, job)
        });
    }

    pub fn load(
        id: &JobId
    ) -> Option<Job> {
        JOBS.with_borrow(|jobs| {
            jobs.get(&id)
        })
    }
}