use crate::{
    guards::*, services::monitor::MonitorService
};

#[ic_cdk::update(guard = "admin_only")]
async fn start_monitors(
) {
    MonitorService::start_all().await;
}