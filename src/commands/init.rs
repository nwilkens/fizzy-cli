use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::client::FizzyClient;
use crate::config::Config;
use crate::models::*;
use crate::project::ProjectConfig;

pub async fn init(client: &FizzyClient, config: &Config, name: Option<&str>) -> Result<()> {
    let cwd = std::env::current_dir()?;

    // Check if already initialized
    if cwd.join(".fizzyctl.toml").exists() {
        anyhow::bail!("Already initialized. Remove .fizzyctl.toml to reinitialize.");
    }

    // Determine board name from flag or directory name
    let board_name = name
        .map(|s| s.to_string())
        .or_else(|| {
            cwd.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
        .ok_or_else(|| anyhow!("Could not determine project name. Use --name."))?;

    println!("Initializing fizzyctl for: {board_name}");

    // 1. Create board
    let body = CreateBoardRequest {
        board: CreateBoardBody {
            name: board_name.clone(),
            all_access: Some(true),
            auto_postpone_period_in_days: None,
        },
    };
    // The create endpoint returns 201 with Location header but no body
    // We need to list boards to find the one we just created
    client.post_raw("/boards", &body).await?;

    // Find the board we just created
    let boards: Vec<Board> = client.get_list("/boards", true).await?;
    let board = boards
        .iter()
        .find(|b| b.name == board_name)
        .ok_or_else(|| anyhow!("Board created but not found in list"))?;

    let board_id = &board.id;
    println!("  Board created: {} ({})", board_name, board_id);

    // 2. Create columns: To Do, In Progress, Review
    for (col_name, color) in [
        ("To Do", "var(--color-card-default)"),
        ("In Progress", "var(--color-card-4)"),
        ("Review", "var(--color-card-3)"),
    ] {
        let col_body = CreateColumnRequest {
            column: CreateColumnBody {
                name: col_name.to_string(),
                color: Some(color.to_string()),
            },
        };
        client
            .post_raw(&format!("/boards/{board_id}/columns"), &col_body)
            .await?;
    }
    println!("  Columns: To Do → In Progress → Review");

    // 3. Write .fizzyctl.toml
    let project = ProjectConfig {
        board_id: Some(board_id.clone()),
        account: config.account(),
    };
    ProjectConfig::save(&cwd.join(".fizzyctl.toml"), &project)?;
    println!("  Config: .fizzyctl.toml");

    // 4. Write Claude Code hooks
    write_claude_hooks(&cwd)?;
    println!("  Hooks: .claude/settings.json");

    // 5. Append to CLAUDE.md
    write_claude_md(&cwd)?;
    println!("  Workflow: CLAUDE.md updated");

    println!();
    println!("Ready! Run `fizzyctl prime` to see your board.");

    Ok(())
}

fn write_claude_hooks(project_root: &Path) -> Result<()> {
    let claude_dir = project_root.join(".claude");
    fs::create_dir_all(&claude_dir)?;

    let settings_path = claude_dir.join("settings.json");

    // If settings.json exists, merge; otherwise create
    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Add hooks
    let hooks = settings
        .as_object_mut()
        .ok_or_else(|| anyhow!("settings.json is not a JSON object"))?
        .entry("hooks")
        .or_insert(serde_json::json!({}));

    let hooks_obj = hooks
        .as_object_mut()
        .ok_or_else(|| anyhow!("hooks is not a JSON object"))?;

    // SessionStart hook
    let session_start = hooks_obj
        .entry("SessionStart")
        .or_insert(serde_json::json!([]));
    let session_arr = session_start
        .as_array_mut()
        .ok_or_else(|| anyhow!("SessionStart is not an array"))?;

    // Check if fizzyctl prime already registered
    let already_has = session_arr.iter().any(|h| {
        h.get("command")
            .and_then(|c| c.as_str())
            .map(|s| s.contains("fizzyctl prime"))
            .unwrap_or(false)
    });

    if !already_has {
        session_arr.push(serde_json::json!({
            "type": "command",
            "command": "fizzyctl prime 2>/dev/null || true"
        }));
    }

    fs::write(
        &settings_path,
        serde_json::to_string_pretty(&settings)?,
    )?;

    Ok(())
}

fn write_claude_md(project_root: &Path) -> Result<()> {
    let claude_md = project_root.join("CLAUDE.md");

    let marker = "<!-- fizzyctl -->";
    let workflow_section = format!(
        r#"
{marker}
## Task Workflow (fizzyctl)

Use `fizzyctl` to manage tasks from the Fizzy board.

### Finding work
- `fizzyctl prime` — see board context, your cards, and what's ready
- `fizzyctl ready` — list cards available for pickup
- `fizzyctl blocked` — list cards waiting on dependencies

### Working on a card
1. `fizzyctl claim <number>` — assign to self, move to In Progress
2. Do the work, commit atomically
3. `fizzyctl progress <number> "message"` — log progress
4. `fizzyctl review <number>` — move to Review for human check, or
   `fizzyctl done <number>` — close the card

### Dependencies
- `fizzyctl dep <card> <depends-on>` — card depends on another (uses `#after-N` tags)
- `fizzyctl blocked` — show cards with unsatisfied dependencies
- Cards with `#after-N` tags won't show in `fizzyctl ready` until card N is closed
{marker}"#
    );

    if claude_md.exists() {
        let content = fs::read_to_string(&claude_md)?;
        if content.contains(marker) {
            // Already has our section — replace it
            let start = content.find(marker).unwrap();
            let end = content[start + marker.len()..]
                .find(marker)
                .map(|i| start + marker.len() + i + marker.len())
                .unwrap_or(content.len());
            let mut new_content = String::new();
            new_content.push_str(&content[..start]);
            new_content.push_str(&workflow_section);
            new_content.push_str(&content[end..]);
            fs::write(&claude_md, new_content)?;
        } else {
            // Append
            let mut content = content;
            content.push_str("\n");
            content.push_str(&workflow_section);
            content.push('\n');
            fs::write(&claude_md, content)?;
        }
    } else {
        fs::write(&claude_md, format!("{workflow_section}\n"))?;
    }

    Ok(())
}
