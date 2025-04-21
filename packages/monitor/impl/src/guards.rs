use crate::state;

#[allow(unused)]
pub fn admin_only(
) -> Result<(), String> {
    if state::read(|state| ic_cdk::caller() == state.administrator()) {
        Ok(())
    } else {
        Err("Forbidden: admins only".to_string())
    }
}

#[allow(unused)]
pub fn owner_only(
) -> Result<(), String> {
    if state::read(|state| ic_cdk::caller() == *state.bot_canister_id()) {
        Ok(())
    } else {
        Err("Forbidden: owner only".to_string())
    }
}
