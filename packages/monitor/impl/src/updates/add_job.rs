use monitor_api::updates::add_job::{AddJobArgs, AddJobResult};
use crate::{guards::*, services::manager::manager::JobManager, types::job::Job};

#[ic_cdk::update(guard = "owner_only")]
pub async fn add_job(
    args: AddJobArgs
) -> AddJobResult {
    let offset = if args.offset > 0 {
        args.offset
    }
    else {
        JobManager::get_current_offset(
            &args.canister_id, &args.method_name
        ).await?
    };

    let job = Job::canister(
        args.canister_id,
        args.method_name,
        args.interval,
        args.batch_size,
        args.output_template,
        offset
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
