
<!-- fizzyctl -->
## Task Workflow (fizzyctl)

Use `fizzyctl` to manage tasks from the Fizzy board.

### Finding work
- `fizzyctl prime` — see board context, your cards, and what's ready
- `fizzyctl ready` — list cards available for pickup
- `fizzyctl blocked` — list cards waiting on dependencies

### Working on a card
1. `fizzyctl claim <number>` — assign to self, move to In Progress (outputs task brief)
2. Enter plan mode (`/plan`) — design your implementation approach based on the task brief
3. Implement the plan, commit atomically
4. `fizzyctl progress <number> "message"` — log progress
5. `fizzyctl review <number>` — move to Review for human check, or
   `fizzyctl done <number>` — close the card

### Dependencies
- `fizzyctl dep <card> <depends-on>` — card depends on another (uses `#after-N` tags)
- `fizzyctl blocked` — show cards with unsatisfied dependencies
- Cards with `#after-N` tags won't show in `fizzyctl ready` until card N is closed
<!-- fizzyctl -->
