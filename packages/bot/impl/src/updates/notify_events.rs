use bot_api::updates::notify_events::NotifiyEventsArgs;
use oc_bots_sdk::{
    oc_api::actions::{send_message, ActionArgsBuilder}, 
    types::{ActionScope, BotApiKeyContext, BotPermissions, Chat, MessageContentInitial, TextContent}
};
use oc_bots_sdk_canister::OPENCHAT_CLIENT_FACTORY;
use crate::{guards::*, state};

#[ic_cdk::update]
pub async fn notify_events(
    args: NotifiyEventsArgs
) -> Result<(), String> {
    monitor_canister_only().await?;

    state::read(|s| {
        if let Some(api_key) = s.api_key_registry().get_key_with_required_permissions(
            &ActionScope::Chat(args.chat),
            &BotPermissions::text_only(),
        ).cloned() {
            ic_cdk::futures::spawn(
                send_message(api_key.to_context(), args.chat)
            )
        }
    });

    Ok(())
}

async fn send_message(
    ctx: BotApiKeyContext,
    chat: Chat
) {
    match OPENCHAT_CLIENT_FACTORY
        .build(ctx)
        .send_message(MessageContentInitial::Text(TextContent { text: "hey!".to_string() }))
        .with_channel_id(chat.channel_id())
        .with_block_level_markdown(true)
        .execute_async()
        .await
    {
        Ok(send_message::Response::Success(_)) => (),
        Err((code, message)) => {
            ic_cdk::println!("error: Failed to send event: code({}): message({})", code, message);
        }
        other => {
            ic_cdk::println!("error: Failed to send event {:?}", other);
        }
    }
}