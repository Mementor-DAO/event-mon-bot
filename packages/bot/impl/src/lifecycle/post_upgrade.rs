use bot_api::lifecycle::init::InitOrUpgradeArgs;
use ic_cdk::post_upgrade;
use ic_stable_structures::reader::{BufferedReader, Reader};
use serde::Deserialize;
use crate::{
    lifecycle::READER_WRITER_BUFFER_SIZE, 
    memory::get_upgrades_memory, 
    state::{MonitorWasm, State},
};
use super::setup;

#[post_upgrade]
fn post_upgrade(
    args: InitOrUpgradeArgs
) {
    let memory = get_upgrades_memory();
    let reader = BufferedReader::new(READER_WRITER_BUFFER_SIZE, Reader::new(&memory, 0));
    let mut deserializer = rmp_serde::Deserializer::new(reader);

    let mut state = State::deserialize(&mut deserializer).unwrap();
    state.set_administrator(args.administrator.clone());
    state.set_oc_public_key(args.oc_public_key.clone());
    state.set_monitor_wasm(MonitorWasm::new(args.monitor_wasm));

    setup(
        state
    ).unwrap();
}