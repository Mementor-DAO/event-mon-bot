mod types;
mod state;
mod utils;
mod guards;
mod lifecycle;
mod updates;
mod memory;
mod services;
mod storage;

use getrandom::register_custom_getrandom;
use monitor_api::{
    lifecycle::init::*, 
    updates::{
        add_job::*, 
        del_job::*
    }
};

ic_cdk::export_candid!();

fn custom_getrandom(
    _: &mut [u8]
) -> Result<(), getrandom::Error> {
    Err(getrandom::Error::UNSUPPORTED)
}

register_custom_getrandom!(custom_getrandom);