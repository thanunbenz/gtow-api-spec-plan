# GTO Wizard Preflop API — Next Actions

Lightweight navigation endpoint. Returns available actions at the next preflop decision node without strategy data.

## Endpoint

```
GET https://api.{DOMAIN}.com/v1/poker/next-actions/
```

Same v1 as the postflop next-actions endpoint.

## Authentication

| Header | Value | Description |
|--------|-------|-------------|
| `authorization` | `Bearer <JWT>` | Access token (short-lived, ~15 min) |
| `gwclientid` | UUID string | Client identifier |

---

## Query Parameters

| Parameter | Type | Required | Example | Description |
|-----------|------|----------|---------|-------------|
| `gametype` | string | Yes | `MTTGeneralV2` | Game format identifier |
| `depth` | string | Yes | `20.125` | Starting stack depth in big blinds |
| `stacks` | string | Yes | `20.125-20.125-...-20.125` | Stack sizes for all 9 seats, dash-separated |
| `preflop_actions` | string | Yes | `F-F-F-F-F-F-R2` | Preflop action sequence so far |

**Key difference from spot-solution:** The action parameters here represent the action just taken (e.g., `preflop_actions=F-F-F-F-F-F-R2` means "BTN just raised, what's next for SB?"), whereas spot-solution uses them to identify the current decision node.

---

## Response

### Top-Level Structure

```json
{
  "next_actions": {...},
  "future_actions": []
}
```

| Field | Type | Description |
|-------|------|-------------|
| `next_actions` | NextActionsNode | The immediate next decision point |
| `future_actions` | NextActionsNode[] | Always empty `[]` for preflop (no street transitions) |

**Unlike postflop**, `future_actions` is never populated for preflop nodes. Street-transition lookahead only applies when both players check and the game advances to the next street, which cannot happen preflop.

---

### NextActionsNode

Represents a decision point in the preflop game tree.

```json
{
  "game": {...},
  "available_actions": [...],
  "custom_solution_id": null,
  "is_node_locked": false,
  "is_edited": false,
  "is_editable": false,
  "forced_fold": false,
  "available_node_edits": null,
  "merged_actions": [],
  "preset_action_code": "F"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `game` | Game | Game state at this node (9 players) |
| `available_actions` | AvailableAction[] | Actions available at this node |
| `custom_solution_id` | string \| null | Custom solution override ID |
| `is_node_locked` | boolean | Whether this node is locked behind paywall |
| `is_edited` | boolean | Whether this node has been manually edited |
| `is_editable` | boolean | Whether this node can be edited by the user |
| `forced_fold` | boolean | Whether a fold is forced at this node |
| `available_node_edits` | any \| null | Available node edit options |
| `merged_actions` | string[] | List of action codes that have been merged |
| `preset_action_code` | string \| null | Default action hint for UI selection (`null` at terminal nodes) |

**Note:** The preflop `NextActionsNode` includes an `is_editable` field that is not present in the postflop spec.

---

### AvailableAction

Each available action at the node. Same structure as postflop.

```json
{
  "action": {
    "code": "F",
    "position": "BTN",
    "type": "FOLD",
    "betsize": "0.000",
    "allin": false,
    "is_hand_end": false,
    "is_showdown": false,
    "next_street": false,
    "display_name": "FOLD",
    "simple_group": "FOLD",
    "advanced_group": "FOLD",
    "betsize_by_pot": null,
    "next_position": "BB"
  },
  "frequency": null,
  "is_solution_end": false,
  "can_be_solved_by_ai": false,
  "next_position": "BB",
  "selected": false
}
```

#### ActionDef Fields

Same schema as spot-solution. See [PREFLOP_SOLUTION_API.md](PREFLOP_SOLUTION_API.md#actiondef-fields) for full reference.

#### Wrapper Fields

| Field | Type | Description |
|-------|------|-------------|
| `action` | ActionDef | Action metadata (same as spot-solution) |
| `frequency` | float \| null | Overall frequency (`null` — only spot-solution provides this) |
| `is_solution_end` | boolean | Whether taking this action reaches a terminal node |
| `can_be_solved_by_ai` | boolean | Whether AI solver is available for the resulting node |
| `next_position` | string | Position of the next player to act |
| `selected` | boolean | Whether this action is currently selected in the UI |

---

## Game Object

Same structure as spot-solution's `game` field. Contains all 9 seats, pot, and active position.

```json
{
  "players": [...],
  "current_street": {
    "type": "PREFLOP",
    "start_pot": "2.625",
    "end_pot": "2.625"
  },
  "pot": "2.625",
  "pot_odds": 0.276,
  "active_position": "BTN",
  "board": "",
  "bet_display_name": "RAISE"
}
```

See [PREFLOP_SOLUTION_API.md](PREFLOP_SOLUTION_API.md#game) for full field reference.

---

## Terminal Node (All Fold)

When all remaining players fold to a bet/raise, the response indicates a terminal state:

```json
{
  "next_actions": {
    "game": {
      "active_position": "LJ",
      "current_street": { "type": "PREFLOP" },
      "pot": "31.625",
      "board": ""
    },
    "available_actions": [],
    "preset_action_code": null
  },
  "future_actions": []
}
```

**Terminal indicators:**
- `available_actions` is an empty array
- `preset_action_code` is `null`
- `future_actions` is empty

---

## Preflop Navigation Flow

Full sequence of next-actions calls for a BTN open → SB 3bet → BTN 4bet all-in → SB folds line:

| Step | preflop_actions | Active | Actions Available | Terminal |
|------|------|--------|---|---|
| 1 | `F-F-F-F-F-F` | BTN | 3 (F, R2, RAI) | No |
| 2 | `F-F-F-F-F-F-R2` | SB | 3 (F, R6, RAI) | No |
| 3 | `F-F-F-F-F-F-R2-F` | BB | 4 (F, C, R6.5, RAI) | No |
| 4 | `F-F-F-F-F-F-R2-F-R6` | BTN | 3 (F, C, RAI) | No |
| 5 | `F-F-F-F-F-F-R2-F-R6-RAI` | SB | 2 (F, C) | No |
| 6 | `F-F-F-F-F-F-R2-F-R6-RAI-F` | — | **0** | **Yes** |

### Observations

- **Active position cycles** through remaining (non-folded) players
- **Action count varies**: more options when fewer players have acted
- **Pot grows** with each raise: 2.625 → 5.5 → 14 → 41.25
- **Terminal reached** when all but one player folds — hand ends without showdown
- **No board** is ever dealt in the preflop tree
- **`future_actions` is always empty** — no street transitions in preflop

---

## Preflop vs Postflop next-actions

| | Preflop | Postflop |
|---|---|---|
| **game.players** | 9 seats (full table) | 2 seats (heads-up) |
| **board** | `""` (empty) | 6–10 chars |
| **current_street.type** | `PREFLOP` | `FLOP`, `TURN`, `RIVER` |
| **future_actions** | Always `[]` | Populated on street transitions |
| **is_editable** | Present | Not present |
| **Terminal condition** | All fold to raise | Showdown (river check-check) or fold |
| **bet_display_name** | `"RAISE"` | `"BET"` |
