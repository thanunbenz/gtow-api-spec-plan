# Precompute API

Returns complete strategy for all 1326 hand combinations at a given decision node.

## Quick Start

```bash
curl -X POST http://localhost:3000/v1/solve/precompute \
  -H "Content-Type: application/json" \
  -d '{
    "solution_id": "2c3c4h_p2",
    "player": "OOP",
    "flop_actions": ""
  }'
```

## Endpoint

**POST /v1/solve/precompute**

Returns strategy distribution across all 1326 hand combinations for each available action.

---

## Request Format

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `solution_id` | string | Yes | Model identifier (e.g., "2c3c4h_p2") |
| `player` | string | Yes | Player position: "OOP" or "IP" |
| `flop_actions` | string | Yes | Action notation (empty string for root) |
| `turn_actions` | string | No | Turn action notation (requires flop_actions) |
| `river_actions` | string | No | River action notation (requires turn_actions) |

### Request Examples

**Flop root:**
```json
{
  "solution_id": "2c3c4h_p2",
  "player": "OOP",
  "flop_actions": ""
}
```

**Turn (after flop X-X):**
```json
{
  "solution_id": "2c3c4h_p2",
  "player": "OOP",
  "flop_actions": "X-X",
  "turn_actions": ""
}
```

**River (after flop X-X, turn X-X):**
```json
{
  "solution_id": "2c3c4h_p2",
  "player": "OOP",
  "flop_actions": "X-X",
  "turn_actions": "X-X",
  "river_actions": ""
}
```

---

## Response Format

### Structure

```json
{
  "action_solutions": [
    {
      "action": {
        "code": "X",
        "position": "BB",
        "type": "CHECK",
        "betsize": "0",
        "allin": false,
        "is_hand_end": false,
        "is_showdown": false,
        "next_street": true,
        "display_name": "CHECK",
        "simple_group": "CHECK",
        "advanced_group": "CHECK",
        "betsize_by_pot": null,
        "next_position": "BTN"
      },
      "total_frequency": 0.999,
      "strategy": [0.999, 0.998, ...],  // 1326 floats
      "evs": [0.4, 0.41, ...]            // 1326 floats
    }
  ]
}
```

### ActionDef Fields

| Field | Type | Description |
|-------|------|-------------|
| `code` | string | Action code (X, B0.5, R1.8, A) |
| `position` | string | Player position (BB, BTN, SB) |
| `type` | string | Action type (CHECK, BET, CALL, FOLD, RAISE, ALLIN) |
| `betsize` | string | Actual bet size in chips |
| `allin` | boolean | Whether this is an all-in action |
| `is_hand_end` | boolean | Hand finished after this action |
| `is_showdown` | boolean | Showdown reached |
| `next_street` | boolean | Moving to next street |
| `display_name` | string | Human-readable name |
| `simple_group` | string | Simple action grouping |
| `advanced_group` | string | Advanced action grouping |
| `betsize_by_pot` | number or null | Bet size as pot fraction (null for CHECK/FOLD/CALL) |
| `next_position` | string | Next player to act |

### ActionSolution Fields

| Field | Type | Description |
|-------|------|-------------|
| `action` | ActionDef | Complete action metadata |
| `total_frequency` | float | Weighted average frequency across all hands |
| `strategy` | float[] | 1326 probabilities (one per hand combo) |
| `evs` | float[] | 1326 expected values in chips |

---

## Strategy Array

**Length:** 1326 elements (all possible hand combinations)

**Values:** Probability [0.0, 1.0] that this hand takes this action

**Example:**
```json
"strategy": [
  0.999,  // Hand combo 0: 2c2d
  0.998,  // Hand combo 1: 2c2h
  0.999,  // Hand combo 2: 2c2s
  ...     // 1323 more combos
]
```

---

## Hand Index Mapping

Frontend needs to know which array index corresponds to which hand.

### Card Encoding (0-51)

Cards are numbered sequentially by rank and suit:

| Rank | Clubs | Diamonds | Hearts | Spades |
|------|-------|----------|--------|--------|
| 2 | 0 | 1 | 2 | 3 |
| 3 | 4 | 5 | 6 | 7 |
| ... | ... | ... | ... | ... |
| A | 48 | 49 | 50 | 51 |

**Pattern:** `card_index = rank × 4 + suit` where rank ∈ [0,12], suit ∈ [0,3]

### Enumeration Algorithm

All 1326 hand combinations are enumerated using nested loops:

```
index = 0
for c1 in 0..52:
    for c2 in (c1+1)..52:
        hands[index] = (c1, c2)
        index += 1
```

**Key point:** `c2 > c1` ensures no duplicates (2c2d = 2d2c)

### Index Calculation

Given two cards, calculate their index:

```
index = c1 × (103 - c1) / 2 + (c2 - c1 - 1)
```

where `c1 < c2`

### Examples

| Hand | c1 | c2 | Index | Calculation |
|------|----|----|-------|-------------|
| 2c2d | 0 | 1 | 0 | 0×103/2 + 0 = 0 |
| 2c2h | 0 | 2 | 1 | 0×103/2 + 1 = 1 |
| 2c2s | 0 | 3 | 2 | 0×103/2 + 2 = 2 |
| AsAh | 50 | 51 | 1325 | 50×53/2 + 0 = 1325 |
| AcAd | 48 | 49 | 1320 | 48×55/2 + 0 = 1320 |

### Frontend Implementation

**Option 1: Pre-compute lookup table** (recommended)
```javascript
// Generate once on app load
const handToIndex = {};
let index = 0;
for (let c1 = 0; c1 < 52; c1++) {
  for (let c2 = c1 + 1; c2 < 52; c2++) {
    handToIndex[`${c1},${c2}`] = index++;
  }
}
```

**Option 2: Calculate on demand**
```javascript
function getHandIndex(c1, c2) {
  if (c1 > c2) [c1, c2] = [c2, c1];
  return c1 * (103 - c1) / 2 + (c2 - c1 - 1);
}
```

### Interactive Demo

See [hand_enumeration_demo.html](hand_enumeration_demo.html) for a working example with search and visualization.

---

## EVs Array

**Length:** 1326 elements

**Values:** Expected value in chips for each hand combo

**Example:**
```json
"evs": [
  0.4,   // Hand combo 0: +0.4 chips
  0.41,  // Hand combo 1: +0.41 chips
  0.39,  // Hand combo 2: +0.39 chips
  ...    // 1323 more combos
]
```

---

## Multi-Street Navigation

### Flop

```json
{
  "solution_id": "2c3c4h_p2",
  "player": "OOP",
  "flop_actions": ""
}
```

Available actions: X, B0.5, B1.0, A

### Turn (after both check on flop)

```json
{
  "solution_id": "2c3c4h_p2",
  "player": "OOP",
  "flop_actions": "X-X",
  "turn_actions": ""
}
```

Available actions: X, B0.66, B1.0, A

### River (after both check on flop and turn)

```json
{
  "solution_id": "2c3c4h_p2",
  "player": "OOP",
  "flop_actions": "X-X",
  "turn_actions": "X-X",
  "river_actions": ""
}
```

Available actions: X (is_showdown=true), B0.75, B1.5, A

---

## Game State Flags

### is_showdown

True when CHECK action leads to showdown (river only).

**Example:**
```json
{
  "code": "X",
  "type": "CHECK",
  "is_showdown": true,
  "next_street": false
}
```

### next_street

True when CHECK-CHECK moves to next street.

**Example:**
```json
{
  "code": "X",
  "type": "CHECK",
  "next_street": true,
  "is_hand_end": false
}
```

### is_hand_end

True when action ends the hand (fold or river showdown).

---

## Response Size

| Component | Size |
|-----------|------|
| Per action | ~11KB (1326 floats × 2 arrays × 4 bytes + metadata) |
| Typical node (4 actions) | ~44KB |
| Compressed (gzip) | ~12-18KB |

**Note:** Add `Accept-Encoding: gzip` header for automatic compression

---

## Action Code Reference

| Code | Type | betsize_by_pot | Description |
|------|------|----------------|-------------|
| `X` | CHECK | null | Check |
| `F` | FOLD | null | Fold |
| `C` | CALL | null | Call |
| `B{frac}` | BET | {frac} | Bet {frac} × pot |
| `R{frac}` | RAISE | {frac} | Raise {frac} × pot |
| `A` | ALLIN | varies | All-in |

**Examples:**
- `B0.5` = Bet 50% pot
- `B1.0` = Bet 100% pot (pot-sized bet)
- `R1.8` = Raise to 1.8× pot

See [FLOP_ACTIONS.md](FLOP_ACTIONS.md) for complete action notation guide.

---

## Performance

| Metric | Value |
|--------|-------|
| Batch inference | ~50-100ms (mock) |
| Response size | ~44KB uncompressed |
| Response size (gzip) | ~15KB |
| Cold start | <200ms |

**Note:** Performance metrics are for mock implementation. Real ONNX inference TBD.

---

## Error Responses

### 400 Bad Request
```json
{
  "error": "Invalid flop_actions format"
}
```

**Causes:**
- Malformed action notation
- Invalid player value (not "OOP" or "IP")
- Missing required fields

### 404 Not Found
```json
{
  "error": "Solution not found"
}
```

**Causes:**
- Invalid solution_id
- Model file not found

### 500 Internal Server Error
```json
{
  "error": "Inference failed"
}
```

**Causes:**
- Model loading error
- ONNX runtime error

---

## Comparison with Single-Hand API

| Feature | `/v1/solve` | `/v1/solve/precompute` |
|---------|-------------|------------------------|
| Input | Single hand | All 1326 combos |
| Output size | ~500 bytes | ~44KB |
| Response time | ~5ms | ~100ms |
| Use case | Real-time lookup | Bulk analysis |
| Caching | Per hand | Per node |

---

## Status

**Current:** Mock implementation (2026-03-13)

**Mock behavior:**
- Returns realistic response structure
- 1326-element arrays with uniform mock data
- Multi-street support (flop/turn/river)
- GTO Wizard compatible format

**Next:** Replace with ONNX batch inference

---

## Related Documentation

- [FLOP_ACTIONS.md](FLOP_ACTIONS.md) - Action notation guide
- [API.md](API.md) - General API documentation
- [NEXT_STEPS.md](../NEXT_STEPS.md) - Implementation roadmap
