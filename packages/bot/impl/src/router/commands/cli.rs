use std::{collections::HashSet, sync::LazyLock};
use async_trait::async_trait;
use candid::{Encode, Principal};
use clap::Parser;
use ic_cdk::api::management_canister::main::{
    CanisterInstallMode, CanisterSettings, 
    CreateCanisterArgument, InstallCodeArgument, LogVisibility
};
use monitor_api::{lifecycle::init::InitOrUpgradeArgs, updates::add_job::{AddJobArgs, AddJobResult}};
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
        monitor::{Monitor, MonitorId}
    }, DEPLOY_MONITOR_CYCLES
};

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(EventsMonCli::definition);

const MIN_INTERVAL: u32 = 15;
const MAX_INTERVAL: u32 = 60;

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
                                ).await
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

        let admin = state::read(|s| s.administrator().clone());

        let canister_id = ic_cdk::api::management_canister::main::create_canister(
            CreateCanisterArgument {
                settings: Some(CanisterSettings{
                    controllers: Some(vec![
                        ic_cdk::api::id(),
                        admin
                    ]),
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

        let (administrator, wasm) = state::read(|s| 
            (
                s.administrator().clone(),
                s.monitor_wasm().clone()
            )
        );
        let bot_canister_id = ic_cdk::id();

        ic_cdk::api::management_canister::main::install_code(InstallCodeArgument {
            mode: CanisterInstallMode::Install,
            canister_id,
            wasm_module: wasm.image,
            arg: Encode!(&InitOrUpgradeArgs { 
                administrator, 
                bot_canister_id,
            }).unwrap()
        }).await
            .map_err(|e| e.1)?;

        MonitorStorage::save(id, Monitor::new(chat, canister_id, wasm.hash));

        Ok(EphemeralMessageBuilder::new(
            MessageContentInitial::from_text(format!("Monitor deployed! Canister id: {}", canister_id)),
            client.context().message_id().unwrap(),
        ).with_block_level_markdown(true).build().into())
    }

    async fn create_canister_job(
        canister_id: String, 
        method_name: String, 
        output_template: String, 
        interval: u32,
        chat: Chat,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {

        let canister_id = Principal::from_text(canister_id).unwrap();

        let mut mon = if let Some(mon) = MonitorStorage::load(&chat.into()) {
            mon
        }
        else {
            return Err("No monitor deployed. Please use '/eventmon deploy' first".to_string());
        };

        if interval < MIN_INTERVAL {
            return Err(format!("Interval to low. Min: {}", MIN_INTERVAL));
        }
        else if interval > MAX_INTERVAL {
            return Err(format!("Interval to high. Max: {}", MAX_INTERVAL));
        }

        let job_id = ic_cdk::call::<(AddJobArgs, ), (AddJobResult, )>(
            mon.canister_id, 
            "add_job", 
            (AddJobArgs {
                canister_id,
                method_name,
                output_template,
                offset: 0,
                size: 8,
                interval,
            }, )
        ).await.map_err(|e| e.1)?.0?;

        mon.jobs.push(job_id);
        MonitorStorage::save(chat.into(), mon);

        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::from_text(format!("New job with id {} created!", job_id)),
                client.context().message_id().unwrap(),
            )
            .with_block_level_markdown(true)
            .build()
            .into()
        )
    }

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

