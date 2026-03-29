use anyhow::{anyhow, Result};

use crate::client::FizzyClient;
use crate::config::Config;
use crate::models::*;
use crate::output;

/// Get the current user's ID in the active account.
async fn my_user_id(client: &FizzyClient) -> Result<(String, String)> {
    let identity: IdentityResponse = client.get_global("/my/identity").await?;
    let slug = client.account_slug();
    let account = identity
        .accounts
        .iter()
        .find(|a| a.slug.trim_start_matches('/') == slug)
        .ok_or_else(|| anyhow!("No user found for account /{slug}"))?;
    Ok((account.user.id.clone(), account.user.name.clone()))
}

/// Find a column by name (case-insensitive) on a board.
async fn find_column(client: &FizzyClient, board_id: &str, name: &str) -> Result<Column> {
    let columns: Vec<Column> = client
        .get_list(&format!("/boards/{board_id}/columns"), true)
        .await?;
    columns
        .into_iter()
        .find(|c| c.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| anyhow!("Column \"{name}\" not found on this board. Available: use `fizzyctl columns <board_id>`"))
}

/// Resolve board ID from flag, config, or error.
fn resolve_board_id(board_flag: Option<&str>, config: &Config) -> Result<String> {
    board_flag
        .map(|s| s.to_string())
        .or_else(|| config.board.clone())
        .ok_or_else(|| {
            anyhow!("No board specified. Use --board <id> or `fizzyctl set board <id>`.")
        })
}

// --- whoami ---

pub async fn whoami(client: &FizzyClient, json: bool) -> Result<()> {
    if json {
        let raw = client.get_global_raw("/my/identity").await?;
        output::print_json(&raw);
    } else {
        let identity: IdentityResponse = client.get_global("/my/identity").await?;
        let slug = client.account_slug();
        for a in &identity.accounts {
            let marker = if a.slug.trim_start_matches('/') == slug {
                " (active)"
            } else {
                ""
            };
            println!(
                "{}{}: {} ({}) — user_id: {}",
                a.name, marker, a.user.name, a.user.role, a.user.id
            );
        }
    }
    Ok(())
}

// --- prime ---

pub async fn prime(
    client: &FizzyClient,
    config: &Config,
    board_flag: Option<&str>,
    json: bool,
) -> Result<()> {
    let board_id = resolve_board_id(board_flag, config)?;
    let (my_id, my_name) = my_user_id(client).await?;

    // Fetch board, columns, and cards in parallel
    let board: Board = client.get(&format!("/boards/{board_id}")).await?;
    let columns: Vec<Column> = client
        .get_list(&format!("/boards/{board_id}/columns"), true)
        .await?;

    // Fetch my cards (assigned to me, on this board)
    let my_cards: Vec<Card> = client
        .get_list(
            &format!("/cards?board_ids[]={board_id}&assignee_ids[]={my_id}"),
            true,
        )
        .await?;

    // Fetch cards awaiting triage (no column) and "To Do" cards
    let all_cards: Vec<Card> = client
        .get_list(&format!("/cards?board_ids[]={board_id}"), true)
        .await?;

    if json {
        let mut ctx = serde_json::Map::new();
        ctx.insert("board".into(), serde_json::json!({ "id": board.id, "name": board.name }));
        ctx.insert("user".into(), serde_json::json!({ "id": my_id, "name": my_name }));
        ctx.insert(
            "columns".into(),
            serde_json::json!(columns.iter().map(|c| &c.name).collect::<Vec<_>>()),
        );
        ctx.insert(
            "my_cards".into(),
            serde_json::json!(my_cards.iter().map(|c| card_summary(c)).collect::<Vec<_>>()),
        );

        let ready: Vec<_> = all_cards
            .iter()
            .filter(|c| is_ready(c))
            .map(card_summary)
            .collect();
        ctx.insert("ready".into(), serde_json::json!(ready));

        let in_triage: Vec<_> = all_cards
            .iter()
            .filter(|c| c.column.is_none() && c.closed != Some(true) && c.postponed != Some(true))
            .map(card_summary)
            .collect();
        ctx.insert("triage".into(), serde_json::json!(in_triage));

        output::print_json(&serde_json::Value::Object(ctx));
        return Ok(());
    }

    // Human/agent-readable compact output
    println!("# Fizzy Context");
    println!("Board: {} ({})", board.name, board.id);
    println!("You: {} ({})", my_name, my_id);
    println!(
        "Columns: {}",
        columns
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(" → ")
    );
    println!();

    // My active cards
    let active: Vec<&Card> = my_cards
        .iter()
        .filter(|c| c.closed != Some(true) && c.postponed != Some(true))
        .collect();
    if !active.is_empty() {
        println!("## Your active cards:");
        for c in &active {
            let col = c
                .column
                .as_ref()
                .map(|col| col.name.as_str())
                .unwrap_or("triage");
            let tags = format_tags(&c.tags);
            println!("  #{} {} [{}]{}", c.number, c.title, col, tags);
        }
        println!();
    }

    // Ready for pickup
    let ready: Vec<&Card> = all_cards.iter().filter(|c| is_ready(c)).collect();
    if !ready.is_empty() {
        println!("## Ready for pickup:");
        for c in &ready {
            let col = c
                .column
                .as_ref()
                .map(|col| col.name.as_str())
                .unwrap_or("triage");
            let tags = format_tags(&c.tags);
            println!("  #{} {} [{}]{}", c.number, c.title, col, tags);
        }
        println!();
    }

    // In triage (backlog)
    let triage: Vec<&Card> = all_cards
        .iter()
        .filter(|c| c.column.is_none() && c.closed != Some(true) && c.postponed != Some(true))
        .collect();
    if !triage.is_empty() {
        println!("## In triage (backlog):");
        for c in &triage {
            let tags = format_tags(&c.tags);
            println!("  #{} {}{}", c.number, c.title, tags);
        }
        println!();
    }

    println!("## Workflow:");
    println!("  1. `fizzyctl claim <number>` — assign to self, move to In Progress");
    println!("  2. Do the work, commit atomically");
    println!("  3. `fizzyctl progress <number> \"message\"` — log progress");
    println!("  4. `fizzyctl review <number>` — move to Review, or");
    println!("     `fizzyctl done <number>` — close the card");

    Ok(())
}

// --- ready ---

pub async fn ready(
    client: &FizzyClient,
    config: &Config,
    board_flag: Option<&str>,
    json: bool,
) -> Result<()> {
    let board_id = resolve_board_id(board_flag, config)?;

    let all_cards: Vec<Card> = client
        .get_list(&format!("/cards?board_ids[]={board_id}"), true)
        .await?;

    let ready_cards: Vec<&Card> = all_cards.iter().filter(|c| is_ready(c)).collect();

    if json {
        let summaries: Vec<_> = ready_cards.iter().map(|c| card_summary(c)).collect();
        output::print_json(&serde_json::json!(summaries));
    } else if ready_cards.is_empty() {
        println!("No cards ready for pickup.");
    } else {
        println!("Cards ready for pickup:");
        for c in &ready_cards {
            let col = c
                .column
                .as_ref()
                .map(|col| col.name.as_str())
                .unwrap_or("triage");
            let tags = format_tags(&c.tags);
            println!(
                "  #{} {} [{}]{}",
                c.number, c.title, col, tags
            );
        }
    }
    Ok(())
}

// --- claim ---

pub async fn claim(client: &FizzyClient, number: u64) -> Result<()> {
    // Get the card to find its board
    let card: Card = client.get(&format!("/cards/{number}")).await?;

    if card.closed == Some(true) {
        anyhow::bail!("Card #{number} is closed.");
    }

    let board_id = &card.board.id;
    let (my_id, my_name) = my_user_id(client).await?;

    // Check if already assigned to me
    let already_assigned = card
        .assignees
        .as_ref()
        .map(|a| a.iter().any(|u| u.id == my_id))
        .unwrap_or(false);

    // Assign to self if not already
    if !already_assigned {
        let body = AssignmentRequest {
            assignee_id: my_id,
        };
        client
            .post_raw(&format!("/cards/{number}/assignments"), &body)
            .await?;
    }

    // Move to "In Progress" column
    match find_column(client, board_id, "In Progress").await {
        Ok(col) => {
            let body = TriageRequest {
                column_id: col.id,
            };
            client
                .post_raw(&format!("/cards/{number}/triage"), &body)
                .await?;
        }
        Err(_) => {
            // No "In Progress" column — just assign, don't move
        }
    }

    println!("Claimed #{number} — assigned to {my_name}, moved to In Progress.");
    Ok(())
}

// --- progress ---

pub async fn progress(client: &FizzyClient, number: u64, message: &str) -> Result<()> {
    let body = CreateCommentRequest {
        comment: CreateCommentBody {
            body: message.to_string(),
        },
    };
    client
        .post_raw(&format!("/cards/{number}/comments"), &body)
        .await?;
    println!("Progress logged on #{number}.");
    Ok(())
}

// --- done ---

pub async fn done(client: &FizzyClient, number: u64, message: Option<&str>) -> Result<()> {
    if let Some(msg) = message {
        let body = CreateCommentRequest {
            comment: CreateCommentBody {
                body: msg.to_string(),
            },
        };
        client
            .post_raw(&format!("/cards/{number}/comments"), &body)
            .await?;
    }
    client
        .post_no_body(&format!("/cards/{number}/closure"))
        .await?;
    println!("Card #{number} done.");
    Ok(())
}

// --- review ---

pub async fn review(
    client: &FizzyClient,
    number: u64,
    message: Option<&str>,
) -> Result<()> {
    let card: Card = client.get(&format!("/cards/{number}")).await?;
    let board_id = &card.board.id;

    if let Some(msg) = message {
        let body = CreateCommentRequest {
            comment: CreateCommentBody {
                body: msg.to_string(),
            },
        };
        client
            .post_raw(&format!("/cards/{number}/comments"), &body)
            .await?;
    }

    // Move to "Review" column
    match find_column(client, board_id, "Review").await {
        Ok(col) => {
            let body = TriageRequest {
                column_id: col.id,
            };
            client
                .post_raw(&format!("/cards/{number}/triage"), &body)
                .await?;
            println!("Card #{number} moved to Review.");
        }
        Err(_) => {
            println!("Card #{number} comment added (no Review column found).");
        }
    }

    Ok(())
}

// --- helpers ---

/// A card is "ready" if it's in a "To Do"-like column or in triage, unassigned, and open.
fn is_ready(card: &Card) -> bool {
    if card.closed == Some(true) || card.postponed == Some(true) {
        return false;
    }
    let has_assignees = card
        .assignees
        .as_ref()
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    if has_assignees {
        return false;
    }
    // In "To Do" column or in triage (no column)
    match &card.column {
        Some(col) => col.name.eq_ignore_ascii_case("to do"),
        None => true, // triage = ready for pickup
    }
}

fn card_summary(card: &Card) -> serde_json::Value {
    serde_json::json!({
        "number": card.number,
        "title": card.title,
        "column": card.column.as_ref().map(|c| c.name.as_str()).unwrap_or("triage"),
        "tags": card.tags,
        "assignees": card.assignees.as_ref().map(|a| a.iter().map(|u| u.name.as_str()).collect::<Vec<_>>()).unwrap_or_default(),
    })
}

fn format_tags(tags: &[String]) -> String {
    if tags.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            tags.iter()
                .map(|t| format!("#{t}"))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}
