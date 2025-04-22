use candid::Principal;
use ic_cdk::api::management_canister::main::{
    canister_info, canister_status, 
    CanisterIdRecord, CanisterInfoRequest, 
    CanisterInfoResponse, CanisterStatusResponse
};

#[allow(unused)]
pub(crate) async fn get_canister_info(
    canister_id: Principal
) -> Result<CanisterInfoResponse, String> {
    let res = canister_info(CanisterInfoRequest { 
        canister_id, 
        num_requested_changes: None 
    }).await.map_err(|e| e.1)?;

    Ok(res.0)
}

pub(crate) async fn get_canister_status(
    canister_id: Principal
) -> Result<CanisterStatusResponse, String> {
    let res = canister_status(CanisterIdRecord { 
        canister_id, 
    }).await.map_err(|e| e.1)?;

    Ok(res.0)
}