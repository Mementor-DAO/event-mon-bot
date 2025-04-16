use std::collections::{BTreeMap, BTreeSet, HashMap};
use oc_bots_sdk::types::{Chat, TimestampMillis};
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
    fn chat(&self) -> Chat;
    fn chat_job_id(&self) -> u8;
    fn set_chat_job_id(&mut self, chat_job_id: u8);
}

#[derive(Serialize, Deserialize)]
pub struct Scheduler<T> {
    pub jobs: HashMap<JobId, T>,
    pub per_chat: HashMap<Chat, BTreeMap<u8, JobId>>,
    pub ordered: BTreeSet<(TimestampMillis, JobId)>,
    pub next_id: JobId,
}