use std::cell::RefCell;
use candid::Principal;
use oc_bots_sdk::ApiKeyRegistry;
use serde::{Deserialize, Serialize};

use crate::types::{job::Job, scheduler::Scheduler};

thread_local! {
    static STATE: RefCell<Option<State>> = RefCell::default();
}

#[derive(Serialize, Deserialize)]
pub struct State {
    oc_public_key: String,
    administrator: Principal,
    api_key_registry: ApiKeyRegistry,
    scheduler: Scheduler<Job>
}

const STATE_ALREADY_INITIALIZED: &str = "State has already been initialized";
const STATE_NOT_INITIALIZED: &str = "State has not been initialized";

pub fn init(
    state: State
) {
    STATE.with_borrow_mut(|s| {
        if s.is_some() {
            panic!("{}", STATE_ALREADY_INITIALIZED);
        } else {
            *s = Some(state);
        }
    })
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
        oc_public_key: String,
        administrator: Principal,
    ) -> Self {
        Self {
            oc_public_key,
            administrator,
            api_key_registry: ApiKeyRegistry::default(),
            scheduler: Scheduler::new()
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
    
    pub fn scheduler_mut(
        &mut self
    ) -> &mut Scheduler<Job> {
        &mut self.scheduler
    }
}