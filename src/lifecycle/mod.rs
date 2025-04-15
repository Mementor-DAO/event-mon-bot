use crate::{
    router::commands::cli::EventsMonCli, 
    state::{self, State}
};

pub mod init;
pub mod post_upgrade;
pub mod pre_upgrade;

const READER_WRITER_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub(crate) fn setup(
    s: State
) -> Result<(), String> {
    ic_wasi_polyfill::init(&[0u8; 32], &[]);

    state::init(s);

    EventsMonCli::setup();

    Ok(())
}
