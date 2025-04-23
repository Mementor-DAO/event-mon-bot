use std::sync::Arc;
use candid::Principal;
use canfund::{
    api::{
        cmc::IcCyclesMintingCanister, ledger::IcLedgerCanister
    }, 
    manager::{
        options::{
            CyclesThreshold, FundManagerOptions, 
            FundStrategy, ObtainCyclesOptions
        }, 
        RegisterOpts
    }, 
    operations::obtain::MintCycles, 
    FundManager
};
use ic_ledger_types::{
    Subaccount, 
    MAINNET_CYCLES_MINTING_CANISTER_ID, 
    MAINNET_LEDGER_CANISTER_ID
};

#[derive(Clone)]
pub struct FundCanisterConfig {
    pub canister_id: Principal,
    pub from_subaccount: Subaccount,
    pub min_cycles: u128,
    pub fund_cycles: u128,
}

pub struct FundService {
    manager: FundManager,
}

impl FundService {
    pub fn new(
    ) -> Self {
        Self {
            manager: FundManager::new()
        }
    }
    pub fn start(
        &mut self,
        configs: Vec<FundCanisterConfig>,
        interval: u64
    ) {
        
        let fund_manager_options = FundManagerOptions::new()
            .with_interval_secs(interval);

        self.manager.with_options(fund_manager_options);

        for config in configs {
            self.manager.register(
                config.canister_id,
                Self::get_register_opts(config)
            );
        }

        self.manager.start();
    }

    pub fn add_canister(
        &mut self,
        config: FundCanisterConfig
    ) {
        self.manager.register(
            config.canister_id, 
            Self::get_register_opts(config)
        );
    }

    fn get_obtain_cycles_config(
        subaccount: Subaccount
    ) -> ObtainCyclesOptions {
        ObtainCyclesOptions {
            obtain_cycles: Arc::new(MintCycles {
                ledger: Arc::new(IcLedgerCanister::new(MAINNET_LEDGER_CANISTER_ID)),
                cmc: Arc::new(IcCyclesMintingCanister::new(
                    MAINNET_CYCLES_MINTING_CANISTER_ID,
                )),
                from_subaccount: subaccount,
            }),
        }
    }

    fn get_register_opts(
        config: FundCanisterConfig
    ) -> RegisterOpts {
        RegisterOpts::new()
            .with_obtain_cycles_options(
                Self::get_obtain_cycles_config(config.from_subaccount)
            )
            .with_strategy(FundStrategy::BelowThreshold(
                CyclesThreshold::new()
                    .with_min_cycles(config.min_cycles)
                    .with_fund_cycles(config.fund_cycles),
            ))
    }
}

