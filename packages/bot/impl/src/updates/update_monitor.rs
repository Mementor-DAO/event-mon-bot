use bot_api::updates::update_monitor::{
    UpdateMonitorArgs, UpdateMonitorResponse
};
use candid::Encode;
use ic_cdk::management_canister::{
    self, CanisterInstallMode, InstallCodeArgs
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
    let (administrator, chat) = state::read(|s| 
        (
            s.administrator().clone(),
            s.chat().clone()
        )
    );
    let bot_canister_id = ic_cdk::api::canister_self();

    state::mutate(|s| {
        s.set_monitor_wasm(Some(MonitorWasm::new(args.image.clone())));
    });

    MonitorStorage::for_each(async |_id, mon| {
        if let Err(err) = management_canister::install_code(
            &InstallCodeArgs { 
                mode: CanisterInstallMode::Upgrade(None), 
                canister_id: mon.canister_id, 
                wasm_module: args.image.clone(), 
                arg: Encode!(&InitOrUpgradeArgs { 
                    administrator, 
                    bot_canister_id,
                    chat
                }).unwrap()
            }
        ).await {
            ic_cdk::println!("error: updating monitor canister {}: {}", mon.canister_id.to_text(), err.to_string());
        };
    }).await;

    Ok(UpdateMonitorResponse {  })
}