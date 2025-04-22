use monitor_api::updates::del_job::{DelJobArgs, DelJobResult};
use crate::{guards::*, services::manager::manager::JobManager};

#[ic_cdk::update(guard = "owner_only")]
pub fn delete_job(
    args: DelJobArgs
) -> DelJobResult {
    match JobManager::delete(args.job_id) {
        Ok(()) =>  {
            Ok(())
        }
        Err(err) => {
            ic_cdk::println!("error: {}", err);
            Err(err)
        }
    }
}
