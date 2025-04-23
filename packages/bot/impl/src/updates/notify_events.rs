use bot_api::{updates::notify_events::{NotifiyEventsArgs, NotifiyEventsResponse}, NOTIFY_EVENT_COST};
use ic_cdk::api::call::{msg_cycles_accept128, msg_cycles_available};
use oc_bots_sdk::{
    oc_api::actions::{send_message, ActionArgsBuilder}, 
    types::{
        ActionScope, BotApiKeyContext, BotPermissions, 
        Chat, MessageContentInitial, TextContent
    }
};
use oc_bots_sdk_canister::OPENCHAT_CLIENT_FACTORY;
use crate::{guards::*, state, storage::monitor::MonitorStorage};

#[ic_cdk::update(guard = "monitor_canister_only")]
pub async fn notify_events(
    args: NotifiyEventsArgs
) -> NotifiyEventsResponse {
    if msg_cycles_available() < NOTIFY_EVENT_COST {
        let err = format!(
            "Not enough cycles sent to cover the costs. From monitor {}: {} < {}", 
            ic_cdk::caller().to_text(), msg_cycles_available(), NOTIFY_EVENT_COST
        );
        ic_cdk::println!("error: {}", err);
        return Err(err);
    }

    msg_cycles_accept128(NOTIFY_EVENT_COST as _);

    let mon = MonitorStorage::load_by_canister_id(&ic_cdk::caller()).unwrap();

    state::read(|s| {
        if let Some(api_key) = s.api_key_registry().get_key_with_required_permissions(
            &ActionScope::Chat(mon.chat),
            &BotPermissions::text_only(),
        ).cloned() {
            ic_cdk::spawn(
                send_messages(
                    api_key.to_context(), 
                    mon.chat, 
                    args.messages
                )
            )
        }
    });

    Ok(())
}

async fn send_messages(
    ctx: BotApiKeyContext,
    chat: Chat,
    messages: Vec<String>
) {
    if messages.len() > 0 {
        let text = messages.join("\n  ---\n  ").replace("\\n", "\n");
        
        match OPENCHAT_CLIENT_FACTORY
            .build(ctx)
            .send_message(MessageContentInitial::Text(TextContent { text }))
            .with_channel_id(chat.channel_id())
            .with_block_level_markdown(true)
            .execute_async()
            .await
        {
            Ok(send_message::Response::Success(_)) => (),
            Err((code, message)) => {
                ic_cdk::println!("error: Failed to send events: code({}): message({})", code, message);
            }
            other => {
                ic_cdk::println!("error: Failed to send events {:?}", other);
            }
        }
    }
}