use std::cell::RefCell;
use serde::{Deserialize, Serialize};
use crate::types::{mon::Mon, scheduler::Scheduler};
use super::{STATE_NOT_INITIALIZED, STATE_ALREADY_INITIALIZED};

#[derive(Serialize, Deserialize)]
pub struct MonState {
    scheduler: Scheduler<Mon>,
}

thread_local! {
    static STATE: RefCell<Option<MonState>> = RefCell::default();
}

pub fn init(
    state: MonState
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
        F: FnOnce(&MonState) -> R {
    STATE.with_borrow(|s| 
        f(s.as_ref().expect(STATE_NOT_INITIALIZED))
    )
}

#[allow(unused)]
pub fn mutate<F, R>(
    f: F
) -> R 
    where 
        F: FnOnce(&mut MonState) -> R {
    STATE.with_borrow_mut(|s| 
        f(s.as_mut().expect(STATE_NOT_INITIALIZED))
    )
}

pub fn take(
) -> MonState {
    STATE.take().expect(STATE_NOT_INITIALIZED)
}

impl MonState {
    pub fn new(
    ) -> Self {
        Self {
            scheduler: Scheduler::new(),
        }
    }
    
    pub fn scheduler(
        &self
    ) -> &Scheduler<Mon> {
        &self.scheduler
    }

    pub fn scheduler_mut(
        &mut self
    ) -> &mut Scheduler<Mon> {
        &mut self.scheduler
    }
}