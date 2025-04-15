use candid::{CandidType, Principal};
use oc_bots_sdk::types::Chat;
use serde::{Deserialize, Serialize};
use super::scheduler::{JobType, Schedulable};

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct JobCanister {
    pub canister_id: Principal,
    pub method_name: String,
    pub output_template: String,
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub enum JobKind {
    Canister(JobCanister)
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub enum JobState {
    Idle,
    Running,
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct Job {
    pub kind: JobKind,
    pub interval: u32,
    pub chat: Chat,
    pub chat_job_id: u8,
    pub state: JobState,
}

impl Job {
    pub fn new_canister(
        canister_id: Principal, 
        method_name: String, 
        output_template: String, 
        interval: u32,
        chat: Chat
    ) -> Self {
        Self {
            kind: JobKind::Canister(JobCanister{
                canister_id,
                method_name,
                output_template,
            }),
            interval,
            chat,
            chat_job_id: 0,
            state: JobState::Running,
        }
    }
}

impl Schedulable<Job> for Job {
    fn ty(
        &self
    ) -> JobType {
        JobType::Recurring
    }

    fn interval(
        &self
    ) -> u64 {
        self.interval as _
    }

    fn chat(
        &self
    ) -> Chat {
        self.chat
    }

    fn chat_job_id(
        &self
    ) -> u8 {
        self.chat_job_id
    }

    fn set_chat_job_id(
        &mut self, 
        chat_job_id: u8
    ) {
        self.chat_job_id = chat_job_id;
    }
}
