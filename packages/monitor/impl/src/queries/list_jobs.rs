use monitor_api::queries::list_jobs::{ListJobsArgs, ListJobsResult};
use crate::{guards::*, services::manager::manager::JobManager};

#[ic_cdk::query(guard = "owner_only")]
pub fn list_jobs(
    args: ListJobsArgs
) -> ListJobsResult {
    Ok(
        JobManager::list(args.offset as _, args.size as _)
    )
}
