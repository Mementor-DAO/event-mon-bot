use ic_cdk::init;
use crate::{
    states::{main::MainState, mon::MonState},
    types::init::InitOrUpgradeArgs
};
use super::setup;

#[init]
fn init(
    args: InitOrUpgradeArgs
) {
    ic_cdk::setup();
    let main_state = MainState::new(
        args.oc_public_key.clone(), 
        args.administrator.clone()
    );
    let mon_state = MonState::new(
    );
    setup(
        main_state,
        mon_state
    ).unwrap();
}

