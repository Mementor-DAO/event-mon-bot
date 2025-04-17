use candid::{CandidType, Principal};
use oc_bots_sdk::types::Chat;
use serde::{Deserialize, Serialize};

#[derive(Clone, CandidType, Serialize, Deserialize)]
pub struct InitOrUpgradeArgs {
    pub oc_public_key: String,
    pub administrator: Principal,
    pub chat: Chat,
}

