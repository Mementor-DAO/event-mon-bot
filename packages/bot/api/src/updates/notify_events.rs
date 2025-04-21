use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType)]
pub struct NotifiyEventsArgs{
    pub messages: Vec<String>,
}