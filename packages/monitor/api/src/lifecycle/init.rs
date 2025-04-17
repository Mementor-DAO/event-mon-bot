use candid::{CandidType, Principal};
use oc_bots_sdk::types::Chat;
use serde::{Deserialize, Serialize};

#[derive(Clone, CandidType, Serialize, Deserialize)]
pub struct InitOrUpgradeArgs {
    pub administrator: Principal,
    pub bot_canister_id: Principal,
    pub chat: Chat,
}

