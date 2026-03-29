
<!-- fz -->
## Task Workflow (fz)

Use `fz` to manage tasks from the Fizzy board.

### Finding work
- `fz prime` — see board context, your cards, and what's ready
- `fz ready` — list cards available for pickup
- `fz blocked` — list cards waiting on dependencies

### Working on a card
1. `fz claim <number>` — assign to self, move to In Progress (outputs task brief)
2. Enter plan mode (`/plan`) — design your implementation approach based on the task brief
3. Implement the plan, commit atomically
4. `fz progress <number> "message"` — log progress
5. `fz review <number>` — move to Review for human check, or
   `fz done <number>` — close the card

### Dependencies
- `fz dep <card> <depends-on>` — card depends on another (uses `#after-N` tags)
- `fz blocked` — show cards with unsatisfied dependencies
- Cards with `#after-N` tags won't show in `fz ready` until card N is closed
<!-- fz -->
