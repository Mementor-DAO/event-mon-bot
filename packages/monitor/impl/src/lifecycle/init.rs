use monitor_api::lifecycle::init::InitOrUpgradeArgs;
use ic_cdk::init;
use crate::state::State;
use super::setup;

#[init]
fn init(
    args: InitOrUpgradeArgs
) {
    let state = State::new(
        args.administrator,
        args.bot_canister_id, 
        args.chat,
    );
    setup(
        state
    ).unwrap();
}

