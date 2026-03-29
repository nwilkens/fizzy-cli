# fz

A command-line interface for [Fizzy](https://fizzy.do) project management, built for both humans and AI agents.

## Install

```bash
cargo install --git https://github.com/nwilkens/fizzy-cli.git
```

Or build from source:

```bash
git clone https://github.com/nwilkens/fizzy-cli.git
cd fizzy-cli
cargo install --path .
```

## Setup

```bash
# Authenticate with your Fizzy account
fz login

# Initialize a project (creates board, .fz.toml, Claude Code hooks)
fz init --name "My Project"

# Or adopt an existing board
fz init --board existing-board-name
```

## Agent Workflow

`fz` is designed for AI coding agents. The `init` command sets up Claude Code hooks and documents the workflow in your `CLAUDE.md` so agents can manage tasks autonomously.

```bash
# See board context — your cards, what's ready, what's blocked
fz prime

# Pick up a card
fz claim 12

# Log progress
fz progress 12 "implemented auth module"

# Send to review or close
fz review 12
fz done 12
```

### Dependencies

Cards can depend on other cards using `#after-N` tags:

```bash
# Card 15 depends on card 12
fz dep 15 12

# See what's blocked and what's available
fz blocked
fz ready
```

Cards with unsatisfied dependencies won't appear in `fz ready`.

### Plans

Each card can have a plan stored as a comment with a `💡` reaction:

```bash
# View the plan for card 12
fz plan 12

# Set a plan
fz plan 12 "1. Add auth middleware\n2. Write tests\n3. Update docs"
```

## Card Management

```bash
# List cards (with filters)
fz cards --column "In Progress" --assignee me --tag bug
fz cards --search "auth" --sort newest

# Create a card
fz card create "Fix login timeout" -b my-board -d "Users report 30s hangs"

# Card operations
fz card show 12
fz card close 12
fz card reopen 12
fz card postpone 12
fz card assign 12 USER_ID
fz card tag 12 bug
fz card gold 12          # mark as priority
```

### Comments & Reactions

```bash
fz card comment 12 "Found the root cause"
fz card react 12 "👍"
```

### Steps (Checklists)

```bash
fz card step-add 12 "Write migration"
fz card step-complete 12 STEP_ID
```

## Board & Column Management

```bash
fz boards
fz board create "Sprint 4"
fz columns BOARD_ID
fz column create BOARD_ID "Blocked" --color Pink
```

## Webhooks

```bash
fz webhooks BOARD_ID
fz webhook create BOARD_ID --name "CI" --payload-url https://ci.example.com/hook --actions card_created,card_closed
```

## Other Commands

```bash
fz whoami              # current user info
fz users               # list account users
fz tags                # list tags
fz pins                # list pinned cards
fz notifications       # list notifications
fz config              # show current config
fz set account my-org  # set default account
```

## Configuration

### Global config

`~/.config/fizzy/config.toml`:

```toml
base_url = "https://app.fizzy.do"
account = "my-org"
token = "your-token"
board = "default-board-id"
```

### Environment variables

| Variable | Overrides |
|----------|-----------|
| `FIZZY_TOKEN` | `token` |
| `FIZZY_URL` | `base_url` |
| `FIZZY_ACCOUNT` | `account` |

### Project config

`fz init` creates `.fz.toml` in your project root:

```toml
board_id = "board-uuid"
account = "my-org"
```

## Global Flags

| Flag | Description |
|------|-------------|
| `--json` | Output raw JSON |
| `-a, --account` | Override account |
| `--url` | Override API URL |

## License

MIT
