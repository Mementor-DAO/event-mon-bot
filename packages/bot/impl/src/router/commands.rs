use cli::EventsMonCli;
use oc_bots_sdk::api::{
    command::CommandHandlerRegistry, 
    definition::BotCommandDefinition
};
use oc_bots_sdk_canister::{
    env::now, http_command_handler, CanisterRuntime, 
    HttpRequest, HttpResponse, OPENCHAT_CLIENT_FACTORY
};
use std::sync::LazyLock;
use crate::state;

mod cli;
mod sync_api_key;

static COMMANDS: LazyLock<CommandHandlerRegistry<CanisterRuntime>> = LazyLock::new(|| {
    CommandHandlerRegistry::new(OPENCHAT_CLIENT_FACTORY.clone())
        .register(EventsMonCli)
        .on_sync_api_key(Box::new(sync_api_key::callback))
});

pub fn definitions(
) -> Vec<BotCommandDefinition> {
    COMMANDS.definitions()
}

pub async fn execute(
    request: HttpRequest
) -> HttpResponse {
    let public_key = state::read(|s| s.oc_public_key().to_string());
    let now = now();

    http_command_handler::execute(request, &COMMANDS, &public_key, now).await
}