use std::borrow::Cow;
use candid::{CandidType, Decode, Encode, Principal};
use ic_ledger_types::AccountIdentifier;
use ic_stable_structures::{storable::Bound, Storable};
use serde::Deserialize;

pub type UserId = Principal;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum UserTransaction {
    IcpWithdraw {
        amount: u64,
        to: AccountIdentifier,
        block_num: u64,
        timestamp: u32,
    },
}

#[derive(Default, CandidType, Deserialize)]
pub struct User {
    pub txs: Vec<UserTransaction>,
}

impl Storable for User {
    fn to_bytes(
        &self
    ) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(
        bytes: std::borrow::Cow<[u8]>
    ) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}