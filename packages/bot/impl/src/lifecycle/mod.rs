use std::time::Duration;
use crate::{services::monitor::MonitorService, state::{self, State}};

pub mod init;
pub mod post_upgrade;
pub mod pre_upgrade;

const READER_WRITER_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub(crate) fn setup(
    state: State
) -> Result<(), String> {
    ic_wasi_polyfill::init(&[0u8; 32], &[]);

    state::init(state);

    ic_cdk_timers::set_timer(
        Duration::from_secs(1), 
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