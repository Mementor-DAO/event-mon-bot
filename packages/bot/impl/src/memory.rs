use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl,
};

const UPGRADES: MemoryId            = MemoryId::new(0);
const MONITORS: MemoryId            = MemoryId::new(1);
const CAN_TO_MON_ID: MemoryId       = MemoryId::new(2);
const USERS: MemoryId               = MemoryId::new(3);

pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    static MEMORY_MANAGER: MemoryManager<DefaultMemoryImpl>
        = MemoryManager::init_with_bucket_size(DefaultMemoryImpl::default(), 128);
}

fn get_memory(id: MemoryId) -> Memory {
    MEMORY_MANAGER.with(|m| m.get(id))
}

pub fn get_upgrades_memory() -> Memory {
    get_memory(UPGRADES)
}

pub fn get_monitors_memory() -> Memory {
    get_memory(MONITORS)
}

pub fn get_can_to_mon_id_memory() -> Memory {
    get_memory(CAN_TO_MON_ID)
}

pub fn get_users_memory() -> Memory {
    get_memory(USERS)
}
