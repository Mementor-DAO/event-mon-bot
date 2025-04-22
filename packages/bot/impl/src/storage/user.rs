use std::cell::RefCell;
use candid::Principal;
use ic_stable_structures::BTreeMap;
use crate::{
    memory::{get_users_memory, Memory}, 
    types::user::{User, UserId}
};

pub struct UserStorage;

thread_local! {
    static USERS: RefCell<BTreeMap<UserId, User, Memory>> = RefCell::new(
        BTreeMap::init(
            get_users_memory()
        )
    );
}

impl UserStorage {
    pub fn save(
        id: Principal,
        user: User
    ) {
        USERS.with_borrow_mut(|users| {
            users.insert(id, user)
        });
    }

    pub fn load(
        id: &UserId
    ) -> User {
        USERS.with_borrow(|users| {
            users.get(&id)
                .unwrap_or_default()
        })
    }
}