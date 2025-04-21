use bot_api::lifecycle::init::InitOrUpgradeArgs;
use ic_cdk::init;
use crate::state::State;
use super::setup;

#[init]
fn init(
    args: InitOrUpgradeArgs
) {
    let state = State::new(
        args.administrator,
        args.oc_public_key,
        args.monitor_wasm,
    );
    setup(
        state
    ).unwrap();
}

