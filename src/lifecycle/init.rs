use ic_cdk::init;
use crate::{
    state::State,
    types::init::InitOrUpgradeArgs
};
use super::setup;

#[init]
fn init(
    args: InitOrUpgradeArgs
) {
    ic_cdk::setup();
    let state = State::new(
        args.oc_public_key.clone(), 
        args.administrator.clone()
    );
    setup(state).unwrap();
}

