use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(Clone, CandidType, Serialize, Deserialize)]
pub struct InitOrUpgradeArgs {
    pub oc_public_key: String,
    pub administrator: Principal,
}

