# GTO Wizard API Spec

Reverse-engineered API specification for the GTO Wizard solver — covering both **preflop** and **postflop** endpoints.

## Specs

| Spec | File | Swagger Port | Description |
|------|------|-------------|-------------|
| **Postflop** | `openapi.yaml` | 3000 | Postflop solver (flop/turn/river), v1 endpoints |
| **Preflop** | `openapi-preflop.yaml` | 3001 | Preflop solver, v4 spot-solution + v1 next-actions |

### Preflop vs Postflop

| | Preflop | Postflop |
|---|---|---|
| **Array length** | 169 (hand groups) | 1326 (card combos) |
| **Players in solution** | 1 (active only) | 2 (IP + OOP) |
| **game.players** | 9 (full table) | 2 (heads-up) |
| **spot-solution version** | v4 | v1 |
| **gametype** | `MTTGeneralV2` | `MTTGeneral` |
| **Board** | empty `""` | 6–10 chars |
| **Response size** | ~68–122 KB | ~960 KB |
| **Blocker data** | empty / null | populated |
| **bet_display_name** | `"RAISE"` | `"BET"` |

---

## Endpoints

### Postflop

GTO Wizard uses two endpoints to power the postflop solution UI:

#### 1. Spot Solution — full strategy data

```
GET /v1/solutions/spot-solution/
```

Returns the **complete GTO solution** at a decision node: strategy frequencies, expected values, ranges, blockers, and hand categories for all 1326 hand combinations. This is the heavy payload (~960KB) that powers the strategy grid and EV display.

#### 2. Next Actions — lightweight navigation

```
GET /v1/poker/next-actions/
```

Returns only the **available actions** at the next node in the game tree. No strategy arrays, no EVs — just the action list and game state metadata. Called when the user clicks an action to navigate forward.

### Preflop

The preflop API uses the same two endpoint patterns:

#### 1. Spot Solution (v4) — full preflop strategy

```
GET /v4/solutions/spot-solution/
```

Returns GTO strategy for all **169 hand groups** at a preflop decision node. Lighter payload (~68–122KB) since preflop uses hand group arrays instead of individual combos.

#### 2. Next Actions — lightweight navigation

```
GET /v1/poker/next-actions/
```

Same endpoint as postflop. Returns available preflop actions. `future_actions` is always empty for preflop (no street transitions).

### How they work together

```
User loads a preflop spot
  └─► spot-solution  → full strategy data (render grid)

User clicks "Raise"
  └─► next-actions   → what actions exist at the next node (render buttons)
  └─► spot-solution  → full strategy for the new node (render grid)
```

### Endpoint Comparison

| | spot-solution | next-actions |
|---|---|---|
| **Purpose** | Full GTO strategy | Navigation / action list |
| **Response size** | ~68–960 KB | ~3–8 KB |
| **strategy[N]** | Yes (169 preflop / 1326 postflop) | No |
| **evs[N]** | Yes | No |
| **ranges[N]** | Yes | No |
| **blocker data** | Postflop only | No |
| **available actions** | Yes (inside `action_solutions`) | Yes (inside `available_actions`) |
| **game state** | Yes | Yes |
| **future_actions** | No | Yes (postflop street transitions only) |
| **is_solution_end** | No | Yes (terminal node detection) |
| **preset_action_code** | No | Yes (UI hint for default selection) |

---

## Project Structure

```
openapi.yaml                             # Postflop OpenAPI spec
openapi-preflop.yaml                     # Preflop OpenAPI spec

docs/
  SOLUTION_API.md                        # Postflop spot-solution endpoint spec
  NEXT_ACTIONS_API.md                    # Postflop next-actions endpoint spec
  PREFLOP_SOLUTION_API.md                # Preflop spot-solution endpoint spec
  PREFLOP_NEXT_ACTIONS_API.md            # Preflop next-actions endpoint spec

examples/
  spot-solution/                         # Postflop spot-solution examples (6)
    1_flop_root_oop/
    2_flop_check_ip/
    3_turn_root_oop/
    4_turn_check_ip/
    5_river_root_oop/
    6_river_check_ip/

  next-actions/                          # Postflop next-actions examples (6)
    1_flop_oop_check/
    2_flop_ip_check/
    3_turn_oop_check/
    4_turn_ip_check/
    5_river_oop_check/
    6_river_ip_check/

  preflop/                               # Preflop examples (8)
    1_utg2_open_spot/                    # UTG+2 opening decision
    2_utg1_open_facing_spot/             # Facing UTG+1 open
    3_btn_open_sb_next/                  # SB 3-bet next actions
    4_btn_facing_3bet_spot/              # BTN facing SB 3-bet strategy
    5_cold_call_chain_spot/              # Cold call chain
    6_3bet_squeeze_spot/                 # 3-bet squeeze next actions
    7_allin_decision_spot/               # All-in decision strategy
    8_terminal_fold_next/                # Terminal: everyone folds
```

Each example folder contains:
- `curl.txt` — full cURL command with headers (auth redacted)
- `payload.txt` — query parameters
- `response.json` — full JSON response

---

## Scenarios

### Postflop Scenario

All postflop examples use the same hand:

- **Format:** 9-handed MTT, 20bb effective
- **Preflop:** BTN opens to 2bb, BB calls
- **Board:** Ac Td 6h (flop) / Th (turn) / Ah (river)
- **Line:** Check-check on every street to showdown

### Preflop Scenario

Preflop examples cover multiple lines:

- **Format:** 9-handed MTT (`MTTGeneralV2`), 20.125bb effective
- **Stacks:** Equal (20.125bb all seats)
- **Lines captured:**
  - UTG+2 open decision (early position)
  - UTG+1 open → cold call → 3-bet → all-in sequences
  - BTN open → SB 3-bet → BTN facing 3-bet
  - Terminal node (all fold to all-in)

---

## Current Coverage

| Endpoint | Version | Phase | Status |
|----------|---------|-------|--------|
| `/solutions/spot-solution/` | v1 | Postflop | Documented |
| `/poker/next-actions/` | v1 | Postflop | Documented |
| `/solutions/spot-solution/` | v4 | Preflop | Documented |
| `/poker/next-actions/` | v1 | Preflop | Documented |

---

## Development

**Swagger UI** (interactive docs):
```bash
# Postflop
npx swagger-ui-watcher openapi.yaml

# Preflop
npx swagger-ui-watcher openapi-preflop.yaml
```

**Mock API server** (queryable endpoints):
```bash
# Postflop (port 3000)
npx @stoplight/prism-cli mock openapi.yaml --port 3000

# Preflop (port 3001)
npx @stoplight/prism-cli mock openapi-preflop.yaml --port 3001
```

### Example Queries

**Postflop** — spot solution:
```bash
curl "http://localhost:3000/v1/solutions/spot-solution?gametype=MTTGeneral&depth=20.125&preflop_actions=F-F-F-F-F-F-R2-F-C&board=AcTd6h" \
  -H "Authorization: Bearer test" \
  -H "gwclientid: test"
```

**Preflop** — spot solution:
```bash
curl "http://localhost:3001/v4/solutions/spot-solution?gametype=MTTGeneralV2&depth=20.125&stacks=20.125-20.125-20.125-20.125-20.125-20.125-20.125-20.125-20.125&preflop_actions=F-F" \
  -H "Authorization: Bearer test" \
  -H "gwclientid: test"
```

**Preflop** — next actions:
```bash
curl "http://localhost:3001/v1/poker/next-actions?gametype=MTTGeneralV2&depth=20.125&stacks=20.125-20.125-20.125-20.125-20.125-20.125-20.125-20.125-20.125&preflop_actions=F-F-F-F-F-F-R2" \
  -H "Authorization: Bearer test" \
  -H "gwclientid: test"
```

---

## Patch: 9-Handed Table (vs Original 8-Handed)

This spec extends the original GTO Wizard API from **8-handed** to **9-handed** tables. Below are the key differences:

### Seat Order

| | 8-Handed (original) | 9-Handed (this spec) |
|---|---|---|
| **Seats** | UTG, UTG+1, LJ, HJ, CO, BTN, SB, BB | UTG, UTG+1, **UTG+2**, LJ, HJ, CO, BTN, SB, BB |
| **Total positions** | 8 | 9 |

### Preflop Action Sequence

The `preflop_actions` parameter gains one additional action for the UTG+2 seat:

```diff
- F-F-F-F-F-R2-F-C          (8 actions)
+ F-F-F-F-F-F-R2-F-C        (9 actions)
  │ │ │ │ │ │ │   │ └─ BB
  │ │ │ │ │ │ │   └─── SB
  │ │ │ │ │ │ └─────── BTN
  │ │ │ │ │ └───────── CO
  │ │ │ │ └─────────── HJ
  │ │ │ └───────────── LJ
+ │ │ └─────────────── UTG+2 (new)
  │ └─────────────────  UTG+1
  └───────────────────  UTG
```

### Game State — `game.players` Array

Responses include **9 player objects** instead of 8. The additional player (UTG+2) appears after UTG+1:

```json
{ "position": "UTG",   "is_folded": true, ... },
{ "position": "UTG+1", "is_folded": true, ... },
{ "position": "UTG+2", "is_folded": true, ... },
{ "position": "LJ",    "is_folded": true, ... },
...
```

### Position Enum

The `position` field now includes `UTG+2`:

```
[BB, SB, BTN, CO, HJ, LJ, UTG+2, UTG+1, UTG]
```

### What Stays the Same

- All postflop logic, strategy arrays, EV calculations, and hand indexing (1326 combos) are unchanged
- Board encoding, action codes (X, F, C, R{size}, RAI), and response schemas remain identical
- The pot size (`5.500` for BTN open + BB call) stays the same since only UTG+2 folds preflop
- Stack sizes and bounty fields are unaffected
