# GTO Wizard Preflop API — Spot Solution

## Endpoint

```
GET https://api.{DOMAIN}.com/v4/solutions/spot-solution/
```

Note: The preflop API uses **v4**, while the postflop API uses v1.

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
| `preflop_actions` | string | No | `F-F` | Preflop action sequence so far (empty = UTG to act) |

### Action Notation

Actions are separated by `-`:

| Code | Meaning | Example |
|------|---------|---------|
| `F` | Fold | `F` |
| `C` | Call | `C` |
| `R{size}` | Raise (size in BB) | `R2`, `R6` |
| `RAI` | All-in | `RAI` |

Note: There is no `X` (Check) or `B{frac}` (Bet) preflop. The first aggression preflop is always a "raise" over the big blind.

### Preflop Action Sequence

The `preflop_actions` field builds up incrementally as each seat acts. Seats act in order: UTG, UTG+1, UTG+2, LJ, HJ, CO, BTN, SB, BB.

| preflop_actions | Active Player | Situation |
|---|---|---|
| _(empty)_ | UTG | UTG first to act |
| `F` | UTG+1 | After UTG folds |
| `F-F` | UTG+2 | After UTG, UTG+1 fold |
| `F-F-F-F-F-F` | BTN | Everyone folds to BTN |
| `F-F-F-F-F-F-R2` | SB | BTN opens to 2bb |
| `F-F-F-F-F-F-R2-F-R6` | BTN | SB 3-bets to 6bb, BTN to decide |

---

## Response

### Top-Level Structure

```json
{
  "action_solutions": [...],
  "players_info": [...],
  "hand_categories_range": [],
  "draw_categories_range": [],
  "blocker_rate": [],
  "unblocker_rate": [],
  "blockers_frequencies": null,
  "game": {...},
  "usage": null,
  "warning": null,
  "hands_locked": null
}
```

| Field | Type | Description |
|-------|------|-------------|
| `action_solutions` | ActionSolution[] | Strategy data for each available action |
| `players_info` | PlayerInfo[] | **1 entry** — the active player with range |
| `hand_categories_range` | int[] | Always `[]` for preflop |
| `draw_categories_range` | int[] | Always `[]` for preflop |
| `blocker_rate` | float[] | Always `[]` for preflop |
| `unblocker_rate` | float[] | Always `[]` for preflop |
| `blockers_frequencies` | null | Always `null` for preflop |
| `game` | Game | Full game state (9 players) |
| `usage` | null | Usage tracking (always null) |
| `warning` | string \| null | Warning message if applicable |
| `hands_locked` | any \| null | Paywall/access lock indicator |

**Key differences from postflop:**
- `hand_categories_range`, `draw_categories_range`, `blocker_rate`, `unblocker_rate` are always empty arrays (no board = no hand/draw categories)
- `blockers_frequencies` is always `null` (no board cards to block)
- `players_info` has **1 entry** (not 2 like postflop)

---

### ActionSolution

Each entry represents one available action at the current decision node.

```json
{
  "action": {
    "code": "R2",
    "position": "UTG+2",
    "type": "RAISE",
    "betsize": "2",
    "allin": false,
    "is_hand_end": false,
    "is_showdown": false,
    "next_street": false,
    "display_name": "RAISE",
    "simple_group": "RAISE",
    "advanced_group": "BET_SMALL",
    "betsize_by_pot": 0.2758620689655172,
    "next_position": "LJ"
  },
  "total_frequency": 0.196961,
  "total_ev": 0.20851745,
  "total_combos": 261.17,
  "strategy": [0.0, 0.0, 0.0, ...],
  "evs": [-0.19195, -0.52871, -0.35936, ...],
  "hand_categories": [],
  "draw_categories": [],
  "equity_buckets": null,
  "equity_buckets_advanced": null,
  "tournament_evs_converter": null
}
```

#### ActionDef Fields

| Field | Type | Description |
|-------|------|-------------|
| `code` | string | Action code: `F`, `C`, `R{size}`, `RAI` |
| `position` | string | Position of the acting player |
| `type` | string | Action type: `FOLD`, `CALL`, `RAISE` |
| `betsize` | string | Bet/raise size in big blinds (`"0"` for fold) |
| `allin` | boolean | Whether this action is all-in |
| `is_hand_end` | boolean | `true` if everyone else folds |
| `is_showdown` | boolean | Always `false` for preflop |
| `next_street` | boolean | Always `false` for preflop |
| `display_name` | string | Human-readable label: `FOLD`, `RAISE`, `ALLIN`, `CALL` |
| `simple_group` | string | Simple grouping: `FOLD`, `RAISE`, `CALL` |
| `advanced_group` | string | Detailed grouping: `FOLD`, `BET_SMALL`, `BET_MEDIUM`, `BET_LARGE`, `BET_OVERBET`, `CALL` |
| `betsize_by_pot` | float \| null | Bet size as fraction of pot (`null` for fold/call) |
| `next_position` | string | Position of the next player to act |

#### ActionSolution Fields

| Field | Type | Description |
|-------|------|-------------|
| `action` | ActionDef | Action metadata |
| `total_frequency` | float | Weighted average frequency across all in-range hands [0.0, 1.0] |
| `total_ev` | float | Overall expected value for the active player |
| `total_combos` | float | Total number of hand combos that take this action |
| `strategy` | float[169] | Probability of taking this action for each hand group [0.0, 1.0] |
| `evs` | float[169] | Expected value for each hand group. `0.0` for FOLD actions. |
| `hand_categories` | int[] | Always `[]` for preflop |
| `draw_categories` | int[] | Always `[]` for preflop |
| `equity_buckets` | null | Always `null` for preflop |
| `equity_buckets_advanced` | null | Always `null` for preflop |
| `tournament_evs_converter` | null | Always `null` for preflop |

**Note:** The preflop `ActionSolution` includes `total_ev`, `total_combos`, `hand_categories`, `draw_categories`, `equity_buckets`, `equity_buckets_advanced`, and `tournament_evs_converter` fields that are not present in the postflop spec.

---

### PlayerInfo

**1 entry** — the active player making the decision.

```json
{
  "player": {
    "relative_postflop_position": "OOP",
    "hand": null,
    "is_dealer": false,
    "is_folded": false,
    "is_hero": true,
    "is_active": true,
    "stack": "20.125",
    "current_stack": "20.000",
    "chips_on_table": "0",
    "bounty": null,
    "profile": null,
    "position": "UTG+2",
    "bounty_in_bb": null,
    "name": "UTG+2",
    "seat": 2
  },
  "range": [1.0, 1.0, 1.0, 1.0, ...]
}
```

#### Player Fields

| Field | Type | Description |
|-------|------|-------------|
| `relative_postflop_position` | string \| null | `"OOP"` or `"IP"`. In preflop, most players are `"OOP"`, only BTN is `"IP"`. |
| `hand` | string \| null | Specific hand (null in solution mode) |
| `is_dealer` | boolean | Whether this player is the dealer (BTN) |
| `is_folded` | boolean | Whether this player has folded |
| `is_hero` | boolean | Whether this is the active decision maker |
| `is_active` | boolean | Whether it's this player's turn to act |
| `stack` | string | Starting stack in BB |
| `current_stack` | string | Remaining stack (after posting blinds/antes) |
| `chips_on_table` | string | Chips currently committed |
| `bounty` | string \| null | Bounty amount (MTT bounty formats) |
| `profile` | string \| null | Player profile identifier |
| `position` | string | Table position |
| `bounty_in_bb` | string \| null | Bounty expressed in big blinds |
| `name` | string | Display name (e.g., `"UTG+2"`, `"BTN"`) |
| `seat` | int | Seat number (0-indexed) |

#### Range Array

| Field | Type | Description |
|-------|------|-------------|
| `range` | float[169] | Range frequency for each hand group [0.0, 1.0]. At the start of preflop, all values are `1.0` (full range). |

---

### Game

Full game state at the current preflop decision point.

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
  "active_position": "UTG+2",
  "board": "",
  "bet_display_name": "RAISE"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `players` | GamePlayer[] | All 9 seats (including folded players) |
| `current_street` | StreetInfo | Always `type: "PREFLOP"` |
| `pot` | string | Total pot size in BB (antes + blinds + raises) |
| `pot_odds` | float \| null | Pot odds as a decimal (e.g., `0.276`). Note: this is a number, not a string. |
| `active_position` | string | Position of the player to act |
| `board` | string | Always empty string `""` (no community cards) |
| `bet_display_name` | string | Always `"RAISE"` for preflop |

**Differences from postflop `Game`:**
- `players` has **9 entries** (full table), not 2 (heads-up)
- `board` is always empty
- `pot_odds` is a numeric value (not null or string)
- `bet_display_name` is `"RAISE"` (not `"BET"`)
- Player order starts from the active position and wraps around the table

#### StreetInfo

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Always `"PREFLOP"` |
| `start_pot` | string | Pot size at the start (antes + blinds = `"2.625"` for this format) |
| `end_pot` | string | Pot size including current action |

#### Game Players vs PlayerInfo

- `game.players` — All 9 seats. Folded players have `is_folded: true` and `relative_postflop_position: null`. No `range`, `name`, or `seat` fields.
- `players_info` — Only the **1 active player** making the decision. Includes `range[169]`, `name`, and `seat`.

---

## Hand Group Indexing

All 169-element arrays use the standard 13×13 hand group matrix. Unlike the postflop API (which uses 1326-element arrays for individual card combos), preflop arrays are grouped by hand type since suit isomorphism applies preflop.

### Matrix Layout

```
         A    K    Q    J    T    9    8    7    6    5    4    3    2
    A  [ 0]  [ 1]  [ 2]  [ 3]  [ 4]  [ 5]  [ 6]  [ 7]  [ 8]  [ 9] [10] [11] [12]
    K  [13]  [14]  [15]  [16]  [17]  [18]  [19]  [20]  [21]  [22] [23] [24] [25]
    Q  [26]  [27]  [28]  [29]  [30]  [31]  [32]  [33]  [34]  [35] [36] [37] [38]
    J  [39]  [40]  [41]  [42]  [43]  [44]  [45]  [46]  [47]  [48] [49] [50] [51]
    T  [52]  [53]  [54]  [55]  [56]  [57]  [58]  [59]  [60]  [61] [62] [63] [64]
    9  [65]  [66]  [67]  [68]  [69]  [70]  [71]  [72]  [73]  [74] [75] [76] [77]
    8  [78]  [79]  [80]  [81]  [82]  [83]  [84]  [85]  [86]  [87] [88] [89] [90]
    7  [91]  [92]  [93]  [94]  [95]  [96]  [97]  [98]  [99] [100] [101] [102] [103]
    6  [104] [105] [106] [107] [108] [109] [110] [111] [112] [113] [114] [115] [116]
    5  [117] [118] [119] [120] [121] [122] [123] [124] [125] [126] [127] [128] [129]
    4  [130] [131] [132] [133] [134] [135] [136] [137] [138] [139] [140] [141] [142]
    3  [143] [144] [145] [146] [147] [148] [149] [150] [151] [152] [153] [154] [155]
    2  [156] [157] [158] [159] [160] [161] [162] [163] [164] [165] [166] [167] [168]
```

- **Diagonal** (row == col): Pocket pairs — AA (0), KK (14), QQ (28), …, 22 (168)
- **Above diagonal** (col > row): Suited hands — AKs (1), AQs (2), KQs (15), …
- **Below diagonal** (row > col): Offsuit hands — AKo (13), AQo (26), KQo (27), …

### Examples

| Hand | Index | Calculation |
|------|-------|-------------|
| AA | 0 | row 0, col 0 (pair) |
| AKs | 1 | row 0, col 1 (suited) |
| AKo | 13 | row 1, col 0 (offsuit) |
| KK | 14 | row 1, col 1 (pair) |
| 72o | 91 | row 7, col 0 (offsuit) |
| 22 | 168 | row 12, col 12 (pair) |

### Index Formula

```
index = row * 13 + col
```

Where row and col are rank indices: A=0, K=1, Q=2, J=3, T=4, 9=5, 8=6, 7=7, 6=8, 5=9, 4=10, 3=11, 2=12.

For a hand XY:
- If X == Y: pair → `index = rank * 13 + rank` (diagonal)
- If suited: row = higher rank, col = lower rank (above diagonal)
- If offsuit: row = lower rank, col = higher rank (below diagonal)

---

## Response Size

| Component | Approximate Size |
|-----------|-----------------|
| Per action (strategy + evs) | ~2.7 KB (169 × 2 arrays) |
| 3 actions typical | ~8 KB |
| 1 player range | ~1.4 KB |
| Game metadata (9 players) | ~5 KB |
| **Total (uncompressed)** | **~68–122 KB** |

Much smaller than postflop (~960 KB) due to 169 vs 1326 array sizes.

---

## Example

### Request

```
GET /v4/solutions/spot-solution/?gametype=MTTGeneralV2&depth=20.125&stacks=20.125-20.125-20.125-20.125-20.125-20.125-20.125-20.125-20.125&preflop_actions=F-F
```

**Scenario:** 9-handed MTT, 20.125bb effective. UTG folds, UTG+1 folds. UTG+2 is first to act.

### Response Summary

3 available actions for UTG+2:

| Action | Code | Type | Bet Size (BB) | Pot Fraction | Total Frequency |
|--------|------|------|---------------|--------------|-----------------|
| Fold | `F` | FOLD | 0 | — | 80.3% |
| Raise 2bb | `R2` | RAISE | 2 | 0.276 | 19.7% |
| All-in | `RAI` | RAISE | 20.0 | 5.241 | 0.0% |

**Note:** UTG+2 has a tight opening range (~20% of hands), which is standard for early position in a 20bb MTT.
