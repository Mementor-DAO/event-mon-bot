use monitor_api::updates::stop_job::{StopJobArgs, StopJobResult};
use crate::{guards::*, services::manager::manager::JobManager};

#[ic_cdk::update(guard = "owner_only")]
pub fn stop_job(
    args: StopJobArgs
) -> StopJobResult {
    match JobManager::stop(args.job_id) {
        Ok(()) =>  {
            Ok(())
        }
        Err(err) => {
            ic_cdk::println!("error: {}", err);
            Err(err)
        }
    }
}
