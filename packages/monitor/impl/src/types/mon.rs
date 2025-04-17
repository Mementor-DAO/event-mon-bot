use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use super::scheduler::{JobType, Schedulable};

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct MonCanister {
    pub canister_id: Principal,
    pub method_name: String,
    pub output_template: String,
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub enum MonType {
    Canister(MonCanister)
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub enum MonState {
    Idle,
    Running,
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct Mon {
    pub ty: MonType,
    pub interval: u32,
    pub state: MonState,
}

impl Mon {
    pub fn canister(
        canister_id: Principal, 
        method_name: String, 
        output_template: String, 
        interval: u32
    ) -> Self {
        Self {
            ty: MonType::Canister(MonCanister{
                canister_id,
                method_name,
                output_template,
            }),
            interval,
            state: MonState::Running,
        }
    }
}

impl Schedulable<Mon> for Mon {
    fn ty(
        &self
    ) -> JobType {
        JobType::Recurring
    }

    fn interval(
        &self
    ) -> u64 {
        (self.interval as u64) * 1_000
    }
}
