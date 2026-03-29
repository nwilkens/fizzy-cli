use anyhow::{anyhow, Result};

use crate::client::FizzyClient;
use crate::config::Config;
use crate::models::*;
use crate::output;
use crate::project::ProjectConfig;

const DEP_PREFIX: &str = "after-";
const PLAN_PREFIX: &str = "## Plan";
const PLAN_REACTION: &str = "💡";

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
        .ok_or_else(|| {
            anyhow!("Column \"{name}\" not found. Use `fizzyctl columns <board_id>`")
        })
}

/// Resolve board ID: flag > project config > global config.
pub fn resolve_board_id(
    board_flag: Option<&str>,
    project: &ProjectConfig,
    config: &Config,
) -> Result<String> {
    ProjectConfig::resolve_board(board_flag, project, config)
}

/// Parse #after-N tags and return the dependency card numbers.
fn parse_deps(tags: &[String]) -> Vec<u64> {
    tags.iter()
        .filter_map(|t| t.strip_prefix(DEP_PREFIX).and_then(|n| n.parse::<u64>().ok()))
        .collect()
}

/// Check if all dependencies are satisfied.
/// Cards not in the open list are assumed closed (the default list only returns open cards).
fn deps_satisfied(card: &Card, open_cards: &[Card]) -> bool {
    let deps = parse_deps(&card.tags);
    if deps.is_empty() {
        return true;
    }
    for dep_num in deps {
        // If the dependency card is in the open cards list, it's NOT closed → not satisfied
        if open_cards.iter().any(|c| c.number == dep_num) {
            return false;
        }
        // If not in the list, it's either closed or doesn't exist → treat as satisfied
    }
    true
}

/// Get unsatisfied dependencies for a card.
fn unsatisfied_deps(card: &Card, open_cards: &[Card]) -> Vec<u64> {
    parse_deps(&card.tags)
        .into_iter()
        .filter(|dep_num| {
            // If found in open cards, it's not closed → still blocking
            open_cards.iter().any(|c| c.number == *dep_num)
        })
        .collect()
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
    project: &ProjectConfig,
    board_flag: Option<&str>,
    json: bool,
) -> Result<()> {
    let board_id = resolve_board_id(board_flag, project, config)?;
    let (my_id, my_name) = my_user_id(client).await?;

    let board: Board = client.get(&format!("/boards/{board_id}")).await?;
    let columns: Vec<Column> = client
        .get_list(&format!("/boards/{board_id}/columns"), true)
        .await?;

    let my_cards: Vec<Card> = client
        .get_list(
            &format!("/cards?board_ids[]={board_id}&assignee_ids[]={my_id}"),
            true,
        )
        .await?;

    let all_cards: Vec<Card> = client
        .get_list(&format!("/cards?board_ids[]={board_id}"), true)
        .await?;

    if json {
        let mut ctx = serde_json::Map::new();
        ctx.insert(
            "board".into(),
            serde_json::json!({ "id": board.id, "name": board.name }),
        );
        ctx.insert(
            "user".into(),
            serde_json::json!({ "id": my_id, "name": my_name }),
        );
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
            .filter(|c| is_ready(c, &all_cards))
            .map(card_summary)
            .collect();
        ctx.insert("ready".into(), serde_json::json!(ready));

        let blocked: Vec<_> = all_cards
            .iter()
            .filter(|c| is_open(c) && !deps_satisfied(c, &all_cards) && !parse_deps(&c.tags).is_empty())
            .map(|c| {
                let mut s = card_summary(c);
                s.as_object_mut().unwrap().insert(
                    "blocked_by".into(),
                    serde_json::json!(unsatisfied_deps(c, &all_cards)),
                );
                s
            })
            .collect();
        ctx.insert("blocked".into(), serde_json::json!(blocked));

        output::print_json(&serde_json::Value::Object(ctx));
        return Ok(());
    }

    // Compact text output
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
        .filter(|c| is_open(c))
        .collect();
    if !active.is_empty() {
        println!("## Your active cards:");
        for c in &active {
            let col = col_name(c);
            let tags = format_tags_without_deps(&c.tags);
            let deps = format_deps(&c.tags);
            println!("  #{} {} [{}]{}{}", c.number, c.title, col, tags, deps);
        }
        println!();
    }

    // Ready for pickup (dependency-aware)
    let ready: Vec<&Card> = all_cards.iter().filter(|c| is_ready(c, &all_cards)).collect();
    if !ready.is_empty() {
        println!("## Ready for pickup:");
        for c in &ready {
            let col = col_name(c);
            let tags = format_tags_without_deps(&c.tags);
            println!("  #{} {} [{}]{}", c.number, c.title, col, tags);
        }
        println!();
    }

    // Blocked cards
    let blocked: Vec<&Card> = all_cards
        .iter()
        .filter(|c| is_open(c) && !deps_satisfied(c, &all_cards) && !parse_deps(&c.tags).is_empty())
        .collect();
    if !blocked.is_empty() {
        println!("## Blocked:");
        for c in &blocked {
            let waiting: Vec<u64> = unsatisfied_deps(c, &all_cards);
            let waiting_str: Vec<String> = waiting.iter().map(|n| format!("#{n}")).collect();
            println!(
                "  #{} {} — waiting on {}",
                c.number,
                c.title,
                waiting_str.join(", ")
            );
        }
        println!();
    }

    println!("## Workflow:");
    println!("  fizzyctl claim <n>       — assign to self, move to In Progress");
    println!("  fizzyctl progress <n> .. — log progress comment");
    println!("  fizzyctl review <n>      — move to Review");
    println!("  fizzyctl done <n>        — close the card");
    println!("  fizzyctl dep <n> <dep>   — add dependency (#after-N tag)");

    Ok(())
}

// --- ready ---

pub async fn ready(
    client: &FizzyClient,
    config: &Config,
    project: &ProjectConfig,
    board_flag: Option<&str>,
    json: bool,
) -> Result<()> {
    let board_id = resolve_board_id(board_flag, project, config)?;

    let all_cards: Vec<Card> = client
        .get_list(&format!("/cards?board_ids[]={board_id}"), true)
        .await?;

    let ready_cards: Vec<&Card> = all_cards
        .iter()
        .filter(|c| is_ready(c, &all_cards))
        .collect();

    if json {
        let summaries: Vec<_> = ready_cards.iter().map(|c| card_summary(c)).collect();
        output::print_json(&serde_json::json!(summaries));
    } else if ready_cards.is_empty() {
        println!("No cards ready for pickup.");
    } else {
        println!("Cards ready for pickup:");
        for c in &ready_cards {
            let col = col_name(c);
            let tags = format_tags_without_deps(&c.tags);
            println!("  #{} {} [{}]{}", c.number, c.title, col, tags);
        }
    }
    Ok(())
}

// --- blocked ---

pub async fn blocked(
    client: &FizzyClient,
    config: &Config,
    project: &ProjectConfig,
    board_flag: Option<&str>,
    json: bool,
) -> Result<()> {
    let board_id = resolve_board_id(board_flag, project, config)?;

    let all_cards: Vec<Card> = client
        .get_list(&format!("/cards?board_ids[]={board_id}"), true)
        .await?;

    let blocked_cards: Vec<&Card> = all_cards
        .iter()
        .filter(|c| {
            is_open(c) && !parse_deps(&c.tags).is_empty() && !deps_satisfied(c, &all_cards)
        })
        .collect();

    if json {
        let summaries: Vec<_> = blocked_cards
            .iter()
            .map(|c| {
                let mut s = card_summary(c);
                s.as_object_mut().unwrap().insert(
                    "blocked_by".into(),
                    serde_json::json!(unsatisfied_deps(c, &all_cards)),
                );
                s
            })
            .collect();
        output::print_json(&serde_json::json!(summaries));
    } else if blocked_cards.is_empty() {
        println!("No blocked cards.");
    } else {
        println!("Blocked cards:");
        for c in &blocked_cards {
            let waiting: Vec<u64> = unsatisfied_deps(c, &all_cards);
            let waiting_str: Vec<String> = waiting.iter().map(|n| format!("#{n}")).collect();
            println!(
                "  #{} {} — blocked by {}",
                c.number,
                c.title,
                waiting_str.join(", ")
            );
        }
    }
    Ok(())
}

// --- dep ---

pub async fn dep(client: &FizzyClient, number: u64, depends_on: u64) -> Result<()> {
    let tag_title = format!("{DEP_PREFIX}{depends_on}");
    let body = TaggingRequest {
        tag_title: tag_title.clone(),
    };
    client
        .post_raw(&format!("/cards/{number}/taggings"), &body)
        .await?;
    println!("#{number} now depends on #{depends_on} (tag: #{tag_title})");
    Ok(())
}

// --- plan ---

/// Check if a card's description contains a ## Plan section.
fn has_plan(description: &str) -> bool {
    description.contains(PLAN_PREFIX)
}

/// Extract the plan section from a card description.
fn extract_plan(description: &str) -> Option<&str> {
    let start = description.find(PLAN_PREFIX)?;
    Some(description[start..].trim())
}

pub async fn plan(client: &FizzyClient, number: u64, text: Option<&str>) -> Result<()> {
    let card: Card = client.get(&format!("/cards/{number}")).await?;

    if let Some(plan_text) = text {
        // Append ## Plan to description (or replace existing plan section)
        let new_desc = if has_plan(&card.description) {
            // Replace everything from ## Plan onward
            let before = card.description.split(PLAN_PREFIX).next().unwrap_or("");
            format!("{}\n{PLAN_PREFIX}\n{plan_text}", before.trim_end())
        } else if card.description.is_empty() {
            format!("{PLAN_PREFIX}\n{plan_text}")
        } else {
            format!("{}\n\n{PLAN_PREFIX}\n{plan_text}", card.description)
        };

        let body = UpdateCardRequest {
            card: UpdateCardBody {
                title: None,
                description: Some(new_desc),
                status: None,
                tag_ids: None,
            },
        };
        client.put(&format!("/cards/{number}"), &body).await?;

        // Add 💡 reaction on the card to mark it as having a plan
        let reaction_body = CreateReactionRequest {
            reaction: CreateReactionBody {
                content: PLAN_REACTION.to_string(),
            },
        };
        let _ = client
            .post_raw(&format!("/cards/{number}/reactions"), &reaction_body)
            .await;

        println!("Plan set on #{number} 💡");
    } else {
        // Show the plan
        if let Some(plan) = extract_plan(&card.description) {
            println!("{plan}");
        } else {
            println!("No plan on #{number}. Set one with: fizzyctl plan {number} \"plan text\"");
        }
    }
    Ok(())
}

// --- claim ---

pub async fn claim(client: &FizzyClient, number: u64) -> Result<()> {
    let card: Card = client.get(&format!("/cards/{number}")).await?;

    if card.closed == Some(true) {
        anyhow::bail!("Card #{number} is closed.");
    }

    let board_id = &card.board.id;
    let (my_id, my_name) = my_user_id(client).await?;

    let already_assigned = card
        .assignees
        .as_ref()
        .map(|a| a.iter().any(|u| u.id == my_id))
        .unwrap_or(false);

    if !already_assigned {
        let body = AssignmentRequest {
            assignee_id: my_id,
        };
        client
            .post_raw(&format!("/cards/{number}/assignments"), &body)
            .await?;
    }

    match find_column(client, board_id, "In Progress").await {
        Ok(col) => {
            let body = TriageRequest {
                column_id: col.id,
            };
            client
                .post_raw(&format!("/cards/{number}/triage"), &body)
                .await?;
        }
        Err(_) => {}
    }

    // Output task brief
    println!("Claimed #{number} — assigned to {my_name}, moved to In Progress.");
    println!();
    print_task_brief(client, &card, number).await?;

    Ok(())
}

/// Print a rich task brief for agent context.
async fn print_task_brief(client: &FizzyClient, card: &Card, number: u64) -> Result<()> {
    println!("---");
    println!("# Task #{}: {}", card.number, card.title);
    println!();
    if !card.description.is_empty() {
        println!("{}", card.description);
        println!();
    }
    if !card.tags.is_empty() {
        let tags = format_tags_without_deps(&card.tags);
        let deps = parse_deps(&card.tags);
        if !tags.is_empty() {
            println!("Tags:{tags}");
        }
        if !deps.is_empty() {
            let dep_strs: Vec<String> = deps.iter().map(|n| format!("#{n}")).collect();
            println!("Depends on: {} (completed)", dep_strs.join(", "));
        }
        println!();
    }
    if let Some(ref steps) = card.steps {
        if !steps.is_empty() {
            println!("Steps:");
            for s in steps {
                let check = if s.completed { "[x]" } else { "[ ]" };
                println!("  {check} {}", s.content);
            }
            println!();
        }
    }

    // Show recent non-system comments
    let comments: Vec<Comment> = client
        .get_list(&format!("/cards/{number}/comments"), true)
        .await
        .unwrap_or_default();
    let user_comments: Vec<&Comment> = comments
        .iter()
        .filter(|c| c.creator.name != "System")
        .collect();
    if !user_comments.is_empty() {
        let show = if user_comments.len() > 5 {
            &user_comments[user_comments.len() - 5..]
        } else {
            &user_comments
        };
        println!("Recent comments:");
        for c in show {
            println!("  {} — {}", c.creator.name, c.body.plain_text);
        }
        println!();
    }

    if has_plan(&card.description) {
        println!("Plan exists in description. Implement it, then:");
    } else {
        println!("No plan yet. Enter plan mode (`/plan`) to design your approach, then:");
    }
    println!("  `fizzyctl progress {number} \"message\"` → `fizzyctl done {number}`");

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

pub async fn review(client: &FizzyClient, number: u64, message: Option<&str>) -> Result<()> {
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

fn is_open(card: &Card) -> bool {
    card.closed != Some(true) && card.postponed != Some(true)
}

/// A card is "ready" if it's open, unassigned, in To Do or triage, AND all deps are satisfied.
fn is_ready(card: &Card, all_cards: &[Card]) -> bool {
    if !is_open(card) {
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
    // Must be in "To Do" column or in triage (no column)
    let in_right_column = match &card.column {
        Some(col) => col.name.eq_ignore_ascii_case("to do"),
        None => true,
    };
    if !in_right_column {
        return false;
    }
    // All dependencies must be satisfied
    deps_satisfied(card, all_cards)
}

fn col_name(card: &Card) -> &str {
    card.column
        .as_ref()
        .map(|c| c.name.as_str())
        .unwrap_or("triage")
}

fn card_summary(card: &Card) -> serde_json::Value {
    serde_json::json!({
        "number": card.number,
        "title": card.title,
        "column": col_name(card),
        "tags": card.tags,
        "deps": parse_deps(&card.tags),
        "assignees": card.assignees.as_ref().map(|a| a.iter().map(|u| u.name.as_str()).collect::<Vec<_>>()).unwrap_or_default(),
    })
}

/// Format tags, excluding #after-N dependency tags.
fn format_tags_without_deps(tags: &[String]) -> String {
    let visible: Vec<&String> = tags
        .iter()
        .filter(|t| !t.starts_with(DEP_PREFIX))
        .collect();
    if visible.is_empty() {
        String::new()
    } else {
        format!(
            " {}",
            visible
                .iter()
                .map(|t| format!("#{t}"))
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

/// Format dependency tags as readable text.
fn format_deps(tags: &[String]) -> String {
    let deps = parse_deps(tags);
    if deps.is_empty() {
        String::new()
    } else {
        let dep_strs: Vec<String> = deps.iter().map(|n| format!("#{n}")).collect();
        format!(" (after {})", dep_strs.join(", "))
    }
}
