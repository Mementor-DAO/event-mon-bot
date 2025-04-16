use std::cell::RefCell;
use candid::Principal;
use oc_bots_sdk::ApiKeyRegistry;
use serde::{Deserialize, Serialize};
use super::{STATE_NOT_INITIALIZED, STATE_ALREADY_INITIALIZED};

#[derive(Serialize, Deserialize)]
pub struct MainState {
    oc_public_key: String,
    administrator: Principal,
    api_key_registry: ApiKeyRegistry,
}

thread_local! {
    static STATE: RefCell<Option<MainState>> = RefCell::default();
}

pub fn init(
    state: MainState
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
        F: FnOnce(&MainState) -> R {
    STATE.with_borrow(|s| 
        f(s.as_ref().expect(STATE_NOT_INITIALIZED))
    )
}

#[allow(unused)]
pub fn mutate<F, R>(
    f: F
) -> R 
    where 
        F: FnOnce(&mut MainState) -> R {
    STATE.with_borrow_mut(|s| 
        f(s.as_mut().expect(STATE_NOT_INITIALIZED))
    )
}

pub fn take(
) -> MainState {
    STATE.take().expect(STATE_NOT_INITIALIZED)
}

impl MainState {
    pub fn new(
        oc_public_key: String,
        administrator: Principal,
    ) -> Self {
        Self {
            oc_public_key,
            administrator,
            api_key_registry: ApiKeyRegistry::default(),
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
}