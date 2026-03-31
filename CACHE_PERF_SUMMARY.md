# Subform Cache & Evaluation Optimization Summary

This document summarizes the performance optimizations implemented in the `jsoneval-rs` engine to resolve "version explosion" issues and achieve sub-millisecond cache-hit performance for complex subform evaluations.

## 1. The Performance Bottleneck: "Version Explosion"

Prior to these optimizations, the engine suffered from a "cascading invalidation" problem during subform (rider) evaluations:
- **Unconditional Bumping**: `store_cache` and `evaluate_internal` bumped version numbers for `$params` formulas every time they were stored, even if the result value was identical.
- **Per-Rider Inflation**: Since riders share `$params` but are evaluated sequentially, each rider's evaluation would trigger a new set of version bumps. For 3 riders and 400 formulas, this created ~1,200 version increments per pass.
- **Recursive Misses**: Downstream formulas (like `TOTAL_WOP_SA`) saw these version bumps and missed their caches repeatedly, forcing re-evaluation in a loop.

## 2. Implemented Optimizations

### 2.1 Conditional Bumping (T2-First Check)
`store_cache` now performs a **Value-Identity Check** against the Global (Tier 2) cache before bumping versions.
- If the new result matches the existing T2 result, the version bump is skipped.
- **T2 Promotion**: When a rider (Tier 1) stores a `$params` result, it is promoted to T2. This ensures subsequent riders find the value in T2 and skip the bump.

### 2.2 Delta-based Invalidation
We moved from a "has this path ever been bumped?" check to a **"was this path bumped *during this specific diff pass*?"** check.
- **`any_newly_bumped_with_prefix`**: Uses a pre-diff version snapshot to detect only fresh data changes.
- **Benefit**: Prevents historical bumps from earlier riders or parent evaluations from triggering spurious table re-evaluations.

### 2.3 Targeted Table Invalidation
Invalidating all `$params` tables when any rider field changes was the final bottleneck.
- **Dependency Filtering**: The engine now filters the invalidation list to only include tables whose declared `dependencies` intersect with the fields that actually changed in the rider.
- **Result**: Tables like `ILST_TABLE` (global) are no longer invalidated when a rider-specific computed field (like `wop_rider_premi`) changes.

### 2.4 Generation-based Fast Skip
Implemented a high-level "Stable Generation" check in `evaluate_internal`.
- **`eval_generation`**: Incremented only when a formula result is actually re-stored with a new value.
- **`mark_evaluated`**: Stamps the current generation at the end of a successful run.
- **Fast Path**: If `eval_generation == last_evaluated_generation`, the entire batch traversal (hundreds of batches) is skipped.
- **Impact**: Reduces "no-change" evaluation time from ~600ms to **~15ms**.

### 2.5 Data & Snapshot Synchronization
- **Result Write-back**: Subform computed values (e.g., `first_prem`) are now written back to the parent `eval_data` immediately.
- **Snapshot Sync**: The subform's internal `item_snapshot` is synchronized with the parent's Tier 1 cache after every pass.
- **Benefit**: Ensures the next diff sees `old_data == new_data`, enabling the generation skip.

## 3. Performance Results (`zip_scenario` Benchmark)

Measurements taken on a warm release binary:

| Evaluation Stage | Baseline | **Optimized** | Improvement |
|------------------|----------|---------------|-------------|
| **Run 3**: Full Eval (No Changes) | 547ms | **20ms** | **-96%** |
| **Run 3a**: Rider Pass (Cache Hits) | 1,770ms | **46ms** | **-97%** |
| **Run 5**: Full Eval (after WOP) | 598ms | **15ms** | **-97%** |
| **Run 5a**: Rider Pass (after WOP) | 1,840ms | **12ms** | **-99.3%** |

## 4. Key Files
- `src/jsoneval/eval_cache.rs`: Core logic for conditional bumping and generation tracking.
- `src/jsoneval/subform_methods.rs`: Delta detection and targeted table invalidation.
- `src/jsoneval/evaluate.rs`: External API generation skip.
- `src/jsoneval/dependents.rs`: Result write-back and snapshot sync logic.
