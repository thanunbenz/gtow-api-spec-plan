# Backend API Implementation

Rust backend that serves pre-computed GTO solver data following the GTO Wizard API spec.

## Tech Stack

| Component | Choice |
|---|---|
| Language | Rust (edition 2024) |
| Web framework | Actix-web 4 |
| Serialization | serde + serde_json |
| Async runtime | Tokio |
| S3 client | aws-sdk-s3 |
| Env config | dotenvy |
| Logging | env_logger + log |
| Cache | LRU (in-memory) |

## Project Structure

```
api/
  Cargo.toml
  config/
    .env.example                  # Env template with docs
  src/
    main.rs                       # Server entry, env loading, route setup
    config/
      constants.rs                # POSITIONS_8/9, RANKS, BB_CHIP_VALUE, combo_count()
    types/
      s3.rs                       # S3 data types (S3Node, S3Settings, SolverTree)
      api.rs                      # API response types matching spec exactly
    services/
      tree_loader.rs              # Load solver tree from local FS or S3 + LRU cache
      node_traversal.rs           # Parse preflop_actions + walk game tree
    transform/
      hand_index.rs               # 169 hand name <-> array index mapping
      action_builder.rs           # S3 action -> ActionDef (code, betsize, all-in)
      game_state.rs               # Build game object (9 players, pot, pot_odds)
      player_info.rs              # Build PlayerInfo (range, EVs, hand counters)
      spot_solution.rs            # Full spot-solution response builder
      next_actions.rs             # Lightweight next-actions response builder
    routes/
      preflop_solution.rs         # GET /v4/solutions/spot-solution
      next_actions.rs             # GET /v1/poker/next-actions
```

**Total: ~1,978 lines of Rust across 19 files.**

## Data Flow

```
HTTP Request
  |
  |  Query params: gametype, depth, stacks, preflop_actions
  v
Route Handler (routes/)
  |
  |  1. Parse stacks -> folder name: "10 17 54 17 15 32 21 29"
  |  2. Load tree from cache or disk/S3
  v
Tree Loader (services/tree_loader.rs)
  |
  |  Reads: settings.json, equity.json, nodes/*.json (~3000 files)
  |  Caches: LRU in-memory (16 trees max)
  v
Node Traversal (services/node_traversal.rs)
  |
  |  Walks tree: "F-R2" -> node 0 -> fold -> node 1 -> raise -> node 726
  |  Matches: F, C, R{size}, RAI against S3 action.type + amount
  v
Transform Layer (transform/)
  |
  |  Converts S3 node data -> API response format
  |  - hands dict -> strategy[169] + evs[169] arrays
  |  - 8 S3 players -> 9 API players (insert virtual UTG+2)
  |  - chips (u64) -> BB strings with correct formatting
  v
JSON Response
```

## Key Design Decisions

### 1. 8-to-9 Player Mapping

S3 solver data has 8 players. The API spec requires 9 (with UTG+2).

```
S3 index:  [0     1       2   3   4   5    6   7 ]
S3 pos:    [UTG   UTG+1   LJ  HJ  CO  BTN  SB  BB]

API seat:  [0     1       2       3   4   5   6    7   8 ]
API pos:   [UTG   UTG+1   UTG+2*  LJ  HJ  CO  BTN  SB  BB]
                          ^^^^^^ virtual (always folded)
```

Implemented in `game_state.rs`:
- `s3_to_api_seat(s3_idx)` -- maps 0-7 to 0-8 (skipping 2)
- `api_seat_to_s3(api_seat)` -- maps 0-8 back, returns `None` for UTG+2

### 2. Two GamePlayer Structs

The spec uses different JSON field ordering per endpoint:

| Endpoint | First field | Struct |
|---|---|---|
| spot-solution | `relative_postflop_position` | `SpotGamePlayer` |
| next-actions | `position` | `NextGamePlayer` |

Separate structs in `types/api.rs` ensure serde serializes fields in the correct order.

### 3. Number Formatting Rules

Derived from spec examples:

| Field | Format | Examples |
|---|---|---|
| `betsize` | Smart: integer -> no decimals, fraction -> 3 dec | `"0"`, `"2"`, `"20.000"` |
| `stack`, `current_stack`, `pot` | Always 3 decimals | `"20.125"`, `"18.000"` |
| `chips_on_table` (spot) | Smart: integer -> no decimals, fraction -> 3 dec | `"0"`, `"0.500"` |
| `chips_on_table` (next) | Always 2 decimals | `"0.00"`, `"0.50"` |
| `pot_odds` | String, 3 decimals | `"0.294"` |
| `betsize_by_pot` | String, trim trailing zeros | `"0.4"`, `"1.02752294"` |

### 4. All-in Detection

Two methods combined:
1. Last raise action in `node.actions[]` is always all-in
2. Amount >= effective stack (stack minus ante)

### 5. Action Code Matching

Maps API action codes to S3 node actions during tree traversal:

| API code | Matching rule |
|---|---|
| `F` | `action.type == "F"` |
| `C` | `action.type == "C"` |
| `R{size}` | `action.type == "R"` AND `amount == size * BB_CHIP_VALUE` |
| `RAI` | `action.type == "R"` with highest amount |

With tolerance fallback for floating-point edge cases.

### 6. Hand Index Mapping

169 hand groups mapped to array indices using 13x13 matrix:

```
index = row * 13 + col

Pair:    row == col  (AA=0, KK=14, 22=168)
Suited:  col > row   (AKs=1, upper triangle)
Offsuit: row > col   (AKo=13, lower triangle)
```

Pre-computed as static `HashMap` via `once_cell::Lazy`.

### 7. Caching Strategy

- LRU cache with 16 slots (one slot = one stack configuration)
- Each tree is ~33MB in memory (~3000 nodes)
- First request loads all nodes; subsequent requests hit cache
- S3 mode: downloads all nodes on first access, then serves from memory

## API Endpoints

### GET /v4/solutions/spot-solution

Full GTO strategy for a preflop decision node.

**Query params:**
```
gametype=MTTGeneralV2
depth=20.125
stacks=10-17-54-17-15-32-21-29
preflop_actions=F-F                  (optional, empty = root)
```

**Response:** `SpotSolutionResponse` -- full strategy data.

<details>
<summary>Example action_solutions (FOLD + RAI)</summary>

```json
{
  "code": "F",
  "position": "UTG",
  "type": "FOLD",
  "betsize": "0",
  "allin": false,
  "is_hand_end": false,
  "is_showdown": false,
  "next_street": false,
  "display_name": "FOLD",
  "simple_group": "FOLD",
  "advanced_group": "FOLD",
  "betsize_by_pot": null,
  "next_position": "UTG+1"
}
```

```json
{
  "code": "RAI",
  "position": "UTG",
  "type": "RAISE",
  "betsize": "9.875",
  "allin": true,
  "is_hand_end": false,
  "is_showdown": false,
  "next_street": false,
  "display_name": "ALLIN",
  "simple_group": "RAISE",
  "advanced_group": "BET_OVERBET",
  "betsize_by_pot": "3.55",
  "next_position": "UTG+1"
}
```

</details>

<details>
<summary>Example game object (spot-solution)</summary>

Players ordered starting from `active_position`, wrapping clockwise. 9 players total (including virtual UTG+2).

```json
{
  "players": [
    {
      "relative_postflop_position": "OOP",
      "hand": null,
      "is_dealer": false,
      "is_folded": false,
      "is_hero": true,
      "is_active": true,
      "stack": "10.000",
      "current_stack": "10.000",
      "chips_on_table": "0",
      "bounty": null,
      "profile": null,
      "position": "UTG",
      "bounty_in_bb": null
    }
  ],
  "current_street": { "type": "PREFLOP", "start_pot": "2.500", "end_pot": "2.500" },
  "pot": "2.500",
  "pot_odds": "0.286",
  "active_position": "UTG",
  "board": "",
  "bet_display_name": "RAISE"
}
```

Note: `chips_on_table` uses smart format (`"0"`, `"0.500"`, `"1"`).

</details>

<details>
<summary>Example players_info</summary>

1 entry for opening spot, 2 entries when facing a raise (aggressor + hero).

```json
{
  "player": {
    "relative_postflop_position": "OOP",
    "hand": null,
    "is_dealer": false,
    "is_folded": false,
    "is_hero": true,
    "is_active": true,
    "stack": "10.000",
    "current_stack": "10.000",
    "chips_on_table": "0",
    "bounty": null,
    "profile": null,
    "position": "UTG",
    "bounty_in_bb": null,
    "name": "UTG",
    "seat": 0
  },
  "range": [1.0, 1.0, 1.0, "...169 total"],
  "hand_evs": [1.65101, 0.64188, 0.35418, "...169 total"],
  "hand_eqs": [0.0, "...169 total"],
  "hand_eqrs": [],
  "total_ev": 0.033,
  "total_eq": null,
  "total_eqr": null,
  "pot_share": 0.0,
  "total_combos": 1326.0,
  "simple_hand_counters": { "AA": 6.0, "AKs": 4.0, "...": "169 entries" },
  "equity_buckets_range": [],
  "equity_buckets_advanced_range": [],
  "equity_buckets": [0, 0, 0, 0],
  "equity_buckets_advanced": [0, 0, 0, 0, 0, 0, 0],
  "hand_categories": [0, 0, 0, "...17 total"],
  "draw_categories": [0, 0, 0, "...8 total"],
  "relative_postflop_position": "OOP",
  "eq_percentile": [0.0, "...169 total"],
  "tournament_evs_converter": null
}
```

</details>

### GET /v1/poker/next-actions

Available actions at a decision node (no strategy data).

**Query params:** same as spot-solution

<details>
<summary>Example response</summary>

Players ordered in table order (UTG -> BB). `chips_on_table` uses 2-decimal format.

```json
{
  "next_actions": {
    "game": {
      "players": [
        {
          "position": "UTG",
          "relative_postflop_position": null,
          "is_folded": true,
          "is_hero": false,
          "stack": "10.000",
          "chips_on_table": "0.00",
          "...": "9 players total"
        }
      ],
      "pot": "2.500",
      "pot_odds": "0.286",
      "active_position": "UTG+1"
    },
    "available_actions": [
      {
        "action": {
          "code": "F",
          "position": "UTG+1",
          "type": "FOLD",
          "betsize": "0",
          "next_position": "LJ"
        },
        "frequency": null,
        "is_solution_end": false,
        "can_be_solved_by_ai": false,
        "next_position": "LJ",
        "selected": false
      },
      {
        "action": {
          "code": "R2",
          "position": "UTG+1",
          "type": "RAISE",
          "betsize": "2",
          "betsize_by_pot": "0.4",
          "next_position": "LJ"
        },
        "is_solution_end": false,
        "next_position": "LJ"
      },
      {
        "action": {
          "code": "RAI",
          "position": "UTG+1",
          "type": "RAISE",
          "betsize": "16.875",
          "allin": true,
          "display_name": "ALLIN"
        },
        "is_solution_end": true,
        "next_position": "UTG+2"
      }
    ],
    "preset_action_code": "F"
  },
  "future_actions": []
}
```

</details>

## Running

```bash
cd api
cp config/.env.example config/.env
# Edit .env as needed

# Development
DATA_PATH=../output cargo run

# Production (S3)
DATA_SOURCE=s3 S3_BUCKET=my-bucket cargo run --release
```

## Testing

```bash
cd api
cargo test    # Unit tests (hand_index mapping)

# Integration test
DATA_PATH=../output cargo run &
curl "http://localhost:3001/v4/solutions/spot-solution?gametype=MTTGeneralV2&depth=20.125&stacks=10-17-54-17-15-32-21-29"
```

## Dependencies

| Crate | Version | Purpose |
|---|---|---|
| actix-web | 4 | HTTP server + routing |
| actix-cors | 0.7 | CORS middleware |
| serde | 1 | Serialization framework |
| serde_json | 1 | JSON serialization |
| tokio | 1 | Async runtime |
| once_cell | 1 | Lazy static initialization |
| lru | 0.12 | LRU cache for solver trees |
| thiserror | 2 | Error types |
| aws-config | 1 | AWS configuration loading |
| aws-sdk-s3 | 1 | S3 client |
| dotenvy | 0.15 | .env file loading |
| env_logger | 0.11 | Log output |
| log | 0.4 | Log macros |
