use std::cell::RefCell;
use candid::{Encode, Principal};
use ic_cdk::api::management_canister::main::{
    create_canister, install_code, start_canister, stop_canister, 
    CanisterIdRecord, CanisterInstallMode, CanisterSettings, 
    CanisterStatusType, CreateCanisterArgument, InstallCodeArgument, LogVisibility
};
use monitor_api::{
    lifecycle::init::InitOrUpgradeArgs, 
    queries::list_jobs::{Job, ListJobsArgs, ListJobsResult}, 
    updates::{
        add_job::{AddJobArgs, AddJobResult, JobId}, 
        del_job::{DelJobArgs, DelJobResult}, 
        start_job::{StartJobArgs, StartJobResult}, 
        stop_job::{StopJobArgs, StopJobResult}
    }
};
use oc_bots_sdk::types::Chat;
use crate::{
    consts::DEPLOY_CANISTER_CYCLES, 
    services::fund::{FundCanisterConfig, FundService}, 
    state::MonitorWasm, 
    storage::monitor::MonitorStorage, 
    types::monitor::{
        Monitor, MonitorId, MonitorState, MonitorStatus
    }, 
    utils::{
        ic::get_canister_status, 
        nat::nat_to_u128
    }
};

pub const MIN_MONITOR_CYCLES: u128 = 500_000_000_000;
const FUND_MONITOR_CYCLES: u128 =    100_000_000_000;
const MONITOR_CYCLES_CHECK_INTERVAL: u64 = 6 * 60 * 60; // every 6 hours

const MIN_INTERVAL: u32 = 60; // 60 seconds
const MAX_INTERVAL: u32 = 24 * 60 * 60; // 24 hours
const ITEMS_PER_PAGE: u32 = 16;

thread_local! {
    static FUND_SERVICE: RefCell<FundService> = RefCell::new(FundService::new());
}

pub struct MonitorService;

impl MonitorService {
    pub fn start(
    ) {
        // start fund service (to auto top-up the monitors deployed)
        let mut canisters = vec![];

        MonitorStorage::for_each_mut(&mut |_mon_id, mon| {
            // for each monitor, use its owner's subaccount to top-up the canister
            canisters.push(
                FundCanisterConfig {
                    canister_id: mon.canister_id.clone(),
                    from_subaccount: mon.owner.into(),
                    min_cycles: MIN_MONITOR_CYCLES,
                    fund_cycles: FUND_MONITOR_CYCLES,
                }
            );
        });

        FUND_SERVICE.with_borrow_mut(|service| {
            service.start(
                canisters,
                MONITOR_CYCLES_CHECK_INTERVAL
            );
        })
    }

    pub async fn deploy(
        chat: Chat,
        user_id: Principal,
        administrator: Principal,
        wasm: MonitorWasm
    ) -> Result<Principal, String> {
        let mon_id = chat.into();
        
        if let Some(mon) = MonitorStorage::load(&mon_id) {
            return Err(format!("Monitor already deployed. Canister id: {}", mon.canister_id))
        }
        
        let bot_canister_id = ic_cdk::id();

        // 1st: create canister
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
            DEPLOY_CANISTER_CYCLES as _
        ).await
            .map_err(|e| e.1)?
            .0.canister_id;

        // 2nd: deposit min cycles
        if let Err(err) = ic_cdk::api::management_canister::main::deposit_cycles(
            CanisterIdRecord { canister_id }, 
            MIN_MONITOR_CYCLES
        ).await {
            ic_cdk::println!("error: depositing cycles to canister {}: {}", canister_id.to_text(), err.1);
        };

        // 3rd: install code
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

        // 4th: auto top-up de canister from users's wallet
        FUND_SERVICE.with_borrow_mut(|service| {
            service.add_canister(FundCanisterConfig {
                canister_id,
                from_subaccount: user_id.into(),
                min_cycles: MIN_MONITOR_CYCLES,
                fund_cycles: FUND_MONITOR_CYCLES,
            });
        });

        MonitorStorage::save(
            mon_id, 
            Monitor::new(chat, user_id, canister_id, wasm.hash)
        );

        Ok(canister_id)
    }

    pub async fn add_canister_job(
        mon_id: MonitorId,
        canister_id: Principal,
        method_name: String,
        interval: u32,
        batch_size: u32,
        output_template: String,
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
                interval,
                batch_size,
                output_template,
                offset: 0,
            }, )
        ).await.map_err(|e| e.1)?.0?;

        mon.jobs.push(job_id);
        MonitorStorage::save(mon_id, mon);

        Ok(job_id)
    }

    pub async fn start_job(
        mon_id: MonitorId,
        job_id: JobId
    ) -> Result<(), String> {
        let mon = if let Some(mon) = MonitorStorage::load(&mon_id) {
            mon
        }
        else {
            return Err("Unknown monitor id".to_string());
        };

        ic_cdk::call::<(StartJobArgs, ), (StartJobResult, )>(
            mon.canister_id, 
            "start_job", 
            (StartJobArgs {
                job_id,
            },)
        ).await.map_err(|e| e.1)?.0?;

        Ok(())
    }

    pub async fn stop_job(
        mon_id: MonitorId,
        job_id: JobId
    ) -> Result<(), String> {
        let mon = if let Some(mon) = MonitorStorage::load(&mon_id) {
            mon
        }
        else {
            return Err("Unknown monitor id".to_string());
        };

        ic_cdk::call::<(StopJobArgs, ), (StopJobResult, )>(
            mon.canister_id, 
            "stop_job", 
            (StopJobArgs {
                job_id,
            },)
        ).await.map_err(|e| e.1)?.0?;

        Ok(())
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

    pub async fn get_status(
        mon_id: MonitorId
    ) -> Result<MonitorStatus, String> {
        let mon = if let Some(mon) = MonitorStorage::load(&mon_id) {
            mon
        }
        else {
            return Err("Unknown monitor id".to_string());
        };

        let status = get_canister_status(mon.canister_id)
            .await
            .map(|s| MonitorStatus {
                status: match s.status {
                    CanisterStatusType::Running => MonitorState::Running,
                    _ => MonitorState::Idle,
                },
                module_hash: hex::encode(s.module_hash.unwrap()),
                memory_size: nat_to_u128(s.memory_size),
                cycles: nat_to_u128(s.cycles),
                idle_cycles_burned_per_day: nat_to_u128(s.idle_cycles_burned_per_day),
            })?;

        Ok(status)
    }

    pub async fn start_all(
    ) {
        MonitorStorage::for_each_async(async |id, mut mon| {
            match mon.state {
                MonitorState::Idle => {
                    ic_cdk::println!("info: starting monitor({})", mon.canister_id.to_text());
                    if let Err(err) = start_canister(CanisterIdRecord {
                        canister_id: mon.canister_id.clone()
                    }).await {
                        ic_cdk::println!("error: starting monitor({}): {}", mon.canister_id.to_text(), err.1);
                    }
                
                    mon.state = MonitorState::Running;
                    MonitorStorage::save(id, mon);
                },
                _ => {}
            }
        }).await;
    }
    
    pub async fn stop_all(
    ) {
        MonitorStorage::for_each_async(async |id, mut mon| {
            match mon.state {
                MonitorState::Running => {
                    ic_cdk::println!("info: stopping monitor({})", mon.canister_id.to_text());
                    if let Err(err) = stop_canister(CanisterIdRecord {
                        canister_id: mon.canister_id.clone()
                    }).await {
                        ic_cdk::println!("error: stopping monitor({}): {}", mon.canister_id.to_text(), err.1);
                    }
                
                    mon.state = MonitorState::Idle;
                    MonitorStorage::save(id, mon);
                },
                _ => {}
            }
        }).await;
    }

    pub async fn update_all(
        administrator: Principal,
        wasm: MonitorWasm
    ) {
        let bot_canister_id = ic_cdk::api::id();
    
        MonitorStorage::for_each_async(async |id, mut mon| {
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