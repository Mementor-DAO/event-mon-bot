use std::{collections::HashSet, sync::LazyLock};
use async_trait::async_trait;
use candid::{Encode, Principal};
use clap::Parser;
use ic_cdk::api::management_canister::main::{
    CanisterInstallMode, CanisterSettings, 
    CreateCanisterArgument, InstallCodeArgument, LogVisibility
};
use monitor_api::lifecycle::init::InitOrUpgradeArgs;
use oc_bots_sdk::{
    api::{
        command::{
            CommandHandler, EphemeralMessageBuilder, SuccessResult
        }, 
        definition::*
    }, 
    oc_api::
        client::Client
    , 
    types::{
        ActionContext, BotCommandContext, BotCommandScope, 
        Chat, ChatRole, MessageContentInitial
    }
};
use oc_bots_sdk_canister::
    CanisterRuntime
;
use crate::{
    state, 
    storage::monitor::MonitorStorage, 
    types::{
        cli::{Cli, Commands, CreateSubcommand}, 
        monitor::{Monitor, MonitorId, MonitorState}
    }, DEPLOY_MONITOR_CYCLES
};

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(EventsMonCli::definition);

pub struct EventsMonCli;

#[async_trait]
impl CommandHandler<CanisterRuntime> for EventsMonCli {
    fn definition(
        &self
    ) -> &BotCommandDefinition {
        &DEFINITION
    }

    async fn execute(
        &self,
        client: Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {
        let ctx = client.context();

        let args = shell_words::split(
            &format!("/eventmon {}", client.context().command.arg::<String>("args"))
        ).unwrap();

        let BotCommandScope::Chat(chat_scope) = &ctx.scope else {
            return Err("This command can only be used in a chat".to_string());
        };

        let chat = chat_scope.chat;

        state::read(|s| {
            if s.api_key_registry()
                .get_key_with_required_permissions(
                    &ctx.scope.clone().into(),
                    &BotPermissions::text_only(),
                ).is_none() {
                Err("You must first register an API key for this chat with the \"send text message\" permission".to_string())
            }
            else {
                Ok(())
            }
        })?;

        let _user_id = Principal::from_text(
            ctx.command.initiator.to_string()
        ).unwrap();

        let res = match Cli::try_parse_from(args) {
            Ok(cli) => {
                match cli.command {
                    Commands::Deploy => {
                        Self::deploy_monitor(
                            chat,
                            &client
                        ).await
                    },
                    Commands::Create ( subcommand ) => {
                        match subcommand {
                            CreateSubcommand::Canister { canister_id, method_name, output_template, interval } => {
                                Self::create_canister_job(
                                    canister_id, method_name, output_template, interval, chat, &client
                                )
                            }
                        }
                    },
                    Commands::List { .. } => {
                        todo!()
                    },
                    Commands::Start { .. } => {
                        todo!()
                    },
                    Commands::Stop { .. } => {
                        todo!()
                    },
                    Commands::Delete { .. } => {
                        todo!()
                    },
                }
            },
            Err(err) => {
                Err(match err.kind() {
                    clap::error::ErrorKind::DisplayVersion => {
                        err.to_string()
                    },
                    _ => {
                        ansi_to_html::Converter::new()
                            .convert(
                                &err.render()
                                    .ansi()
                                    .to_string()
                            ).unwrap()
                    }
                })
            },
        };

        match res {
            Ok(success_res) => {
                Ok(success_res)
            },
            Err(text) => {
                Ok(EphemeralMessageBuilder::new(
                    MessageContentInitial::from_text(text),
                    ctx.message_id().unwrap(),
                ).with_block_level_markdown(true).build().into())
            }
        }
    }
}

impl EventsMonCli {
    async fn deploy_monitor(
        chat: Chat,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {
        let id = MonitorId::from(chat);
        if let Some(mon) = MonitorStorage::load(&id) {
            return Err(format!("Monitor already deployed. Canister id: {}", mon.canister_id))
        }

        let canister_id = ic_cdk::api::management_canister::main::create_canister(
            CreateCanisterArgument {
                settings: Some(CanisterSettings{
                    controllers: Some(vec![ic_cdk::api::id()]),
                    compute_allocation: None,
                    memory_allocation: None,
                    freezing_threshold: None,
                    reserved_cycles_limit: None,
                    log_visibility: Some(LogVisibility::Public),
                    wasm_memory_limit: None,
                }),
            }, 
            DEPLOY_MONITOR_CYCLES
        ).await
            .map_err(|e| e.1)?
            .0.canister_id;

        let (administrator, wasm_module) = state::read(|s| 
            (
                s.administrator().clone(),
                s.monitor_wasm().image.clone()
            )
        );
        let bot_canister_id = ic_cdk::id();

        ic_cdk::api::management_canister::main::install_code(InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            wasm_module,
            arg: Encode!(&InitOrUpgradeArgs { 
                administrator, 
                bot_canister_id,
            }).unwrap()
        }).await
            .map_err(|e| e.1)?;

        MonitorStorage::save(id, Monitor{
            chat,
            state: MonitorState::Running,
            canister_id,
        });

        Ok(EphemeralMessageBuilder::new(
            MessageContentInitial::from_text(format!("Monitor deployed! Canister id: {}", canister_id)),
            client.context().message_id().unwrap(),
        ).with_block_level_markdown(true).build().into())
    }

    fn create_canister_job(
        canister_id: String, 
        _method_name: String, 
        _output_template: String, 
        _interval: u32,
        _chat: Chat,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {

        let _canister_id = Principal::from_text(canister_id).unwrap();

        let job_id = 0; /*states::mon::mutate(|s| {
            let job = Mon::new_canister(
                canister_id, 
                method_name, 
                output_template, 
                interval, 
                chat
            );
    
            let scheduler = s.scheduler_mut();
            match scheduler.add(job, chat, env::now()) {
                Ok(res) => {
                    if res.next_due {
                        scheduler.start_if_required(Self::timer_cb);
                    }
                    else {
                        scheduler.restart(Self::timer_cb);
                    }
        
                    Ok(res.chat_job_id)
                },
                Err(err) => {
                    Err(err)
                },
            }
        })?;*/

        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::from_text(format!("New job created with id {}", job_id)),
                client.context().message_id().unwrap(),
            )
            .with_block_level_markdown(true)
            .build()
            .into()
        )
    }

    /*async fn mon_cb(
        ctx: BotApiKeyContext,
        mon: Mon
    ) {
        match OPENCHAT_CLIENT_FACTORY
            .build(ctx)
            .send_message(MessageContentInitial::Text(TextContent { text: "hey!".to_string() }))
            .with_channel_id(mon.chat.channel_id())
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

    fn timer_cb(
    ) {
        states::mon::mutate(|mon_state| {
            state::read(|main_state| {
                mon_state.scheduler_mut().process(
                    main_state.api_key_registry(),
                    Self::timer_cb,
                    Self::mon_cb
                );
            })
        })
    }*/

    fn definition(
    ) -> BotCommandDefinition {
        BotCommandDefinition {
            name: "eventmon".to_string(),
            description: Some("Events Monitor Bot's command interface. Type -h for help".to_string()),
            placeholder: Some("Please wait...".to_string()),
            params: vec![
                BotCommandParam {
                    name: "args".to_string(),
                    description: Some("Arguments separated by white-spaces (leave empty for help)".to_string()),
                    placeholder: Some("Enter the arguments".to_string()), 
                    required: false,
                    param_type: BotCommandParamType::StringParam(StringParam{
                        choices: vec![],
                        min_length: 0,
                        max_length: 1024,
                        multi_line: true,
                    }),
                },
            ],
            permissions: BotPermissions::default().with_message(&HashSet::from([
                MessagePermission::Text,
            ])),
            default_role: Some(ChatRole::Admin),
            direct_messages: Some(true),
        }
    }
}

