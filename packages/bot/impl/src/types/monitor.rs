use std::{borrow::Cow, fmt::Display};
use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use monitor_api::updates::add_job::JobId;
use oc_bots_sdk::types::Chat;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct MonitorId(pub Chat);

impl From<Chat> for MonitorId {
    fn from(
        value: Chat
    ) -> Self {
        Self(value)
    }
}

impl Display for MonitorId {
    fn fmt(
        &self, 
        fmt: &mut std::fmt::Formatter<'_>
    ) -> std::fmt::Result {
        fmt.write_fmt(format_args!(
            "{}.{}", 
            self.0.canister_id(), 
            self.0.channel_id().unwrap_or(0)
        ))
    }
}

impl Ord for MonitorId {
    fn cmp(
        &self, 
        other: &Self
    ) -> std::cmp::Ordering {
        match self.0.canister_id().cmp(&other.0.canister_id()) {
            std::cmp::Ordering::Equal => {
                self.0.channel_id().cmp(&other.0.channel_id())
            },
            other => other,
        }
    }
}

impl PartialOrd for MonitorId {
    fn partial_cmp(
        &self, 
        other: &Self
    ) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Storable for MonitorId {
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

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub enum MonitorState {
    Idle,
    Running
}

#[derive(Clone, Serialize, Deserialize, CandidType)]
pub struct Monitor {
    pub chat: Chat,
    pub state: MonitorState,
    pub canister_id: Principal,
    pub wasm_hash: Vec<u8>,
    pub jobs: Vec<JobId>
}

impl Monitor {
    pub fn new(
        chat: Chat,
        canister_id: Principal,
        wasm_hash: Vec<u8>
    ) -> Self {
        Self {
            chat,
            state: MonitorState::Running,
            canister_id,
            wasm_hash,
            jobs: vec![],
        }
    }
}

impl Storable for Monitor {
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