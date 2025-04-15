use std::{cell::Cell, collections::{BTreeMap, BTreeSet, HashMap}, future::Future, time::Duration};
use oc_bots_sdk_canister::env;
use oc_bots_sdk::types::{ActionScope, BotApiKeyContext, BotPermissions, Chat, TimestampMillis};
use ic_cdk_timers::TimerId;
use crate::{state, types::scheduler::{JobId, JobType, Schedulable, Scheduler}};

// code adapted from https://github.com/open-chat-labs/open-chat-bots/blob/main/rs/canister/examples/reminder/src/model/reminders.rs

const MAX_JOBS: usize = 100_000;
const MAX_JOBS_PER_CHAT: usize = 100;

pub struct AddResult {
    pub chat_job_id: u8,
    pub next_due: bool,
}

thread_local! {
    static TIMER_ID: Cell<Option<TimerId>> = Cell::default();
}

impl<T> Scheduler<T>
    where T: Clone + Schedulable<T> {

    pub fn new(
    ) -> Self {
        Self {
            jobs: HashMap::new(),
            per_chat: HashMap::new(),
            ordered: BTreeSet::new(),
            next_id: 0,
        }
    }

    pub fn add(
        &mut self,
        mut job: T,
        chat: Chat,
        utc_now: TimestampMillis,
    ) -> Result<AddResult, String> {
        if self.jobs.len() >= MAX_JOBS {
            return Err("Too many jobs".to_string());
        }

        if let Some(per_chat) = self.per_chat.get(&chat) {
            if per_chat.len() >= MAX_JOBS_PER_CHAT {
                return Err("Too many jobs in this chat".to_string());
            }
        } else {
            self.per_chat.insert(chat, BTreeMap::new());
        }

        let timestamp = match &job.ty() {
            JobType::Recurring => {
                Self::next_job_time(&job, utc_now).unwrap()
            }
            JobType::Once => {
                utc_now + job.interval()
            },
        };

        // Determine the next global ID and chat ID
        let global_id = self.next_id;
        self.next_id += 1;
        let chat_job_id = self.get_next_available_chat_id(&chat);

        self.per_chat
            .get_mut(&chat)
            .unwrap()
            .insert(chat_job_id, global_id);

        job.set_chat_job_id(chat_job_id);

        self.jobs.insert(
            global_id,
            job,
        );

        // Insert the reminder into the ordered set
        self.ordered.insert((timestamp, global_id));

        // Check if this reminder is actually the next due reminder
        let next_due = self.peek().map(|(_, id)| id == global_id).unwrap();

        Ok(AddResult {
            chat_job_id,
            next_due,
        })
    }

    pub fn process<TF, JF, R>(
        &mut self,
        timer_cb: TF,
        job_cb: JF
    ) where 
        TF: 'static + FnOnce() -> (),
        JF: FnOnce(BotApiKeyContext, T) -> R + Copy,
        R: 'static + Future<Output = ()> {
        TIMER_ID.set(None);

        while let Some(job) = self.pop_next_due_job(env::now()) {
            if let Some(api_key) = state::read(|s| s.api_key_registry()
                .get_key_with_required_permissions(
                    &ActionScope::Chat(job.chat()),
                    &BotPermissions::text_only(),
                ).cloned()) {
                let job = job.clone();
                ic_cdk::spawn(job_cb(
                    api_key.to_context(),
                    job
                ));
            } else {
                continue;
            }
        }

        self.start_if_required(timer_cb);
    }

    pub fn start_if_required<F>(
        &mut self,
        timer_cb: F
    ) -> bool 
        where F: 'static + FnOnce() -> () {
        if TIMER_ID.get().is_none() {
            if let Some(next_job_due) = self.peek().map(|(timestamp, _)| timestamp) {
                let utc_now = env::now();
                let timer_id = ic_cdk_timers::set_timer(
                    Duration::from_millis(next_job_due.saturating_sub(utc_now)),
                    timer_cb,
                );
                TIMER_ID.set(Some(timer_id));
                return true;
            }
        }
    
        false
    }
    
    pub fn restart<F>(
        &mut self,
        timer_cb: F
    ) where F: 'static + FnOnce() -> () {
        if let Some(timer_id) = TIMER_ID.get() {
            ic_cdk_timers::clear_timer(timer_id);
            TIMER_ID.set(None);
        }
    
        self.start_if_required(timer_cb);
    }

    pub fn peek(
        &self
    ) -> Option<(TimestampMillis, JobId)> {
        self.ordered.iter().next().copied()
    }

    pub fn pop_next_due_job(
        &mut self, 
        utc_now: TimestampMillis
    ) -> Option<T> {
        let (timestamp, global_id) = self.peek()?;

        if timestamp > utc_now {
            return None;
        }

        self.ordered.pop_first();

        let job = self.jobs.get_mut(&global_id)?;

        let (job, repeating) = if let Some(next) =
            Self::next_job_time(&job, utc_now) {
            self.ordered.insert((next, global_id));

            (job.clone(), true)
        } 
        else {
            (self.jobs.remove(&global_id).unwrap(), false)
        };

        if !repeating {
            match self.delete_from_chat(&job.chat(), job.chat_job_id()) {
                Ok(_) => (),
                Err(error) => {
                    ic_cdk::println!(
                        "Failed to delete reminder from chat: {} {}",
                        job.chat().canister_id().to_string(),
                        error
                    );
                }
            }
        }

        Some(job)
    }

    pub fn delete(
        &mut self, 
        chat: &Chat, 
        chat_reminder_id: u8
    ) -> Result<T, String> {
        let global_id = self.delete_from_chat(chat, chat_reminder_id)?;

        self.jobs
            .remove(&global_id)
            .ok_or("Job not found".to_string())
    }

    pub fn list(
        &self, 
        chat: &Chat
    ) -> Vec<T> {
        self.per_chat
            .get(chat)
            .map(|chat_jobs| {
                chat_jobs
                    .iter()
                    .filter_map(|(_, global_id)| self.jobs.get(global_id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn count(
        &self
    ) -> usize {
        self.jobs.len()
    }

    pub fn chats_count(
        &self
    ) -> usize {
        self.per_chat.len()
    }

    fn delete_from_chat(
        &mut self, 
        chat: &Chat, 
        chat_reminder_id: u8
    ) -> Result<u64, String> {
        let chat_reminders = self
            .per_chat
            .get_mut(chat)
            .ok_or("Chat not found".to_string())?;

        let global_id = chat_reminders
            .remove(&chat_reminder_id)
            .ok_or("Reminder not found".to_string())?;

        if chat_reminders.is_empty() {
            self.per_chat.remove(chat);
        }

        Ok(global_id)
    }

    fn next_job_time(
        job: &T,
        utc_now: TimestampMillis,
    ) -> Option<TimestampMillis> {
        match job.ty() {
            JobType::Recurring => {
                Some(utc_now + job.interval())
            },
            JobType::Once => {
                None
            },
        }
    }

    fn get_next_available_chat_id(
        &self, 
        chat: &Chat
    ) -> u8 {
        let per_chat = self.per_chat.get(chat).unwrap();
        for i in 1..(MAX_JOBS_PER_CHAT + 1) as u8 {
            if !per_chat.contains_key(&i) {
                return i;
            }
        }
        unreachable!()
    }

}