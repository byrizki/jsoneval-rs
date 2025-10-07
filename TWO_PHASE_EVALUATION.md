# Two-Phase Table Evaluation Architecture

## Overview

Our table evaluation uses a **two-phase bidirectional approach** to handle both regular dependencies and forward references (cells that depend on future values).

---

## Phase 1: TOP TO BOTTOM (Forward Pass)

**Purpose:** Evaluate columns WITHOUT forward references  
**Direction:** Iteration 1 → N (start to end)  
**Dependencies:** Only backward references (current or past rows)

### Process:
1. Iterate from first row to last row
2. For each row:
   - Set `$iteration` to current row number
   - Evaluate columns in **topological dependency order**
   - Update scope with calculated values
   - Make current row available to next iteration

### Example Columns (Forward Pass):
```
POL_MONTH, POL_YEAR, INSAGE_POLYEAR
IND_PROB_DEATH, IND_PROB_PA
CF_DEATH, CF_PA, CF_PREMIUM
BENPAY_DEATHSA, WOP_PROB
... and any column that doesn't use VALUEAT($iteration + 1, ...)
```

---

## Phase 2: BOTTOM TO TOP (Backward Pass)

**Purpose:** Evaluate columns WITH forward references  
**Direction:** Iteration N → 1 (end to start)  
**Dependencies:** Future row values (next iteration)

### Process:
1. Iterate from last row to first row (reverse)
2. For each row:
   - Set `$iteration` to current row number
   - Restore Phase 1 values to scope
   - Evaluate forward-referencing columns
   - Update table so previous iteration can access these values

### Example Columns (Backward Pass):
```
EPV_CF_PREMIUM    = (CF_PREMIUM + VALUEAT(POL_TABLE, $iteration + 1, EPV_CF_PREMIUM)) / (1 + MONTHLY_INT_RATE)
EPV_CF_PA         = (CF_PA + VALUEAT(POL_TABLE, $iteration + 1, EPV_CF_PA)) / (1 + MONTHLY_INT_RATE)
EPV_CF_PA_ANN_AE_BEN = (CF_ANN_AE_BEN + VALUEAT(POL_TABLE, $iteration + 1, EPV_CF_PA_ANN_AE_BEN)) / (1 + MONTHLY_INT_RATE)
EPV_CF_WOP        = (CF_WOP + VALUEAT(POL_TABLE, $iteration + 1, EPV_CF_WOP)) / (1 + MONTHLY_INT_RATE)
EPV_CF_SURR_BEN   = (CF_SURRENDER + VALUEAT(POL_TABLE, $iteration + 1, EPV_CF_SURR_BEN)) / (1 + MONTHLY_INT_RATE)
```

These are **Expected Present Value (EPV)** calculations that accumulate future cash flows, working backward from the last period.

---

## Detection of Forward References

A column has forward references if its formula contains:
- `VALUEAT($iteration + positive_number, ...)` 
- `VALUEAT($iteration + 1, ...)` is most common

The detection is done at compile time by analyzing the AST for patterns like:
```rust
Add([Var("$iteration"), Literal(positive_number)])
```

---

## Why This Approach Works

### 1. **Independence**
Phase 1 columns can be evaluated independently (with topological ordering for dependencies within the phase).

### 2. **Availability**
By the time Phase 2 starts, ALL Phase 1 values are available for ALL rows.

### 3. **Correctness**
Phase 2 works backward, so when evaluating row N, row N+1 has already been calculated.

### 4. **Efficiency**
- Single forward pass for majority of columns
- Single backward pass only for EPV-style calculations
- No iteration or convergence needed

---

## Example Execution

```
Table with 781 iterations:

=== PHASE 1: Forward Pass (Top to Bottom) ===
Direction: iteration 1 → 781
Evaluating 50 columns without forward references

Iteration 1:   Calculate CF_PREMIUM, CF_DEATH, CF_PA, ...
Iteration 2:   Can reference values from iteration 1
Iteration 3:   Can reference values from iteration 1, 2
...
Iteration 781: Can reference values from iteration 1-780

=== PHASE 2: Backward Pass (Bottom to Top) ===
Direction: iteration 781 → 1
Evaluating 12 columns with forward references

Iteration 781: Calculate EPV_CF_* (using row 782 = Null → 0)
Iteration 780: Calculate EPV_CF_* (using row 781 values)
Iteration 779: Calculate EPV_CF_* (using row 780 values)
...
Iteration 1:   Calculate EPV_CF_* (using row 2 values)
```

---

## Current Results

✅ **Working Features:**
- Two-phase evaluation with clear separation
- Topological sorting of dependencies
- JavaScript-like type coercion for comparisons
- Forward and backward references
- Static + repeat row handling

⚠️ **Known Issues:**
- EPV calculations produce ~62% of expected values
- Likely formula interpretation or missing calculation component
- Core infrastructure is correct

**Execution Time:** ~7 seconds for 782 row table with 60+ columns

---

## Code Structure

```rust
// Phase 1: Normal columns (no forward refs)
let normal_cols: Vec<_> = columns.iter()
    .filter(|c| !c.has_forward_ref)
    .collect();

// Phase 2: Forward-referencing columns
let forward_cols: Vec<_> = columns.iter()
    .filter(|c| c.has_forward_ref)
    .collect();

// Execute Phase 1 (forward)
for iteration in start_idx..=end_idx {
    // Evaluate normal_cols in dependency order
}

// Execute Phase 2 (backward)
for iteration in (start_idx..=end_idx).rev() {
    // Evaluate forward_cols
}
```
