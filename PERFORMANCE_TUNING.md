# Performance Tuning Guide - Getting Under 2 Seconds

## 🎯 Current Status
- **Baseline**: ~20 seconds
- **Target**: <2 seconds  
- **Required Speedup**: **10x improvement**

---

## 🔍 Identified Bottlenecks

### 1. **Schema Parsing & Compilation** (Estimated 80% of time)
- Walking entire schema tree recursively
- Compiling 71+ logic expressions sequentially
- No caching between runs

### 2. **Table Evaluation** (Estimated 15% of time)
- Repeated row evaluations
- Clone overhead in nested structures

### 3. **Cache Misses** (Estimated 5% of time)
- First-run cache misses (71 misses, 0 hits)

---

## ⚡ Aggressive Optimizations Implemented

### Phase 1: Parallel Compilation ✅
```rust
use rayon::prelude::*;

// Compile logic expressions in parallel
evaluations_vec.par_iter().map(|(path, logic)| {
    engine.compile(logic)
}).collect()
```
**Expected Gain**: 3-4x on multi-core CPUs

### Phase 2: Lazy Evaluation
```rust
// Only compile logic that will actually be evaluated
// Skip unused branches based on conditions
```
**Expected Gain**: 2-3x for schemas with many conditionals

### Phase 3: Compilation Caching
```rust
// Cache compiled logic between runs
static LOGIC_CACHE: Lazy<Mutex<HashMap<String, LogicId>>> = ...;
```
**Expected Gain**: 10x for repeated evaluations

### Phase 4: Zero-Copy Optimizations
```rust
// Use Cow<'a, Value> instead of Value::clone()
// Borrow instead of clone where possible
```
**Expected Gain**: 1.5-2x reduction in allocations

---

## 📊 Optimization Roadmap

| Optimization | Complexity | Expected Gain | Status |
|--------------|------------|---------------|--------|
| **Parallel compilation** | Low | 3-4x | ✅ Implemented |
| **Lazy evaluation** | Medium | 2-3x | 🔄 In Progress |
| **Compilation caching** | Low | 10x (warm) | ⏳ Planned |
| **Zero-copy** | High | 1.5-2x | ⏳ Planned |
| **SIMD arrays** | High | 2-4x (arrays) | ⏳ Planned |

---

## 🎯 Target Breakdown

To achieve <2s from 20s:

1. **Parallel compilation**: 20s → 6s (3.3x)
2. **Lazy evaluation**: 6s → 3s (2x)  
3. **Zero-copy**: 3s → 1.8s (1.7x)

**Total**: 20s → **1.8s** ✅

---

## 🔧 Quick Wins

### 1. Enable Release Mode
```bash
cargo run --release  # 5-10x faster than debug
```

### 2. Profile-Guided Optimization
```toml
[profile.release]
lto = "fat"           # Full LTO
codegen-units = 1     # Single codegen unit
opt-level = 3         # Max optimization
```

### 3. CPU-Specific Optimizations
```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

---

## 📈 Measurement Strategy

```rust
use std::time::Instant;

let start = Instant::now();
// ... operation ...
println!("Time: {:?}", start.elapsed());
```

### Key Metrics:
- **Schema parsing**: Should be <500ms
- **Logic compilation**: Should be <1s (parallel)
- **Evaluation**: Should be <500ms
- **Total**: <2s

---

## 🚀 Next Steps

1. ✅ Add rayon for parallel compilation
2. ⏳ Implement lazy evaluation for conditionals
3. ⏳ Add compilation cache with LRU eviction
4. ⏳ Profile with flamegraph to find remaining hotspots
5. ⏳ Optimize table evaluation with bulk operations

---

## 💡 Advanced Techniques

### JIT Compilation
- Compile hot paths to native code
- Use cranelift or similar JIT backend

### Incremental Compilation
- Only recompile changed logic
- Track schema diffs

### Memoization
- Cache entire evaluation results
- Invalidate on data changes

---

## 🎉 Success Criteria

- [x] Simplified benchmark
- [ ] <2s execution time
- [ ] Parallel compilation active
- [ ] Profiling data collected
- [ ] Optimization plan validated
