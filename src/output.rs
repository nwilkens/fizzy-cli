use chrono::{DateTime, Utc};
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_BORDERS_ONLY, Table, ContentArrangement};

use crate::models::*;

pub fn relative_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(*dt);

    let secs = diff.num_seconds();
    if secs < 0 {
        return "just now".to_string();
    }
    if secs < 60 {
        return format!("{secs}s ago");
    }
    let mins = diff.num_minutes();
    if mins < 60 {
        return format!("{mins}m ago");
    }
    let hours = diff.num_hours();
    if hours < 24 {
        return format!("{hours}h ago");
    }
    let days = diff.num_days();
    if days < 30 {
        return format!("{days}d ago");
    }
    let months = days / 30;
    if months < 12 {
        return format!("{months}mo ago");
    }
    let years = days / 365;
    format!("{years}y ago")
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

// --- Account listing ---

pub fn print_accounts(accounts: &[AccountWithUser]) {
    let mut table = new_table();
    table.set_header(vec!["NAME", "SLUG", "ROLE", "USER"]);
    for a in accounts {
        table.add_row(vec![
            &a.name,
            &a.slug,
            &a.user.role,
            &a.user.name,
        ]);
    }
    println!("{table}");
}

// --- Board listing ---

pub fn print_boards(boards: &[Board]) {
    let mut table = new_table();
    table.set_header(vec!["NAME", "ACCESS", "ENTROPY", "CREATED"]);
    for b in boards {
        let entropy = b
            .auto_postpone_period_in_days
            .map(|d| format!("{d}d"))
            .unwrap_or_else(|| "--".to_string());
        let access = if b.all_access { "all" } else { "selective" };
        table.add_row(vec![
            &b.name,
            access,
            &entropy,
            &relative_time(&b.created_at),
        ]);
    }
    println!("{table}");
}

pub fn print_board_detail(b: &Board) {
    println!("{}", b.name.bold());
    println!("{}", "─".repeat(40));
    println!("ID:       {}", b.id);
    println!("Access:   {}", if b.all_access { "all users" } else { "selective" });
    if let Some(d) = b.auto_postpone_period_in_days {
        println!("Entropy:  {d} days");
    }
    println!("Creator:  {}", b.creator.name);
    println!("Created:  {}", b.created_at);
    if let Some(ref url) = b.public_url {
        println!("Public:   {url}");
    }
    println!("URL:      {}", b.url);
}

// --- Card listing ---

pub fn print_cards(cards: &[Card]) {
    let mut table = new_table();
    table.set_header(vec!["#", "TITLE", "BOARD", "COLUMN", "TAGS", "CREATED"]);
    for c in cards {
        let col = c
            .column
            .as_ref()
            .map(|col| col.name.as_str())
            .unwrap_or("--");
        let tags = if c.tags.is_empty() {
            "--".to_string()
        } else {
            c.tags.iter().map(|t| format!("#{t}")).collect::<Vec<_>>().join(" ")
        };
        table.add_row(vec![
            &c.number.to_string(),
            &truncate(&c.title, 40),
            &c.board.name,
            col,
            &tags,
            &relative_time(&c.created_at),
        ]);
    }
    println!("{table}");
}

pub fn print_card_detail(c: &Card) {
    println!("{} {}", format!("#{}", c.number).bold(), c.title.bold());
    println!("{}", "━".repeat(50));
    println!("Board:    {}", c.board.name);
    if let Some(ref col) = c.column {
        println!("Column:   {} ({})", col.name, col.color);
    }
    println!("Status:   {}", c.status);
    if !c.tags.is_empty() {
        let tags: String = c.tags.iter().map(|t| format!("#{t}")).collect::<Vec<_>>().join(" ");
        println!("Tags:     {tags}");
    }
    if let Some(ref assignees) = c.assignees {
        if !assignees.is_empty() {
            let names: String = assignees.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ");
            println!("Assigned: {names}");
        }
    }
    println!("Creator:  {}", c.creator.name);
    if let Some(closed) = c.closed {
        if closed {
            println!("Closed:   {}", "yes".red());
        }
    }
    if let Some(postponed) = c.postponed {
        if postponed {
            println!("Postponed: {}", "yes".yellow());
        }
    }
    if c.golden {
        println!("Golden:   {}", "yes".yellow());
    }
    println!("Created:  {}", c.created_at);
    println!("Active:   {}", c.last_active_at);
    println!("URL:      {}", c.url);

    if !c.description.is_empty() {
        println!();
        println!("{}", c.description);
    }

    if let Some(ref steps) = c.steps {
        if !steps.is_empty() {
            println!();
            println!("{}", "Steps:".bold());
            for s in steps {
                let check = if s.completed { "[x]" } else { "[ ]" };
                println!("  {check} {} ({})", s.content, s.id.dimmed());
            }
        }
    }
}

// --- Column listing ---

pub fn print_columns(columns: &[Column]) {
    let mut table = new_table();
    table.set_header(vec!["ID", "NAME", "COLOR"]);
    for c in columns {
        table.add_row(vec![&c.id, &c.name, &c.color.to_string()]);
    }
    println!("{table}");
}

// --- User listing ---

pub fn print_users(users: &[User]) {
    let mut table = new_table();
    table.set_header(vec!["NAME", "ROLE", "EMAIL", "ACTIVE"]);
    for u in users {
        let active = if u.active { "yes" } else { "no" };
        let email = u.email_address.as_deref().unwrap_or("--");
        table.add_row(vec![u.name.as_str(), u.role.as_str(), email, active]);
    }
    println!("{table}");
}

pub fn print_user_detail(u: &User) {
    println!("{}", u.name.bold());
    println!("{}", "─".repeat(40));
    println!("ID:      {}", u.id);
    println!("Role:    {}", u.role);
    println!("Active:  {}", u.active);
    if let Some(ref email) = u.email_address {
        println!("Email:   {email}");
    }
    println!("Created: {}", u.created_at);
    println!("URL:     {}", u.url);
}

// --- Tag listing ---

pub fn print_tags(tags: &[Tag]) {
    let mut table = new_table();
    table.set_header(vec!["TITLE", "ID", "CREATED"]);
    for t in tags {
        table.add_row(vec![
            &format!("#{}", t.title),
            &t.id,
            &relative_time(&t.created_at),
        ]);
    }
    println!("{table}");
}

// --- Comment listing ---

pub fn print_comments(comments: &[Comment]) {
    for c in comments {
        println!(
            "{}  {}  {}",
            c.creator.name.bold(),
            "·".dimmed(),
            c.created_at.to_string().dimmed()
        );
        println!("  {}", c.body.plain_text);
        println!();
    }
    if comments.is_empty() {
        println!("No comments.");
    }
}

// --- Reaction listing ---

pub fn print_reactions(reactions: &[Reaction]) {
    if reactions.is_empty() {
        println!("No reactions.");
        return;
    }
    for r in reactions {
        println!("{} {} ({})", r.content, r.reacter.name, r.id.dimmed());
    }
}

// --- Notification listing ---

pub fn print_notifications(notifications: &[Notification]) {
    let mut table = new_table();
    table.set_header(vec!["READ", "TITLE", "BODY", "FROM", "CARD", "CREATED"]);
    for n in notifications {
        let read_marker = if n.read { " " } else { "*" };
        table.add_row(vec![
            read_marker,
            &truncate(&n.title, 30),
            &truncate(&n.body, 30),
            &n.creator.name,
            &n.card.title,
            &relative_time(&n.created_at),
        ]);
    }
    println!("{table}");
}

// --- Webhook listing ---

pub fn print_webhooks(webhooks: &[Webhook]) {
    let mut table = new_table();
    table.set_header(vec!["ID", "NAME", "URL", "ACTIVE", "ACTIONS"]);
    for w in webhooks {
        let actions = w.subscribed_actions.join(", ");
        table.add_row(vec![
            &w.id,
            &w.name,
            &truncate(&w.payload_url, 40),
            &(if w.active { "yes".to_string() } else { "no".to_string() }),
            &truncate(&actions, 30),
        ]);
    }
    println!("{table}");
}

pub fn print_webhook_detail(w: &Webhook) {
    println!("{}", w.name.bold());
    println!("{}", "─".repeat(40));
    println!("ID:      {}", w.id);
    println!("URL:     {}", w.payload_url);
    println!("Active:  {}", w.active);
    println!("Secret:  {}", w.signing_secret);
    println!("Actions: {}", w.subscribed_actions.join(", "));
    println!("Board:   {}", w.board.name);
    println!("Created: {}", w.created_at);
}

// --- Pins ---

pub fn print_pins(cards: &[Card]) {
    print_cards(cards);
}

// --- JSON output ---

pub fn print_json(value: &serde_json::Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
}

// --- Table helper ---

fn new_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}
