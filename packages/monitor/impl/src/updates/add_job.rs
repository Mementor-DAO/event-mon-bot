use monitor_api::updates::add_job::AddJobArgs;
use crate::{guards::*, services::manager::manager::JobManager, types::job::Job};

#[ic_cdk::update(guard = "owner_only")]
pub fn add_job(
    args: AddJobArgs
) -> Result<(), String> {
    let job = Job::canister(
        args.canister_id,
        args.method_name,
        args.output_template,
        args.interval
    );

    if let Err(err) = JobManager::new(job) {
        ic_cdk::println!("error: {}", err);
        Err(err)
    }
    else {
        Ok(())
    }
}
