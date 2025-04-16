use crate::{
    router::commands::cli::EventsMonCli, 
    states::{self, main::MainState, mon::MonState}
};

pub mod init;
pub mod post_upgrade;
pub mod pre_upgrade;

const READER_WRITER_BUFFER_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub(crate) fn setup(
    main_state: MainState,
    mon_state: MonState
) -> Result<(), String> {
    ic_wasi_polyfill::init(&[0u8; 32], &[]);

    states::main::init(main_state);
    states::mon::init(mon_state);

    EventsMonCli::setup();

    Ok(())
}
