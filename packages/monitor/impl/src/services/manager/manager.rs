use std::collections::BTreeMap;
use bot_api::updates::notify_events::{NotifiyEventsArgs, NotifiyEventsResponse};
use candid::Principal;
use icrc_ledger_types::icrc::generic_value::Value;
use monitor_api::types::job::JobType;
use crate::{
    state, 
    storage::job::job::JobStorage, 
    types::{
        job::Job, 
        scheduler::JobId
    }
};

pub struct JobManager;

impl JobManager {
    pub fn add(
        job: Job
    ) -> Result<JobId, String> {
        let now = ic_cdk::api::time() / 1_000_000;

        match state::mutate(|s| {
            let scheduler = s.scheduler_mut();
            match scheduler.add(job.clone().into(), now) {
                Ok((job_id, next_due)) => {
                    JobStorage::save(job_id, job);
                    Ok((job_id, next_due))
                },
                Err(err) => {
                    Err(err)
                },
            }
        }) {
            Ok((job_id, next_due)) => {
                state::read(|s| {
                    if next_due {
                        s.scheduler().start_if_required(Self::timer_cb);
                    }
                    else {
                        s.scheduler().restart(Self::timer_cb);
                    }
                });

                Ok(job_id)
            },
            Err(err) => {
                Err(err)
            }
        }
    }

    pub fn delete(
        job_id: JobId
    ) -> Result<(), String> {
        state::mutate(|s| {
            s.scheduler_mut()
                .delete(job_id)
        })?;

        JobStorage::remove(job_id);

        Ok(())
    }

    pub fn list(
        offset: usize,
        size: usize
    ) -> Vec<monitor_api::queries::list_jobs::Job> {
        JobStorage::list(offset, size).iter()
            .cloned()
            .map(|(id, job)| monitor_api::queries::list_jobs::Job {
                id,
                ty: job.ty,
                output_template: job.output_template,
                interval: job.interval,
                state: job.state,
            })
            .collect()
    }
    

    pub fn start_if_required(
    ) {
        state::read(|s| {
            s.scheduler()
                .start_if_required(Self::timer_cb);
        });
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
        if let Some(mut job) = JobStorage::load(job_id) {
            match job.ty.clone() {
                JobType::Canister(can) => {
                    match Self::query_canister(
                        &can.canister_id, 
                        &can.method_name, 
                        &mut job
                    ).await {
                        Ok(messages) => {
                            if messages.len() > 0 {
                                if let Err(err) = Self::notify_events(messages).await {
                                    ic_cdk::println!("error: notifying events: {}", err);    
                                }
                            }
                        }
                        Err(err) => {
                            ic_cdk::println!("error: calling {}.{}: {}", can.canister_id.to_text(), can.method_name, err);
                        }
                    };
                },
            }

            JobStorage::save(job_id, job);
        }
    }

    async fn query_canister(
        canister_id: &Principal, 
        method_name: &String, 
        job: &mut Job
    ) -> Result<Vec<String>, String> {
        
        ic_cdk::println!("info: quering canister {}.{}", canister_id, method_name);
        let events = ic_cdk::call::<(u32, u32), (Result<Vec<BTreeMap<String, Value>>, String>, )>(
            canister_id.clone(), 
            method_name, 
            (job.offset, job.size)
        ).await
            .map_err(|e| e.1)?
            .0?;

        job.offset += events.len() as u32;

        let mut messages = vec![];

        for event in events {
            let mut text = job.output_template.clone();
            for (key, value) in event.keys().zip(event.values()) {
                text = text.replace(&format!("{{{}}}", key), &value.to_string());
            }
            messages.push(text);
        }

        Ok(messages)
    }
    
    async fn notify_events(
        messages: Vec<String>
    ) -> Result<(), String> {
        let canister_id = state::read(|s| s.bot_canister_id().clone());
        
        ic_cdk::call::<(NotifiyEventsArgs, ), (NotifiyEventsResponse, )>(
            canister_id, 
            "notify_events", 
            (NotifiyEventsArgs {
                messages,
            },)
        ).await
            .map_err(|e| e.1)?
            .0?;

        Ok(())
    }
}