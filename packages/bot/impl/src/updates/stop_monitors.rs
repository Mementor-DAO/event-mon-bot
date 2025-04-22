use crate::{
    guards::*, services::monitor::MonitorService
};

#[ic_cdk::update(guard = "admin_only")]
async fn stop_monitors(
) {
    MonitorService::stop_all().await;
}