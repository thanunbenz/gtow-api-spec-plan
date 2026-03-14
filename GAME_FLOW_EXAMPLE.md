# Complete Game Flow Example

This guide demonstrates a full poker hand from flop to showdown using the Precompute API.

## Scenario: Check-Check All Streets

Board: 2c 3c 4h (flop), Kh (turn), 9s (river)

Players: OOP (BB) vs IP (BTN)

---

## Step 1: Flop Root (OOP to act)

**Request:**
```bash
curl -X POST http://localhost:3000/v1/solve/precompute \
  -H "Content-Type: application/json" \
  -d '{
    "solution_id": "2c3c4h_p2",
    "player": "OOP",
    "flop_actions": ""
  }'
```

**Response (partial):**
```json
{
  "action_solutions": [
    {
      "action": {
        "code": "X",
        "type": "CHECK",
        "display_name": "CHECK",
        "next_street": true,
        "is_showdown": false
      },
      "total_frequency": 0.999
    },
    {
      "action": {
        "code": "B0.5",
        "type": "BET",
        "display_name": "Bet 50%",
        "betsize_by_pot": 0.5
      },
      "total_frequency": 0.0008
    }
  ]
}
```

**Decision:** OOP checks (next_street: true means IP can also check to advance)

---

## Step 2: Flop - IP Response to Check

**Request:**
```bash
curl -X POST http://localhost:3000/v1/solve/precompute \
  -H "Content-Type: application/json" \
  -d '{
    "solution_id": "2c3c4h_p2",
    "player": "IP",
    "flop_actions": "X"
  }'
```

**Response (partial):**
```json
{
  "action_solutions": [
    {
      "action": {
        "code": "X",
        "type": "CHECK",
        "display_name": "CHECK",
        "next_street": true,
        "is_showdown": false,
        "next_position": "BB"
      },
      "total_frequency": 0.999
    },
    {
      "action": {
        "code": "B0.5",
        "type": "BET",
        "display_name": "Bet 50%"
      },
      "total_frequency": 0.0008
    }
  ]
}
```

**Decision:** IP checks (both checked, proceed to turn)

---

## Step 3: Turn Root (OOP to act)

**Request:**
```bash
curl -X POST http://localhost:3000/v1/solve/precompute \
  -H "Content-Type: application/json" \
  -d '{
    "solution_id": "2c3c4h_p2",
    "player": "OOP",
    "flop_actions": "X-X",
    "turn_actions": ""
  }'
```

**Response (partial):**
```json
{
  "action_solutions": [
    {
      "action": {
        "code": "X",
        "type": "CHECK",
        "display_name": "CHECK",
        "next_street": true,
        "is_showdown": false
      },
      "total_frequency": 0.999
    },
    {
      "action": {
        "code": "B0.66",
        "type": "BET",
        "display_name": "Bet 66%",
        "betsize_by_pot": 0.66
      },
      "total_frequency": 0.0008
    }
  ]
}
```

**Note:** Bet sizes changed (0.66 instead of 0.5 for turn)

**Decision:** OOP checks

---

## Step 4: Turn - IP Response to Check

**Request:**
```bash
curl -X POST http://localhost:3000/v1/solve/precompute \
  -H "Content-Type: application/json" \
  -d '{
    "solution_id": "2c3c4h_p2",
    "player": "IP",
    "flop_actions": "X-X",
    "turn_actions": "X"
  }'
```

**Response (partial):**
```json
{
  "action_solutions": [
    {
      "action": {
        "code": "X",
        "type": "CHECK",
        "display_name": "CHECK",
        "next_street": true,
        "is_showdown": false
      },
      "total_frequency": 0.999
    },
    {
      "action": {
        "code": "B0.66",
        "type": "BET",
        "display_name": "Bet 66%"
      },
      "total_frequency": 0.0008
    }
  ]
}
```

**Decision:** IP checks (both checked, proceed to river)

---

## Step 5: River Root (OOP to act)

**Request:**
```bash
curl -X POST http://localhost:3000/v1/solve/precompute \
  -H "Content-Type: application/json" \
  -d '{
    "solution_id": "2c3c4h_p2",
    "player": "OOP",
    "flop_actions": "X-X",
    "turn_actions": "X-X",
    "river_actions": ""
  }'
```

**Response (partial):**
```json
{
  "action_solutions": [
    {
      "action": {
        "code": "X",
        "type": "CHECK",
        "display_name": "CHECK",
        "next_street": false,
        "is_showdown": true,
        "is_hand_end": false
      },
      "total_frequency": 0.999
    },
    {
      "action": {
        "code": "B0.75",
        "type": "BET",
        "display_name": "Bet 75%",
        "betsize_by_pot": 0.75
      },
      "total_frequency": 0.0008
    },
    {
      "action": {
        "code": "B1.5",
        "type": "BET",
        "display_name": "Bet 150%",
        "betsize_by_pot": 1.5
      },
      "total_frequency": 0.0001
    }
  ]
}
```

**Note:**
- Bet sizes changed again (0.75, 1.5 for river)
- CHECK has is_showdown: true (will lead to showdown if both check)

**Decision:** OOP checks

---

## Step 6: River - IP Final Decision

**Request:**
```bash
curl -X POST http://localhost:3000/v1/solve/precompute \
  -H "Content-Type: application/json" \
  -d '{
    "solution_id": "2c3c4h_p2",
    "player": "IP",
    "flop_actions": "X-X",
    "turn_actions": "X-X",
    "river_actions": "X"
  }'
```

**Response (partial):**
```json
{
  "action_solutions": [
    {
      "action": {
        "code": "X",
        "type": "CHECK",
        "display_name": "CHECK",
        "next_street": false,
        "is_showdown": true,
        "is_hand_end": false
      },
      "total_frequency": 0.999,
      "strategy": [0.999, 0.999, ...],  // 1326 elements
      "evs": [0.4, 0.41, ...]            // 1326 elements
    },
    {
      "action": {
        "code": "B0.75",
        "type": "BET",
        "display_name": "Bet 75%"
      },
      "total_frequency": 0.0008
    }
  ]
}
```

**Decision:** IP checks

**Result:** SHOWDOWN (is_showdown: true)

---

## Game Summary

| Street | OOP Action | IP Action | Next Street |
|--------|-----------|-----------|-------------|
| Flop   | Check     | Check     | Turn        |
| Turn   | Check     | Check     | River       |
| River  | Check     | Check     | Showdown    |

**Final State:** Both players checked all streets, hand goes to showdown

---

## Key Observations

### Bet Size Progression

| Street | Small Bet | Large Bet |
|--------|-----------|-----------|
| Flop   | 50%       | 100%      |
| Turn   | 66%       | 100%      |
| River  | 75%       | 150%      |

### Game State Flags

**Flop/Turn:**
- `next_street: true` (can advance to next street)
- `is_showdown: false`

**River:**
- `next_street: false` (last street)
- `is_showdown: true` (CHECK leads to showdown)

---

## Alternative Scenario: OOP Bets River

**Request:**
```bash
curl -X POST http://localhost:3000/v1/solve/precompute \
  -H "Content-Type: application/json" \
  -d '{
    "solution_id": "2c3c4h_p2",
    "player": "IP",
    "flop_actions": "X-X",
    "turn_actions": "X-X",
    "river_actions": "B0.75"
  }'
```

**Response:** IP's options facing OOP's 75% pot bet

**Available actions:**
- FOLD
- CALL
- RAISE (various sizes)

---

## Complete Request Sequence

```bash
# 1. Flop OOP
curl -X POST ... -d '{"player":"OOP","flop_actions":""}'

# 2. Flop IP (after OOP check)
curl -X POST ... -d '{"player":"IP","flop_actions":"X"}'

# 3. Turn OOP (after X-X)
curl -X POST ... -d '{"player":"OOP","flop_actions":"X-X","turn_actions":""}'

# 4. Turn IP (after OOP check)
curl -X POST ... -d '{"player":"IP","flop_actions":"X-X","turn_actions":"X"}'

# 5. River OOP (after X-X, X-X)
curl -X POST ... -d '{"player":"OOP","flop_actions":"X-X","turn_actions":"X-X","river_actions":""}'

# 6. River IP (after OOP check)
curl -X POST ... -d '{"player":"IP","flop_actions":"X-X","turn_actions":"X-X","river_actions":"X"}'
```

---

## Notes

- Each request returns 1326-element arrays for all hand combinations
- `strategy` values sum to ~1.0 across all actions for each hand
- `evs` represent expected value in chips for each hand/action pair
- Mock data uses uniform values; real implementation will have hand-specific strategies

---

## Related Documentation

- [PRECOMPUTE_API.md](PRECOMPUTE_API.md) - Complete API reference
- [FLOP_ACTIONS.md](FLOP_ACTIONS.md) - Action notation guide
- [API.md](API.md) - General API documentation
