# GTO API Server

Rust backend that serves pre-computed GTO solver data, following the [GTO Wizard API spec](../README.md).

## Quick Start

```bash
# 1. Setup
cp config/.env.example config/.env

# 2. Run
cargo run

# 3. Test
curl "http://localhost:3001/v4/solutions/spot-solution?gametype=MTTGeneralV2&depth=20.125&stacks=10-17-54-17-15-32-21-29"
```

## Endpoints

| Method | Path | Description |
|---|---|---|
| GET | `/v4/solutions/spot-solution` | Full preflop strategy (strategy[169], EVs, ranges) |
| GET | `/v1/poker/next-actions` | Available actions at a node (no strategy data) |

### Query Parameters

| Parameter | Required | Example | Description |
|---|---|---|---|
| `gametype` | Yes | `MTTGeneralV2` | Game format |
| `depth` | Yes | `20.125` | Starting stack depth (BB) |
| `stacks` | Yes | `10-17-54-17-15-32-21-29` | Stack sizes, dash-separated |
| `preflop_actions` | No | `F-F-R2` | Action sequence (empty = root node) |

### Action Codes

| Code | Meaning |
|---|---|
| `F` | Fold |
| `C` | Call |
| `R{size}` | Raise to size BB (e.g., `R2`, `R6`) |
| `RAI` | All-in |

## Configuration

All settings via environment variables. See [config/.env.example](config/.env.example) for full reference.

| Variable | Default | Description |
|---|---|---|
| `PORT` | `3001` | HTTP port |
| `RUST_LOG` | - | Log level (`info`, `debug`) |
| `DATA_SOURCE` | `local` | `local` or `s3` |
| `DATA_PATH` | `output` | Solver data path (local mode) |
| `S3_BUCKET` | - | S3 bucket name (s3 mode, required) |

Full env documentation: [docs/BACKEND_ENV.md](../docs/BACKEND_ENV.md)

## Data Source

The server reads solver output from `DATA_PATH` (local) or S3. Each stack configuration is a folder:

```
output/
  10 17 54 17 15 32 21 29/     <- stacks / 10M, space-separated
    settings.json               <- solver config
    equity.json                 <- ICM equity data
    nodes/
      0.json                    <- root node (UTG to act)
      1.json                    <- after UTG folds
      ...                       <- ~3000 nodes per config
```

The folder name is derived from the `stacks` query parameter:
```
stacks=10-17-54-17-15-32-21-29  ->  folder "10 17 54 17 15 32 21 29"
```

## Architecture

```
src/
  main.rs               # Actix-web server, env loading
  config/constants.rs    # Positions, ranks, BB conversion
  types/s3.rs            # S3 data types
  types/api.rs           # API response types (matches spec)
  services/
    tree_loader.rs       # Load from FS/S3 + LRU cache (16 trees)
    node_traversal.rs    # Walk game tree by action sequence
  transform/
    hand_index.rs        # 169 hand name <-> index
    action_builder.rs    # S3 action -> API ActionDef
    game_state.rs        # Build game (9 players, pot)
    player_info.rs       # PlayerInfo (range, EVs, counters)
    spot_solution.rs     # spot-solution response
    next_actions.rs      # next-actions response
  routes/
    preflop_solution.rs  # GET /v4/solutions/spot-solution
    next_actions.rs      # GET /v1/poker/next-actions
```

Full architecture documentation: [docs/BACKEND_ARCHITECTURE.md](../docs/BACKEND_ARCHITECTURE.md)

## Build

```bash
cargo build              # dev
cargo build --release    # production
cargo test               # run tests
```

## Tech Stack

Rust (edition 2024), Actix-web 4, serde, Tokio, aws-sdk-s3, dotenvy
