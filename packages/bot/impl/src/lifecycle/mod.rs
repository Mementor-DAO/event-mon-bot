use std::{cell::RefCell, time::Duration};
use ic_ledger_types::DEFAULT_SUBACCOUNT;
use crate::{
    services::{
        fund::fund::{FundCanisterConfig, FundService}, 
        monitor::MonitorService
    }, 
    state::{self, State}
};

pub mod init;
pub mod post_upgrade;
pub mod pre_upgrade;

const READER_WRITER_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB

thread_local! {
    static FUND_SERVICE: RefCell<FundService> = RefCell::new(FundService::new());
}

pub(crate) fn setup(
    state: State
) -> Result<(), String> {
    ic_wasi_polyfill::init(&[0u8; 32], &[]);

    state::init(state);

    // start fund service (to auto top-up our canisters)
    FUND_SERVICE.with_borrow_mut(|service| {
        service.start(
            vec![FundCanisterConfig {
                canister_id: ic_cdk::id(),
                from_subaccount: DEFAULT_SUBACCOUNT,
                min_cycles:  5_000_000_000_000,
                fund_cycles: 1_000_000_000_000,
            }], 
            30 * 60 // every 30 minutes
        );
    });

    MonitorService::start();

    // update all deployed monitors if the wasm changed
    ic_cdk_timers::set_timer(
        Duration::ZERO, 
        || ic_cdk::spawn(update_monitors())
    );

    Ok(())
}

async fn update_monitors(
) {
    let (administrator, wasm) = state::read(|s| 
        (
            s.administrator().clone(),
            s.monitor_wasm().clone()
        )
    );
    
    MonitorService::stop_all().await;
    MonitorService::update_all(administrator, wasm).await;
    MonitorService::start_all().await;
}