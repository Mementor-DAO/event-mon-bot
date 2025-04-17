use crate::{state, utils::ic::get_canister_info};

pub fn admin_only(
) -> Result<(), String> {
    if state::read(|state| ic_cdk::api::msg_caller() == state.administrator()) {
        Ok(())
    } else {
        Err("Forbidden: admins only".to_string())
    }
}

pub async fn monitor_canister_only(
) -> Result<(), String> {
    let caller_info = get_canister_info(ic_cdk::api::msg_caller()).await.unwrap();

    if state::read(|state| caller_info.module_hash.unwrap() == state.monitor_wasm().unwrap().hash) {
        Ok(())
    } else {
        Err("Forbidden: monitor canisters only".to_string())
    }
}
