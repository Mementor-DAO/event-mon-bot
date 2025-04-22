use std::{cell::RefCell, io::Write};
use candid::Principal;
use oc_bots_sdk::ApiKeyRegistry;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const STATE_ALREADY_INITIALIZED: &str = "State has already been initialized";
const STATE_NOT_INITIALIZED: &str = "State has not been initialized";

#[derive(Clone, Serialize, Deserialize)]
pub struct MonitorWasm {
    pub image: Vec<u8>,
    pub hash: Vec<u8>,
}

impl MonitorWasm {
    pub fn new(
        image: Vec<u8>
    ) -> Self {
        let mut hasher = Sha256::new();
        let _ = hasher.write(&image);
        let hash = hasher.finalize().to_vec();
        
        Self {
            image,
            hash,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    oc_public_key: String,
    administrator: Principal,
    api_key_registry: ApiKeyRegistry,
    monitor_wasm: MonitorWasm,
}

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

pub fn init(
    state: State
) {
    STATE.with_borrow_mut(|s| {
        if s.is_some() {
            panic!("{}", STATE_ALREADY_INITIALIZED);
        } else {
            *s = Some(state);
        }
    });
}

pub fn read<F, R>(
    f: F
) -> R 
    where 
        F: FnOnce(&State) -> R {
    STATE.with_borrow(|s| 
        f(s.as_ref().expect(STATE_NOT_INITIALIZED))
    )
}

#[allow(unused)]
pub fn mutate<F, R>(
    f: F
) -> R 
    where 
        F: FnOnce(&mut State) -> R {
    STATE.with_borrow_mut(|s| 
        f(s.as_mut().expect(STATE_NOT_INITIALIZED))
    )
}

pub fn take(
) -> State {
    STATE.take().expect(STATE_NOT_INITIALIZED)
}

impl State {
    pub fn new(
        administrator: Principal,
        oc_public_key: String,
        wasm: Vec<u8>
    ) -> Self {
        Self {
            oc_public_key,
            administrator,
            api_key_registry: ApiKeyRegistry::default(),
            monitor_wasm: MonitorWasm::new(wasm),
        }
    }

    pub fn oc_public_key(
        &self
    ) -> &str {
        &self.oc_public_key
    }
    
    pub fn set_oc_public_key(
        &mut self, 
        oc_public_key: String
    ) {
        self.oc_public_key = oc_public_key;
    }
    
    pub fn administrator(
        &self
    ) -> Principal {
        self.administrator
    }

    pub fn set_administrator(
        &mut self, 
        administrator: Principal
    ) {
        self.administrator = administrator;
    }
    
    pub fn api_key_registry(
        &self
    ) -> &ApiKeyRegistry {
        &self.api_key_registry
    }
    
    pub fn api_key_registry_mut(
        &mut self
    ) -> &mut ApiKeyRegistry {
        &mut self.api_key_registry
    }
    
    pub fn monitor_wasm(
        &self
    ) -> &MonitorWasm {
        &self.monitor_wasm
    }
    
    pub fn set_monitor_wasm(
        &mut self, 
        monitor_wasm: MonitorWasm
    ) {
        self.monitor_wasm = monitor_wasm;
    }
}