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

## Quick Start

### 1. Create an API key

Log in to [app.fizzy.do](https://app.fizzy.do), go to your account settings, and create a personal access token.

### 2. Authenticate

Pass your token to `fz` using any of these methods:

```bash
# Option A: store in config (recommended)
fz login --token YOUR_TOKEN

# Option B: environment variable
export FIZZY_TOKEN=YOUR_TOKEN
```

`fz login` without `--token` will start an interactive magic-link flow via email.

### 3. Verify

```bash
fz whoami       # confirm your identity
fz accounts     # list available accounts
fz config       # show current configuration
```

If you have multiple accounts, set a default:

```bash
fz set account my-org
```

### 4. Initialize a project

Run this in your project directory to create a board and wire up agent hooks:

```bash
fz init --name "My Project"
```

Or adopt an existing board:

```bash
fz init --board existing-board-name
```

This creates `.fz.toml` in your project root and, if you use Claude Code, adds a session hook to `fz prime` on startup and appends workflow docs to `CLAUDE.md`.

## Usage

### Listing and filtering cards

```bash
fz cards                                    # all cards on the default board
fz cards --column "In Progress"             # filter by column
fz cards --assignee me --tag bug            # filter by assignee and tag
fz cards --search "auth" --sort newest      # search with sorting
```

### Working with cards

```bash
fz card show 12                             # view card details
fz card create "Fix timeout" -b my-board    # create a card
fz card create "Add caching" -b my-board -d "Description here" --tags perf
fz card close 12                            # close a card
fz card reopen 12                           # reopen a closed card
fz card postpone 12                         # move to Not Now
fz card assign 12 USER_ID                   # toggle assignment
fz card tag 12 bug                          # toggle a tag
fz card gold 12                             # mark as priority
```

### Comments, reactions, and checklists

```bash
fz card comment 12                          # list comments
fz card comment 12 "Found the root cause"   # add a comment
fz card react 12 "👍"                       # add a reaction
fz card step-add 12 "Write migration"       # add a checklist item
fz card step-complete 12 STEP_ID            # check it off
```

### Boards and columns

```bash
fz boards                                   # list boards
fz board create "Sprint 4"                  # create a board
fz columns BOARD_ID                         # list columns
fz column create BOARD_ID "Blocked" --color Pink
```

### Webhooks

```bash
fz webhooks BOARD_ID
fz webhook create BOARD_ID --name "CI" \
  --payload-url https://ci.example.com/hook \
  --actions card_created,card_closed
```

### Other commands

```bash
fz users                # list account users
fz tags                 # list tags
fz pins                 # pinned cards
fz notifications        # list notifications
```

## Agent Workflow

`fz` includes commands designed for AI coding agents (e.g., Claude Code). The `fz init` command sets up hooks and documentation so agents can pick up, track, and close tasks autonomously.

### Lifecycle

```bash
fz prime                # board context: your cards, ready cards, blocked cards
fz ready                # cards available for pickup (respects dependencies)
fz claim 12             # assign to self, move to In Progress
fz progress 12 "msg"    # log a progress comment
fz review 12            # move to Review column
fz done 12              # close the card
```

### Dependencies

Cards can depend on other cards. Dependencies use `#after-N` tags — a card tagged `#after-12` won't appear in `fz ready` until card 12 is closed.

```bash
fz dep 15 12            # card 15 depends on card 12
fz blocked              # show cards with unsatisfied dependencies
```

### Plans

Each card can have a plan stored as a comment marked with a `💡` reaction:

```bash
fz plan 12              # view the plan
fz plan 12 "1. Add auth middleware\n2. Write tests"   # set a plan
```

## Configuration

### Global config

Stored at `~/.config/fizzy/config.toml`:

```toml
token = "your-api-key"
account = "my-org"
base_url = "https://app.fizzy.do"   # optional, this is the default
board = "default-board-id"          # optional, used by agent commands
```

The config file is created with `0600` permissions to protect your token.

### Environment variables

Environment variables take precedence over the config file:

| Variable | Overrides |
|----------|-----------|
| `FIZZY_TOKEN` | `token` |
| `FIZZY_ACCOUNT` | `account` |
| `FIZZY_URL` | `base_url` |

### Project config

`fz init` creates `.fz.toml` in your project root:

```toml
board_id = "board-uuid"
account = "my-org"
```

### Global flags

These flags work with any command:

| Flag | Description |
|------|-------------|
| `--json` | Output raw JSON instead of formatted tables |
| `-a, --account SLUG` | Override the default account |
| `--url URL` | Override the API base URL |

## License

MIT
