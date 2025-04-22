use candid::{Encode, Principal};
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, start_canister, stop_canister, 
    CanisterIdRecord, CanisterInstallMode, CanisterSettings, 
    CreateCanisterArgument, InstallCodeArgument, LogVisibility
};
use monitor_api::{
    lifecycle::init::InitOrUpgradeArgs, queries::list_jobs::{Job, ListJobsArgs, ListJobsResult}, updates::{
        add_job::{AddJobArgs, AddJobResult, JobId}, 
        del_job::{DelJobArgs, DelJobResult}
    }
};
use oc_bots_sdk::types::Chat;
use crate::{
    state::MonitorWasm, 
    storage::monitor::MonitorStorage, 
    types::monitor::{Monitor, MonitorId, MonitorState}, 
    DEPLOY_MONITOR_CYCLES
};

const MIN_INTERVAL: u32 = 15;
const MAX_INTERVAL: u32 = 60;
const ITEMS_PER_PAGE: u32 = 16;

pub struct MonitorService;

impl MonitorService {
    pub async fn deploy(
        chat: Chat,
        administrator: Principal,
        wasm: MonitorWasm
    ) -> Result<Principal, String> {
        let mon_id = chat.into();
        
        if let Some(mon) = MonitorStorage::load(&mon_id) {
            return Err(format!("Monitor already deployed. Canister id: {}", mon.canister_id))
        }
        
        let bot_canister_id = ic_cdk::id();

        let canister_id = create_canister(
            CreateCanisterArgument {
                settings: Some(CanisterSettings{
                    controllers: Some(vec![
                        bot_canister_id,
                        administrator
                    ]),
                    compute_allocation: None,
                    memory_allocation: None,
                    freezing_threshold: None,
                    reserved_cycles_limit: None,
                    log_visibility: Some(LogVisibility::Public),
                    wasm_memory_limit: None,
                }),
            }, 
            DEPLOY_MONITOR_CYCLES
        ).await
            .map_err(|e| e.1)?
            .0.canister_id;

        ic_cdk::api::management_canister::main::install_code(InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            wasm_module: wasm.image,
            arg: Encode!(&InitOrUpgradeArgs { 
                administrator, 
                bot_canister_id,
            }).unwrap()
        }).await
            .map_err(|e| e.1)?;

        MonitorStorage::save(
            mon_id, 
            Monitor::new(chat, canister_id, wasm.hash)
        );

        Ok(canister_id)
    }

    pub async fn add_canister_job(
        mon_id: MonitorId,
        canister_id: Principal,
        method_name: String,
        output_template: String,
        interval: u32
    ) -> Result<JobId, String> {
        let mut mon = if let Some(mon) = MonitorStorage::load(&mon_id) {
            mon
        }
        else {
            return Err(format!("Unknown monitor id: {}", mon_id));
        };

        if interval < MIN_INTERVAL {
            return Err(format!("Interval too low. Min: {}", MIN_INTERVAL));
        }
        else if interval > MAX_INTERVAL {
            return Err(format!("Interval too high. Max: {}", MAX_INTERVAL));
        }
        
        let job_id = ic_cdk::call::<(AddJobArgs, ), (AddJobResult, )>(
            mon.canister_id, 
            "add_job", 
            (AddJobArgs {
                canister_id,
                method_name,
                output_template,
                offset: 0,
                size: 8,
                interval,
            }, )
        ).await.map_err(|e| e.1)?.0?;

        mon.jobs.push(job_id);
        MonitorStorage::save(mon_id, mon);

        Ok(job_id)
    }

    pub async fn del_job(
        mon_id: MonitorId,
        job_id: JobId
    ) -> Result<(), String> {
        let mut mon = if let Some(mon) = MonitorStorage::load(&mon_id) {
            mon
        }
        else {
            return Err("Unknown monitor id".to_string());
        };

        ic_cdk::call::<(DelJobArgs, ), (DelJobResult, )>(
            mon.canister_id, 
            "delete_job", 
            (DelJobArgs {
                job_id,
            },)
        ).await.map_err(|e| e.1)?.0?;

        mon.jobs.retain(|id| *id == job_id);
        MonitorStorage::save(mon_id, mon);

        Ok(())
    }

    pub async fn list_jobs(
        mon_id: MonitorId,
        page: u32
    ) -> Result<Vec<Job>, String> {
        let mon = if let Some(mon) = MonitorStorage::load(&mon_id) {
            mon
        }
        else {
            return Err("Unknown monitor id".to_string());
        };

        let jobs = ic_cdk::call::<(ListJobsArgs, ), (ListJobsResult, )>(
            mon.canister_id, 
            "list_jobs", 
            (ListJobsArgs {
                offset: page * ITEMS_PER_PAGE,
                size: ITEMS_PER_PAGE
            },)
        ).await.map_err(|e| e.1)?.0?;

        Ok(jobs)
    }

    pub async fn start_all(
    ) {
        MonitorStorage::for_each(async |id, mut mon| {
            ic_cdk::println!("info: starting monitor({})", mon.canister_id.to_text());
            if let Err(err) = start_canister(CanisterIdRecord {
                canister_id: mon.canister_id.clone()
            }).await {
                ic_cdk::println!("error: starting monitor({}): {}", mon.canister_id.to_text(), err.1);
            }
        
            mon.state = MonitorState::Running;
            MonitorStorage::save(id, mon);
        }).await;
    }
    
    pub async fn stop_all(
    ) {
        MonitorStorage::for_each(async |id, mut mon| {
            ic_cdk::println!("info: stopping monitor({})", mon.canister_id.to_text());
            if let Err(err) = stop_canister(CanisterIdRecord {
                canister_id: mon.canister_id.clone()
            }).await {
                ic_cdk::println!("error: stopping monitor({}): {}", mon.canister_id.to_text(), err.1);
            }
        
            mon.state = MonitorState::Idle;
            MonitorStorage::save(id, mon);
        }).await;
    }

    pub async fn update_all(
        administrator: Principal,
        wasm: MonitorWasm
    ) {
        let bot_canister_id = ic_cdk::api::id();
    
        MonitorStorage::for_each(async |id, mut mon| {
            if wasm.hash != mon.wasm_hash {
                ic_cdk::println!("info: updating monitor({})", mon.canister_id.to_text());
                if let Err(err) = install_code(
                    InstallCodeArgument { 
                        mode: CanisterInstallMode::Upgrade(None), 
                        canister_id: mon.canister_id, 
                        wasm_module: wasm.image.clone(), 
                        arg: Encode!(&InitOrUpgradeArgs { 
                            administrator, 
                            bot_canister_id,
                        }).unwrap()
                    }
                ).await {
                    ic_cdk::println!("error: updating monitor({}): {}", mon.canister_id.to_text(), err.1);
                }
                else {
                    mon.wasm_hash = wasm.hash.clone();
                    MonitorStorage::save(id, mon);
                }
            }
        }).await;
    }
}