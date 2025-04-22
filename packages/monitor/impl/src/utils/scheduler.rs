use std::{cell::Cell, collections::{BTreeSet, HashMap}, future::Future, time::Duration};
use ic_cdk_timers::TimerId;
use crate::
    types::scheduler::{JobId, Schedulable, Scheduler}
;

// code adapted from https://github.com/open-chat-labs/open-chat-bots/blob/main/rs/canister/examples/reminder/src/model/reminders.rs

const MAX_JOBS: usize = 100;

thread_local! {
    static TIMER_ID: Cell<Option<TimerId>> = Cell::default();
}

impl<T> Scheduler<T>
    where T: Clone + Schedulable<T> {

    pub fn new(
    ) -> Self {
        Self {
            jobs: HashMap::new(),
            ordered: BTreeSet::new(),
            next_id: 0,
        }
    }

    pub fn add(
        &mut self,
        job: T,
        now: u64,
    ) -> Result<(JobId, bool), String> {
        if self.jobs.len() >= MAX_JOBS {
            return Err("Too many jobs".to_string());
        }

        let timestamp = now + job.interval();

        let job_id = self.next_id;
        self.next_id += 1;

        self.jobs.insert(
            job_id,
            job,
        );

        self.ordered.insert((timestamp, job_id));

        let next_due = self.peek().unwrap().1 == job_id;

        Ok((job_id, next_due))
    }

    pub fn process<TF, JF, JR>(
        &mut self,
        timer_cb: TF,
        job_cb: JF
    ) where 
        TF: FnOnce() -> () + 'static,
        JF: FnOnce(JobId) -> JR + Copy + 'static,
        JR: Future<Output = ()> + 'static {
            
        TIMER_ID.set(None);

        let now = ic_cdk::api::time() / 1_000_000;
        while let Some(job_id) = self.pop_next_due_job(now) {
            ic_cdk::spawn(job_cb(job_id));
        }

        self.start_if_required(timer_cb);
    }

    pub fn start_if_required<F>(
        &self,
        timer_cb: F
    ) -> bool 
        where F: 'static + FnOnce() -> () {
        if TIMER_ID.get().is_none() {
            if let Some(next_job_due) = self.peek().map(|(timestamp, _)| timestamp) {
                let now = ic_cdk::api::time() / 1_000_000;
                let timer_id = ic_cdk_timers::set_timer(
                    Duration::from_millis(next_job_due.saturating_sub(now)),
                    timer_cb,
                );
                TIMER_ID.set(Some(timer_id));
                return true;
            }
        }
    
        false
    }
    
    pub fn restart<F>(
        &self,
        timer_cb: F
    ) where F: 'static + FnOnce() -> () {
        if let Some(timer_id) = TIMER_ID.get() {
            ic_cdk_timers::clear_timer(timer_id);
            TIMER_ID.set(None);
        }
    
        self.start_if_required(timer_cb);
    }

    fn peek(
        &self
    ) -> Option<(u64, JobId)> {
        self.ordered.iter().next().copied()
    }

    fn pop_next_due_job(
        &mut self, 
        now: u64
    ) -> Option<JobId> {
        let (timestamp, job_id) = self.peek()?;

        if timestamp > now {
            return None;
        }

        self.ordered.pop_first();

        let job = self.jobs.get_mut(&job_id)?;

        if let Some(next) =
            Self::next_job_time(&job, now) {
            self.ordered.insert((next, job_id));
        } 
        else {
            self.jobs.remove(&job_id).unwrap();
        }

        Some(job_id)
    }

    #[allow(unused)]
    pub fn delete(
        &mut self,
        job_id: u64
    ) -> Result<T, String> {
        self.jobs
            .remove(&job_id)
            .ok_or("Job not found".to_string())
    }

    fn next_job_time(
        job: &T,
        now: u64,
    ) -> Option<u64> {
        if job.repeat() {
            Some(now + job.interval())
        }
        else {
            None
        }
    }
}