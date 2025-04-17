use candid::Principal;
use ic_cdk::management_canister::{canister_info, CanisterInfoArgs, CanisterInfoResult};

pub(crate) async fn get_canister_info(
    canister_id: Principal
) -> Result<CanisterInfoResult, String> {
    let res = canister_info(&CanisterInfoArgs { 
        canister_id, 
        num_requested_changes: None 
    }).await.map_err(|e| e.to_string())?;

    Ok(res)
}