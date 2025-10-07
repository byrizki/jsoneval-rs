# Performance Optimization TODO

## Current Status: Baseline build & run successful

### Completed

1. **Hash map swap** in `src/rlogic/data_wrapper.rs`, `src/rlogic/cache.rs`, `src/rlogic/compiled.rs` now uses `FxHashMap::default()` everywhere.
2. **Decimal → f64 migration** finished across `src/rlogic/evaluator.rs`; all arithmetic now uses `f64`.
3. **Unused imports cleaned** (e.g., removed `use std::str::FromStr`).
4. **Build & run verification**: `cargo build --release` and `cargo run --release` pass.

### Next Optimizations to Apply

4. **Reduce cloning** in `table_evaluate.rs`:
   - Use `&str` instead of `String` where possible
   - Reuse allocated vectors
   - Use `Cow<str>` for conditional ownership

5. **Parallelize table evaluation** using rayon:
   ```rust
   use rayon::prelude::*;
   
   // In lib.rs evaluate_all():
   tables.par_iter().for_each(|table| {
       evaluate_table(...)
   });
   ```

6. **Cache tuning and custom operator micro-optimizations**:
   - Profile evaluator cache hit rate and explore precomputing keys.
   - Consider specialized fast paths for `VALUEAT`/`INDEXAT`.

6. **Optimize caching**:
   - Add memoization for frequently evaluated expressions
   - Use SmallVec for small arrays to avoid heap allocation

7. **Custom operator optimizations**:
   - VALUEAT: Direct array indexing without bounds checking in hot path
   - INDEXAT: Binary search for sorted lookups
   - Arithmetic: SIMD operations for array operations

### Build Command
```bash
cargo build --release 2>&1 | less
```

### Test Command  
```bash
cargo run --release
```
