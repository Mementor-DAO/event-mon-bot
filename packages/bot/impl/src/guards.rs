use crate::{state, storage::monitor::MonitorStorage};

pub fn admin_only(
) -> Result<(), String> {
    if state::read(|state| ic_cdk::caller() == state.administrator()) {
        Ok(())
    } else {
        Err("Forbidden: admins only".to_string())
    }
}

pub fn monitor_canister_only(
) -> Result<(), String> {
    if MonitorStorage::load_by_canister_id(&ic_cdk::caller()).is_none() {
        Err("Forbidden: monitor canisters only".to_string())
    }
    else {
        Ok(())
    }
}
