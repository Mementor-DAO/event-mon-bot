use std::{collections::HashSet, sync::LazyLock};
use async_trait::async_trait;
use candid::Principal;
use clap::Parser;
use ic_ledger_types::{AccountIdentifier, DEFAULT_FEE, DEFAULT_SUBACCOUNT};
use monitor_api::updates::add_job::JobId;
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
use oc_bots_sdk_canister::CanisterRuntime;
use crate::{
    consts::DEPLOY_MONITOR_CYCLES, 
    services::{
        monitor::MonitorService, 
        wallet::wallet::WalletService
    }, 
    state, 
    storage::user::UserStorage, 
    types::{
        cli::{Cli, Commands, CreateSubcommand, Job, Wallet}, 
        user::{UserId, UserTransaction}
    }, utils::cmc::Cmc
};

static DEFINITION: LazyLock<BotCommandDefinition> = LazyLock::new(EventsMonCli::definition);

const LOG_ITEMS_PER_PAGE: usize = 8;

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

        let user_id = Principal::from_text(
            ctx.command.initiator.to_string()
        ).unwrap();

        let res = match Cli::try_parse_from(args) {
            Ok(cli) => {
                match cli.command {
                    Commands::Deploy => {
                        Self::deploy_monitor(
                            user_id,
                            chat,
                            &client
                        ).await
                    },
                    Commands::Status => {
                        Self::monitor_status(
                            chat,
                            &client
                        ).await
                    },
                    Commands::Job (command) => {
                        match command {
                            Job::Create ( subcommand ) => {
                                match subcommand {
                                    CreateSubcommand::Canister { 
                                        canister_id, method_name, output_template, 
                                        batch_size, interval } => {
                                        Self::create_canister_job(
                                            canister_id, method_name, interval, 
                                            batch_size, output_template, chat, &client
                                        ).await
                                    }
                                }
                            },
                            Job::List { page } => {
                                Self::list_jobs(page.max(1) - 1, chat, &client)
                                    .await
                            },
                            Job::Start { id } => {
                                Self::start_job(id, chat, &client)
                                    .await
                            },
                            Job::Stop { id } => {
                                Self::stop_job(id, chat, &client)
                                    .await
                            },
                            Job::Delete { id } => {
                                Self::delete_job(id, chat, &client)
                                    .await
                            },
                        }
                    },
                    Commands::Wallet (command) => {
                        match command {
                            Wallet::Balance => {
                                Self::wallet_balance(user_id, &client)
                                    .await
                            },
                            Wallet::Address => {
                                Self::wallet_address(user_id, &client)
                                    .await
                            },
                            Wallet::Withdraw { to, amount } => {
                                Self::wallet_withdraw(user_id, to, amount, &client)
                                    .await
                            },
                            Wallet::Logs { page } => {
                                Self::wallet_logs(
                                    user_id,
                                    page.max(1) - 1,
                                    &client
                                )
                            },
                        }
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
        user_id: UserId,
        chat: Chat,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {
        let (administrator, wasm) = state::read(|s| 
            (
                s.administrator().clone(),
                s.monitor_wasm().clone()
            )
        );

        let cost = Cmc::cycles_to_icp(DEPLOY_MONITOR_CYCLES).await?;
        let balance = WalletService::balance_of(user_id).await? as u128;
        if balance < cost {
            let acc_id = WalletService::address_of(user_id);
            return Err(
                format!(
                    "Your EventMon wallet balance of **{:.8}** ICP is too low to cover the current monitor deployment cost of **{:.8}** ICP  \nPlease transfer enough ICP to this address: **{}**", 
                    (balance as f32) / 100000000.0,
                    (cost as f32) / 100000000.0,
                    acc_id
                )
            );
        }
        
        if let Err(err) = WalletService::transfer(
            user_id.into(), 
            AccountIdentifier::new(&ic_cdk::id(), &DEFAULT_SUBACCOUNT), 
            cost as _
        ).await {
            let err = format!(
                "Failed paying the deployment cost: {}.", 
                err
            );
            ic_cdk::println!("error: {}", err);
            return Err(err);
        };
        
        let canister_id = match MonitorService::deploy(
            chat, user_id, administrator, wasm).await {
            Ok(canister_id) => {
                canister_id
            },
            Err(err) => {
                ic_cdk::println!("error: monitor deployment failed: {}", err);

                if let Err(err) = WalletService::transfer(
                    None, 
                    AccountIdentifier::new(&ic_cdk::id(), &user_id.into()), 
                    cost as u64 + DEFAULT_FEE.e8s()
                ).await {
                    ic_cdk::println!(
                        "error: could not return deployment cost {} to user {}: {}", 
                        cost, 
                        user_id.to_text(), 
                        err
                    );
                };

                return Err(err);
            },
        };

        Ok(EphemeralMessageBuilder::new(
            MessageContentInitial::from_text(format!("Monitor deployed! Canister id: {}", canister_id)),
            client.context().message_id().unwrap(),
        ).with_block_level_markdown(true).build().into())
    }

    async fn create_canister_job(
        canister_id: String, 
        method_name: String, 
        interval: u32,
        batch_size: u32,
        output_template: String, 
        chat: Chat,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {

        let canister_id = Principal::from_text(canister_id).unwrap();

        let job_id = MonitorService::add_canister_job(
            chat.into(), canister_id, method_name, interval, batch_size, output_template
        ).await?;

        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::from_text(format!("New job with id {} created!", job_id)),
                client.context().message_id().unwrap(),
            )
            .build()
            .into()
        )
    }

    async fn start_job(
        job_id: JobId, 
        chat: Chat,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {
        MonitorService::start_job(chat.into(), job_id).await?;

        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::from_text(format!("Job {} started!", job_id)),
                client.context().message_id().unwrap(),
            )
            .build()
            .into()
        )
    }

    async fn stop_job(
        job_id: JobId, 
        chat: Chat,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {
        MonitorService::stop_job(chat.into(), job_id).await?;

        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::from_text(format!("Job {} stopped!", job_id)),
                client.context().message_id().unwrap(),
            )
            .build()
            .into()
        )
    }

    async fn delete_job(
        job_id: JobId, 
        chat: Chat,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {
        MonitorService::del_job(chat.into(), job_id).await?;

        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::from_text(format!("Job {} deleted!", job_id)),
                client.context().message_id().unwrap(),
            )
            .build()
            .into()
        )
    }

    async fn list_jobs(
        page: u32,
        chat: Chat,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {

        let list = MonitorService::list_jobs(
            chat.into(), page
        ).await?;

        let text = list.iter()
            .map(|j| format!(
                "**Job ({})**:  \n- interval: {}s  \n- state: {}  \n- type: {}  \n- template: ```{}```", 
                j.id, 
                j.interval, 
                j.state, 
                j.ty, 
                j.output_template
            ))
            .collect::<Vec<_>>()
            .join("  \n  \n---  \n");

        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::from_text(text),
                client.context().message_id().unwrap(),
            )
            .with_block_level_markdown(true)
            .build()
            .into()
        )
    }

    async fn monitor_status(
        chat: Chat,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {

        let status = MonitorService::get_status(
            chat.into()
        ).await?;

        let text = format!(
            "- state: {}  \n- module hash: {}  \n- memory size: {}  \n- cycles available: **{:3.8}**",
            status.status,
            status.module_hash,
            status.memory_size,
            (status.cycles as f32) / 100000000.0
        );

        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::from_text(text),
                client.context().message_id().unwrap(),
            )
            .with_block_level_markdown(true)
            .build()
            .into()
        )
    }

    async fn wallet_balance(
        user_id: Principal,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {

        let icps = WalletService::balance_of(
            user_id
        ).await?;

        let content = format!(
            "Balance:  \nICP: {:.8}  \n", 
            icps as f32 / 100000000.0
        );
        
        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::Text(content.into()), 
                client.context().message_id().unwrap()
            ).with_block_level_markdown(true)
            .build()
            .into()
        )
    }

    async fn wallet_address(
        user_id: Principal,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {
        let icp_acc_id = WalletService::address_of(
            user_id
        );

        let content = format!(
            "Address:  \nICP: {}  \n", 
            icp_acc_id
        );
        
        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::Text(content.into()), 
                client.context().message_id().unwrap()
            ).with_block_level_markdown(true)
            .build()
            .into()
        )
    }

    async fn wallet_withdraw(
        user_id: Principal,
        to: Option<String>,
        amount: f32,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {
        let (block_num, account_id) = WalletService::transfer_hex(
            user_id,
            to,
            (amount * 100000000.0) as u64
        ).await?;

        let content = format!(
            "Withdrawn of **{}** ICP to account id **{}** completed! At block index: **{}**", 
            amount, account_id, block_num
        );
        
        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::Text(content.into()), 
                client.context().message_id().unwrap()
            ).with_block_level_markdown(true)
            .build()
            .into()
        )
    }

    fn wallet_logs(
        user_id: Principal,
        page_num: usize,
        client: &Client<CanisterRuntime, BotCommandContext>
    ) -> Result<SuccessResult, String> {
        let user = UserStorage::load(&user_id);

        let logs = user.txs.iter()
            .skip(page_num * LOG_ITEMS_PER_PAGE)
            .take(LOG_ITEMS_PER_PAGE)
            .cloned()
            .map(|tx| match tx {
                UserTransaction::IcpWithdraw { amount, to, block_num, timestamp } => 
                    format!(
                        "Withdraw: amount({} ICP) to account_id({}) with block_num({}) at timestamp({})", 
                        amount as f32 / 100000000.0, to, block_num, timestamp
                    ),
            })
            .collect::<Vec<_>>()
            .join("  \n");

        let num_pages = (user.txs.len() + LOG_ITEMS_PER_PAGE-1) / LOG_ITEMS_PER_PAGE;
        let page_num = (1+page_num).min(num_pages);

        Ok(
            EphemeralMessageBuilder::new(
                MessageContentInitial::Text(
                    format!("{}  \n  \nPage {}/{}",
                        if logs.len() > 0 {
                            logs
                        } 
                        else {
                            "No transactions found".to_string()
                        },
                        page_num,
                        num_pages
                    ).into()
                ), 
                client.context().message_id().unwrap()
            ).with_block_level_markdown(true)
                .build()
                .into()
        )
    }

    fn definition(
    ) -> BotCommandDefinition {
        BotCommandDefinition {
            name: "eventmon".to_string(),
            description: Some("Event Monitor's command interface. Type -h for help".to_string()),
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

