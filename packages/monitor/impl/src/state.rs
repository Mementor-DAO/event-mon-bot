use std::cell::RefCell;
use candid::Principal;
use serde::{Deserialize, Serialize};
use crate::types::{active_job::ActiveJob, scheduler::Scheduler};

const STATE_ALREADY_INITIALIZED: &str = "State has already been initialized";
const STATE_NOT_INITIALIZED: &str = "State has not been initialized";

#[derive(Serialize, Deserialize)]
pub struct State {
    administrator: Principal,
    bot_canister_id: Principal,
    scheduler: Scheduler<ActiveJob>
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
        bot_canister_id: Principal,
    ) -> Self {
        Self {
            administrator,
            bot_canister_id,
            scheduler: Scheduler::new()
        }
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
    
    pub fn bot_canister_id(
        &self
    ) -> &Principal {
        &self.bot_canister_id
    }
    
    pub fn set_bot_canister_id(
        &mut self, 
        bot_canister_id: Principal
    ) {
        self.bot_canister_id = bot_canister_id;
    }

    pub fn scheduler(
        &self
    ) -> &Scheduler<ActiveJob> {
        &self.scheduler
    }

    pub fn scheduler_mut(
        &mut self
    ) -> &mut Scheduler<ActiveJob> {
        &mut self.scheduler
    }
}