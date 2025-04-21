use candid::Principal;
use ic_cdk::api::management_canister::main::{
    canister_info, CanisterInfoRequest, CanisterInfoResponse
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