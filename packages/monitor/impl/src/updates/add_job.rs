use monitor_api::updates::add_job::{AddJobArgs, AddJobResult};
use crate::{guards::*, services::manager::manager::JobManager, types::job::Job};

#[ic_cdk::update(guard = "owner_only")]
pub fn add_job(
    args: AddJobArgs
) -> AddJobResult {
    let job = Job::canister(
        args.canister_id,
        args.method_name,
        args.output_template,
        args.offset,
        args.size,
        args.interval
    );

    match JobManager::add(job) {
        Ok(job_id) =>  {
            Ok(job_id)
        }
        Err(err) => {
            ic_cdk::println!("error: {}", err);
            Err(err)
        }
    }
}
