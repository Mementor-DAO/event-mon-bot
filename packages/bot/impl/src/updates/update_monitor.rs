use bot_api::updates::update_monitor::{
    UpdateMonitorArgs, UpdateMonitorResponse
};
use candid::Encode;
use ic_cdk::api::management_canister::main::{
    install_code, CanisterInstallMode, InstallCodeArgument
};
use monitor_api::lifecycle::init::InitOrUpgradeArgs;
use crate::{
    guards::*, 
    state::{self, MonitorWasm}, 
    storage::monitor::MonitorStorage
};

#[ic_cdk::update(guard = "admin_only")]
async fn update_monitor(
    args: UpdateMonitorArgs
) -> Result<UpdateMonitorResponse, String> {
    let administrator = state::read(|s| 
        s.administrator().clone()
    );
    let bot_canister_id = ic_cdk::api::id();

    state::mutate(|s| {
        s.set_monitor_wasm(MonitorWasm::new(args.wasm.clone()));
    });

    MonitorStorage::for_each(async |_id, mon| {
        if let Err(err) = install_code(
            InstallCodeArgument { 
                mode: CanisterInstallMode::Upgrade(None), 
                canister_id: mon.canister_id, 
                wasm_module: args.wasm.clone(), 
                arg: Encode!(&InitOrUpgradeArgs { 
                    administrator, 
                    bot_canister_id,
                }).unwrap()
            }
        ).await {
            ic_cdk::println!("error: updating monitor({}): {}", mon.canister_id.to_text(), err.1);
        };
    }).await;

    Ok(UpdateMonitorResponse {  })
}