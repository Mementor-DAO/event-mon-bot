use std::borrow::Cow;
use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct JobCanister {
    pub canister_id: Principal,
    pub method_name: String,
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
    pub output_template: String,
    pub interval: u32,
    pub state: JobState,
    pub offset: u32,
    pub size: u32,
}

impl Job {
    pub fn canister(
        canister_id: Principal, 
        method_name: String, 
        output_template: String, 
        offset: u32,
        size: u32,
        interval: u32
    ) -> Self {
        Self {
            ty: JobType::Canister(JobCanister{
                canister_id,
                method_name,
            }),
            interval,
            output_template,
            state: JobState::Running,
            offset,
            size
        }
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
