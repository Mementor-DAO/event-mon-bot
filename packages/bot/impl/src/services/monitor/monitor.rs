use candid::{Encode, Principal};
use ic_cdk::api::management_canister::main::{
    install_code, start_canister, stop_canister, 
    CanisterIdRecord, CanisterInstallMode, InstallCodeArgument
};
use monitor_api::lifecycle::init::InitOrUpgradeArgs;
use crate::{
    state::MonitorWasm, 
    storage::monitor::MonitorStorage, 
    types::monitor::MonitorState
};

pub struct MonitorService;

impl MonitorService {
    pub async fn start_all(
    ) {
        MonitorStorage::for_each(async |id, mut mon| {
            ic_cdk::println!("info: starting monitor({})", mon.canister_id.to_text());
            if let Err(err) = start_canister(CanisterIdRecord {
                canister_id: mon.canister_id.clone()
            }).await {
                ic_cdk::println!("error: starting monitor({}): {}", mon.canister_id.to_text(), err.1);
            }
        
            mon.state = MonitorState::Running;
            MonitorStorage::save(id, mon);
        }).await;
    }
    
    pub async fn stop_all(
    ) {
        MonitorStorage::for_each(async |id, mut mon| {
            ic_cdk::println!("info: stopping monitor({})", mon.canister_id.to_text());
            if let Err(err) = stop_canister(CanisterIdRecord {
                canister_id: mon.canister_id.clone()
            }).await {
                ic_cdk::println!("error: stopping monitor({}): {}", mon.canister_id.to_text(), err.1);
            }
        
            mon.state = MonitorState::Idle;
            MonitorStorage::save(id, mon);
        }).await;
    }

    pub async fn update_all(
        administrator: Principal,
        wasm: MonitorWasm
    ) {
        let bot_canister_id = ic_cdk::api::id();
    
        MonitorStorage::for_each(async |id, mut mon| {
            if wasm.hash != mon.wasm_hash {
                ic_cdk::println!("info: updating monitor({})", mon.canister_id.to_text());
                if let Err(err) = install_code(
                    InstallCodeArgument { 
                        mode: CanisterInstallMode::Upgrade(None), 
                        canister_id: mon.canister_id, 
                        wasm_module: wasm.image.clone(), 
                        arg: Encode!(&InitOrUpgradeArgs { 
                            administrator, 
                            bot_canister_id,
                        }).unwrap()
                    }
                ).await {
                    ic_cdk::println!("error: updating monitor({}): {}", mon.canister_id.to_text(), err.1);
                }
                else {
                    mon.wasm_hash = wasm.hash.clone();
                    MonitorStorage::save(id, mon);
                }
            }
        }).await;
    }
}