use monitor_api::updates::start_job::{StartJobArgs, StartJobResult};
use crate::{guards::*, services::manager::manager::JobManager};

#[ic_cdk::update(guard = "owner_only")]
pub fn start_job(
    args: StartJobArgs
) -> StartJobResult {
    match JobManager::start(args.job_id) {
        Ok(()) =>  {
            Ok(())
        }
        Err(err) => {
            ic_cdk::println!("error: {}", err);
            Err(err)
        }
    }
}
