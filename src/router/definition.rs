use super::commands;
use oc_bots_sdk::api::definition::*;
use oc_bots_sdk_canister::{HttpRequest, HttpResponse};

pub async fn get(
    _request: HttpRequest
) -> HttpResponse {
    HttpResponse::json(
        200,
        &BotDefinition {
            description: "Events Monitor Bot posts real-time updates from any canister to your channel!"
                .to_string(),
            commands: commands::definitions(),
            autonomous_config: Some(AutonomousConfig { 
                permissions: BotPermissions::text_only(), 
                sync_api_key: true
            }),
        },
    )
}