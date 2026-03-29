mod cli;
mod client;
mod commands;
mod config;
mod models;
mod output;
mod project;

use anyhow::Result;
use clap::Parser;

use cli::*;
use client::FizzyClient;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let json = cli.json;
    let account_override = cli.account;
    let url_override = cli.url;

    match cli.command {
        Commands::Login { token } => {
            commands::login::login(token, url_override.as_deref(), json).await?;
        }
        Commands::Logout => {
            commands::login::logout().await?;
        }
        Commands::Accounts => {
            let config = Config::load()?;
            commands::accounts::list(&config, url_override.as_deref(), json).await?;
        }
        Commands::Config => {
            let config = Config::load()?;
            println!("Config file: {}", Config::config_path().display());
            println!("Base URL:    {}", config.base_url());
            println!("Account:     {}", config.account().unwrap_or_else(|| "(not set)".to_string()));
            println!("Board:       {}", config.board.as_deref().unwrap_or("(not set)"));
            println!("Token:       {}", if config.token().is_some() { "(set)" } else { "(not set)" });
        }
        Commands::Set { key, value } => {
            let mut config = Config::load()?;
            match key.as_str() {
                "account" => {
                    config.account = Some(value.clone());
                    println!("Account set to: {value}");
                }
                "url" => {
                    config.base_url = Some(value.clone());
                    println!("Base URL set to: {value}");
                }
                "board" => {
                    config.board = Some(value.clone());
                    println!("Default board set to: {value}");
                }
                _ => {
                    anyhow::bail!("Unknown config key: {key}. Valid keys: account, url, board");
                }
            }
            config.save()?;
        }

        // --- Boards ---
        Commands::Boards => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::boards::list(&client, json).await?;
        }
        Commands::Board(cmd) => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            match cmd {
                BoardCommand::Show { id } => {
                    commands::boards::show(&client, &id, json).await?;
                }
                BoardCommand::Create { name, all_access, entropy } => {
                    commands::boards::create(&client, &name, all_access, entropy, json).await?;
                }
                BoardCommand::Update { id, name, all_access, entropy } => {
                    commands::boards::update(&client, &id, name, all_access, entropy).await?;
                }
                BoardCommand::Delete { id } => {
                    commands::boards::delete(&client, &id).await?;
                }
                BoardCommand::Publish { id } => {
                    commands::boards::publish(&client, &id, json).await?;
                }
                BoardCommand::Unpublish { id } => {
                    commands::boards::unpublish(&client, &id).await?;
                }
            }
        }

        // --- Cards ---
        Commands::Cards { board, column, assignee, tag, index, sort, search, all } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::cards::list(
                &client,
                board.as_deref(),
                column.as_deref(),
                assignee.as_deref(),
                tag.as_deref(),
                index.as_deref(),
                sort.as_deref(),
                search.as_deref(),
                all,
                json,
            )
            .await?;
        }
        Commands::Card(cmd) => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            match cmd {
                CardCommand::Show { number } => {
                    commands::cards::show(&client, number, json).await?;
                }
                CardCommand::Create { title, board, description, tags, draft } => {
                    commands::cards::create(
                        &client,
                        &board,
                        &title,
                        description.as_deref(),
                        tags.as_deref(),
                        draft,
                        json,
                    )
                    .await?;
                }
                CardCommand::Update { number, title, description } => {
                    commands::cards::update(&client, number, title, description, json).await?;
                }
                CardCommand::Delete { number } => {
                    commands::cards::delete(&client, number).await?;
                }
                CardCommand::Close { number } => {
                    commands::cards::close(&client, number).await?;
                }
                CardCommand::Reopen { number } => {
                    commands::cards::reopen(&client, number).await?;
                }
                CardCommand::Postpone { number } => {
                    commands::cards::postpone(&client, number).await?;
                }
                CardCommand::Triage { number, column } => {
                    commands::cards::triage(&client, number, &column).await?;
                }
                CardCommand::Untriage { number } => {
                    commands::cards::untriage(&client, number).await?;
                }
                CardCommand::Tag { number, tag } => {
                    commands::cards::tag(&client, number, &tag).await?;
                }
                CardCommand::Assign { number, user } => {
                    commands::cards::assign(&client, number, &user).await?;
                }
                CardCommand::Watch { number } => {
                    commands::cards::watch(&client, number).await?;
                }
                CardCommand::Unwatch { number } => {
                    commands::cards::unwatch(&client, number).await?;
                }
                CardCommand::Gold { number } => {
                    commands::cards::gold(&client, number).await?;
                }
                CardCommand::Ungold { number } => {
                    commands::cards::ungold(&client, number).await?;
                }
                CardCommand::Pin { number } => {
                    commands::cards::pin(&client, number).await?;
                }
                CardCommand::Unpin { number } => {
                    commands::cards::unpin(&client, number).await?;
                }
                CardCommand::Comment { number, body } => {
                    if let Some(body) = body {
                        commands::comments::create(&client, number, &body).await?;
                    } else {
                        commands::comments::list(&client, number, json).await?;
                    }
                }
                CardCommand::CommentUpdate { number, comment_id, body } => {
                    commands::comments::update(&client, number, &comment_id, &body).await?;
                }
                CardCommand::CommentDelete { number, comment_id } => {
                    commands::comments::delete(&client, number, &comment_id).await?;
                }
                CardCommand::React { number, content } => {
                    commands::reactions::create_card(&client, number, &content).await?;
                }
                CardCommand::Unreact { number, reaction_id } => {
                    commands::reactions::delete_card(&client, number, &reaction_id).await?;
                }
                CardCommand::Reactions { number } => {
                    commands::reactions::list_card(&client, number, json).await?;
                }
                CardCommand::StepAdd { number, content } => {
                    commands::steps::add(&client, number, &content).await?;
                }
                CardCommand::StepComplete { number, step_id } => {
                    commands::steps::complete(&client, number, &step_id).await?;
                }
                CardCommand::StepDelete { number, step_id } => {
                    commands::steps::delete(&client, number, &step_id).await?;
                }
                CardCommand::CommentReact { number, comment_id, content } => {
                    commands::reactions::create_comment(&client, number, &comment_id, &content).await?;
                }
                CardCommand::CommentUnreact { number, comment_id, reaction_id } => {
                    commands::reactions::delete_comment(&client, number, &comment_id, &reaction_id).await?;
                }
                CardCommand::CommentReactions { number, comment_id } => {
                    commands::reactions::list_comment(&client, number, &comment_id, json).await?;
                }
            }
        }

        // --- Columns ---
        Commands::Columns { board } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::columns::list(&client, &board, json).await?;
        }
        Commands::Column(cmd) => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            match cmd {
                ColumnCommand::Create { board, name, color } => {
                    commands::columns::create(&client, &board, &name, color).await?;
                }
                ColumnCommand::Update { board, column_id, name, color } => {
                    commands::columns::update(&client, &board, &column_id, name, color).await?;
                }
                ColumnCommand::Delete { board, column_id } => {
                    commands::columns::delete(&client, &board, &column_id).await?;
                }
            }
        }

        // --- Users ---
        Commands::Users => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::users::list(&client, json).await?;
        }
        Commands::User { id } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::users::show(&client, &id, json).await?;
        }

        // --- Tags ---
        Commands::Tags => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::tags::list(&client, json).await?;
        }

        // --- Agent workflow ---
        Commands::Init { name } => {
            let config = Config::load()?;
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::init::init(&client, &config, name.as_deref()).await?;
        }
        Commands::Whoami => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::agent::whoami(&client, json).await?;
        }
        Commands::Prime { board } => {
            let config = Config::load()?;
            let project = project::ProjectConfig::load_or_default();
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::agent::prime(&client, &config, &project, board.as_deref(), json).await?;
        }
        Commands::Ready { board } => {
            let config = Config::load()?;
            let project = project::ProjectConfig::load_or_default();
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::agent::ready(&client, &config, &project, board.as_deref(), json).await?;
        }
        Commands::Blocked { board } => {
            let config = Config::load()?;
            let project = project::ProjectConfig::load_or_default();
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::agent::blocked(&client, &config, &project, board.as_deref(), json).await?;
        }
        Commands::Dep { number, depends_on } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::agent::dep(&client, number, depends_on).await?;
        }
        Commands::Claim { number } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::agent::claim(&client, number).await?;
        }
        Commands::Progress { number, message } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::agent::progress(&client, number, &message).await?;
        }
        Commands::Done { number, message } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::agent::done(&client, number, message.as_deref()).await?;
        }
        Commands::Review { number, message } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::agent::review(&client, number, message.as_deref()).await?;
        }

        // --- Pins ---
        Commands::Pins => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::pins::list(&client, json).await?;
        }

        // --- Notifications ---
        Commands::Notifications { read_all } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            if read_all {
                commands::notifications::read_all(&client).await?;
            } else {
                commands::notifications::list(&client, json).await?;
            }
        }
        Commands::NotificationRead { id } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::notifications::mark_read(&client, &id).await?;
        }
        Commands::NotificationUnread { id } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::notifications::mark_unread(&client, &id).await?;
        }

        // --- Webhooks ---
        Commands::Webhooks { board } => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            commands::webhooks::list(&client, &board, json).await?;
        }
        Commands::Webhook(cmd) => {
            let client = make_client(account_override.as_deref(), url_override.as_deref())?;
            match cmd {
                WebhookCommand::Show { board, id } => {
                    commands::webhooks::show(&client, &board, &id, json).await?;
                }
                WebhookCommand::Create { board, name, payload_url, actions } => {
                    commands::webhooks::create(&client, &board, &name, &payload_url, &actions, json).await?;
                }
                WebhookCommand::Update { board, id, name, actions } => {
                    commands::webhooks::update(&client, &board, &id, name, actions).await?;
                }
                WebhookCommand::Delete { board, id } => {
                    commands::webhooks::delete(&client, &board, &id).await?;
                }
                WebhookCommand::Activate { board, id } => {
                    commands::webhooks::activate(&client, &board, &id).await?;
                }
            }
        }
    }

    Ok(())
}

fn make_client(account: Option<&str>, url: Option<&str>) -> Result<FizzyClient> {
    let config = Config::load()?;
    if account.is_none() {
        config.require_account()?;
    }
    FizzyClient::new(&config, account, url)
}
