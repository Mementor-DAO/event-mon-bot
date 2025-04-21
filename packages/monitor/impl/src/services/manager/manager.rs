use crate::{state, storage::job::job::JobStorage, types::{job::{Job, JobType}, scheduler::JobId}};

pub struct JobManager;

impl JobManager {
    pub fn new(
        job: Job
    ) -> Result<(), String> {
        state::mutate(|s| {
            let scheduler = s.scheduler_mut();
            match scheduler.add(job.clone(), ic_cdk::api::time() / 1_000_000) {
                Ok((job_id, next_due)) => {
                    JobStorage::save(job_id, job);
                    if next_due {
                        scheduler.start_if_required(Self::timer_cb);
                    }
                    else {
                        scheduler.restart(Self::timer_cb);
                    }
                    Ok(())
                },
                Err(err) => {
                    Err(err)
                },
            }
        })
    }

    fn timer_cb(
    ) {
        state::mutate(|s| {
            s.scheduler_mut().process(
                Self::timer_cb,
                Self::job_cb
            );
        });
    }

    async fn job_cb(
        job_id: JobId
    ) {    
        if let Some(job) = JobStorage::load(&job_id) {
            match &job.ty {
                JobType::Canister(can) => {
                    ic_cdk::call::<(), ()>(
                        can.canister_id, 
                        &can.method_name, 
                        ()
                    ).await;
                },
            }
        }
    }    
    
}