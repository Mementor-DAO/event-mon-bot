use candid::CandidType;
use oc_bots_sdk::types::Chat;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType)]
pub struct NotifiyEventsArgs{
    pub chat: Chat,
}