use crate::state::{self, State};

pub mod init;
pub mod post_upgrade;
pub mod pre_upgrade;

const READER_WRITER_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub(crate) fn setup(
    state: State
) -> Result<(), String> {
    state::init(state);

    Ok(())
}
