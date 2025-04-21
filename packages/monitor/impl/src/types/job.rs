use std::borrow::Cow;
use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use super::scheduler::Schedulable;

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct JobCanister {
    pub canister_id: Principal,
    pub method_name: String,
    pub output_template: String,
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub enum JobType {
    Canister(JobCanister)
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub enum JobState {
    Idle,
    Running,
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct Job {
    pub ty: JobType,
    pub interval: u32,
    pub state: JobState,
}

impl Job {
    pub fn canister(
        canister_id: Principal, 
        method_name: String, 
        output_template: String, 
        interval: u32
    ) -> Self {
        Self {
            ty: JobType::Canister(JobCanister{
                canister_id,
                method_name,
                output_template,
            }),
            interval,
            state: JobState::Running,
        }
    }
}

impl Schedulable<Job> for Job {
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

impl Storable for Job {
    fn to_bytes(
        &self
    ) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(
        bytes: Cow<[u8]>
    ) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}
