# DeepRun Solver REST API

High-performance REST API for neural network poker strategy predictions.

**Features:** 4ms queries (ONNX) • Pure Rust inference • LRU caching • Swagger UI

## Overview

Real-time poker strategy predictions for flop scenarios. Returns optimal mixed strategies for any hand at any decision node.

## Quick Start

**Start server:**
```bash
cargo run -p api
# → http://localhost:3000
# → http://localhost:3000/swagger-ui
```

**Example:**

```bash
curl -X POST http://localhost:3000/v1/solve \
  -H "Content-Type: application/json" \
  -d '{
    "solution_id": "2c3c4h_p2",
    "hand": "KcKd",
    "player": "OOP",
    "flop_actions": ""
  }'
```

**Response:**

```json
{
  "board": "2c 3c 4h",
  "player": "OOP",
  "combos": [
    {
      "hand": "KcKd",
      "strategy": [0.979, 0.018, 0.003, 0.0]
    }
  ],
  "actions": [
    { "name": "Check", "frequency": 0.979, "action_type": "check" },
    { "name": "Bet(18)", "frequency": 0.018, "action_type": "bet" },
    { "name": "Bet(41)", "frequency": 0.003, "action_type": "bet" },
    { "name": "AllIn(180)", "frequency": 0.0, "action_type": "allin" }
  ],
  "num_combos": 1
}
```

## Architecture

```
api/             → HTTP server (Actix-web + Swagger UI)
model-service/   → ML inference (ONNX via tract-onnx + LRU cache)
shared/          → Common utilities (card parsing)
```

**Backend:** Pure Rust ONNX inference (40% faster than PyTorch). See [ONNX_MIGRATION.md](ONNX_MIGRATION.md) for details.

## API Reference

### POST /v1/solve

Predicts poker strategy for a given hand and game state.

**Request Body:**

```json
{
  "solution_id": "string",    // Model identifier (e.g., "2c3c4h_p2")
  "hand": "string",           // 4 characters: rank+suit per card (e.g., "KcKd")
  "player": "string",         // "OOP" or "IP"
  "flop_actions": "string"    // RECOMMENDED: Action notation (e.g., "X-R1.1")
                              // OR use node_history (legacy)
}
```

**Action Notation Format (`flop_actions`):**

Actions separated by `-`:
- **X** = Check
- **F** = Fold
- **C** = Call
- **A** = AllIn
- **B{fraction}** = Bet (e.g., `B0.5` = bet 50% pot, `B1.0` = bet 100% pot)
- **R{fraction}** = Raise (e.g., `R1.1` = raise 110% pot, `R2.0` = raise 200% pot)

**Examples:**
- `""` → Root node (no actions yet)
- `"X"` → Check
- `"X-B1.0"` → Check, then bet pot
- `"B0.5-C"` → Bet 50% pot, then call
- `"X-B1.0-R1.1"` → Check, bet pot, raise 1.1x pot

**Response:**

```json
{
  "board": "string",          // Board cards (e.g., "2c 3c 4h")
  "player": "string",         // Player position
  "combos": [{
    "hand": "string",         // Hand queried
    "strategy": [number]      // Probabilities matching actions order
  }],
  "actions": [{
    "name": "string",         // Action name (e.g., "Check", "Bet(18)")
    "frequency": number,      // Probability [0.0, 1.0]
    "action_type": "string"   // Type: check/fold/call/bet/raise/allin
  }],
  "num_combos": number        // Number of combos in response
}
```

**Status Codes:**

- `200` - Success
- `400` - Invalid request (bad hand format, unknown player)
- `404` - Solution ID not found or invalid node history
- `500` - Internal server error (model load failure, prediction error)

### GET /health

Health check endpoint.

**Response:** `{"status":"ok"}`

## Examples

**Root node (OOP):**
```json
{
  "solution_id": "2c3c4h_p2",
  "hand": "KcKd",
  "player": "OOP",
  "flop_actions": ""
}
```

**After check (IP response):**
```json
{
  "solution_id": "2c3c4h_p2",
  "hand": "AhAs",
  "player": "IP",
  "flop_actions": "X"
}
```

**After check-raise 1.1x pot (OOP response to raise):**
```json
{
  "solution_id": "2c3c4h_p2",
  "hand": "KcKd",
  "player": "OOP",
  "flop_actions": "X-R1.1"
}
```

**After check-bet 50% pot-call (IP response after calling):**
```json
{
  "solution_id": "2c3c4h_p2",
  "hand": "QhQd",
  "player": "IP",
  "flop_actions": "X-B0.5-C"
}
```

See Swagger UI at http://localhost:3000/swagger-ui for interactive examples.

## Performance

| Metric | Value |
|--------|-------|
| Inference latency (ONNX) | **~4ms** |
| First load (cold cache) | ~100ms |
| Cached query (hot) | <5ms |
| Cache capacity | 10 models |
| Model size | ~600KB (.onnx) |

## Development

**Build & Run:**
```bash
cargo build -p api
cargo run -p api
cargo test
```

**Add new model:**
1. Train: `python3 trainings/nn1_train.py data/out/YOUR_BOARD.flop`
2. Export to ONNX: `python3 trainings/export_onnx.py models/nn1/YOUR_MODEL.pt models/onnx/YOUR_MODEL.onnx`
3. Register in `model-service/src/registry.rs`
4. Restart server

## Common Errors

| Error | Fix |
|-------|-----|
| `ModelNotFound` | Check solution_id in `model-service/src/registry.rs` |
| `ONNX model not found` | Ensure .onnx and .json files exist in `models/onnx/` |
| `Invalid hand format` | Hand must be 4 chars (e.g., "KcKd") |
| `Invalid flop_actions` | Check action notation syntax (see FLOP_ACTIONS.md) |
| `Address already in use` | Kill process on port 3000: `lsof -ti:3000 \| xargs kill` |

## Legacy API (Deprecated)

The `node_history` field is deprecated. Use `flop_actions` instead.

**Legacy format:**
```json
{
  "solution_id": "2c3c4h_p2",
  "hand": "KcKd",
  "player": "OOP",
  "node_history": [0, 1]  // Array of action indices
}
```

**Recommended format:**
```json
{
  "solution_id": "2c3c4h_p2",
  "hand": "KcKd",
  "player": "OOP",
  "flop_actions": "X-B1.0"  // Human-readable notation
}
```

See [FLOP_ACTIONS.md](FLOP_ACTIONS.md) for full notation guide.
