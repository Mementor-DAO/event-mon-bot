use std::fmt::Display;
use candid::{CandidType, Principal};
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

impl Display for JobType {
    fn fmt(
        &self, 
        fmt: &mut std::fmt::Formatter<'_>
    ) -> std::fmt::Result {
        let s = match self {
            JobType::Canister(can) => {
                format!("Canister(id:{}, method:{})", can.canister_id.to_text(), can.method_name)
            },
        };

        fmt.write_fmt(format_args!("{}", s))
    }
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub enum JobState {
    Idle,
    Running,
}

impl Display for JobState {
    fn fmt(
        &self, 
        fmt: &mut std::fmt::Formatter<'_>
    ) -> std::fmt::Result {
        let s = match self {
            JobState::Idle => "idle",
            JobState::Running => "running",
        };

        fmt.write_fmt(format_args!("{}", s))
    }
}