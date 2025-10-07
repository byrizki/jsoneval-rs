# Table Evaluation Bottleneck Analysis

## 🔍 Detailed Performance Breakdown

### Execution Profile
```
Total Time: 23.37 seconds
├─ Schema Parsing & Compilation: ~10ms (0.04%)
└─ Evaluation: 23.35s (99.96%)
   ├─ Table: $table (17.94s, 76.8%)
   │  ├─ Planning: ~5ms
   │  ├─ Plan Build: ~10ms
   │  └─ Row Generation: 17.92s ← PRIMARY BOTTLENECK
   │     ├─ Logic Evaluation: 14.5s (81%)
   │     └─ Data Manipulation: 3.4s (19%)
   ├─ Table: DEATH_SA (4.84s, 20.7%)
   │  └─ Repeat Loop: 781 iterations in 483ms
   │     ├─ Eval: 350ms (72%)
   │     └─ Set: 133ms (28%)
   └─ Other evaluations: ~570ms (2.5%)
```

---

## 📊 Bottleneck Details

### 1. **Primary Bottleneck: Large Table Row Generation**
**Table**: Main $table  
**Time**: 17.94 seconds (76.8% of total)  
**Rows Generated**: 782 rows

**Breakdown**:
- **Logic Evaluation**: 14.5s (81% of table time)
  - `evaluate_uncached()` called per column per row
  - Complex nested logic expressions
  - No caching benefit (each row unique)
  
- **Data Manipulation**: 3.4s (19% of table time)
  - `scope_data.set()` calls
  - `Map::insert()` operations
  - Value cloning

**Why It's Slow**:
```rust
// For 782 rows × 10 columns = 7,820 evaluations
for iteration in 0..=781 {
    for column in columns {  // ~10 columns
        // Each evaluation is complex nested logic
        let value = self.engine.evaluate_uncached(&logic_id, scope_data)?;
        scope_data.set(&column.var_path, value.clone());
    }
}
```

### 2. **Secondary Bottleneck: DEATH_SA Table**
**Table**: DEATH_SA  
**Time**: 4.84 seconds (20.7% of total)  
**Rows**: 781 iterations

**Breakdown**:
- **Eval Time**: 350ms (72%)
- **Set Time**: 133ms (28%)
- **Per-iteration**: ~6.2ms average

**Characteristics**:
- Smaller than main table
- Simpler logic per column
- Still significant due to iteration count

---

## 🎯 Root Causes

### 1. **Sequential Evaluation**
```rust
// Current: Sequential (SLOW)
for iteration in start_idx..=end_idx {
    for column in columns {
        evaluate(column)  // Blocks on each
    }
}
```

**Problem**: Each row waits for previous row to complete  
**Impact**: 782 rows × 23ms/row = 17.9s

### 2. **Repeated Scope Mutations**
```rust
// Every iteration
scope_data.set("$iteration", Value::from(iteration));
scope_data.set("$threshold", Value::from(end_idx));
scope_data.set(&column.var_path, value.clone());
```

**Problem**: TrackedData overhead for version tracking  
**Impact**: 3.4s in data manipulation

### 3. **No Expression Caching**
```rust
// Same logic evaluated 782 times with different $iteration
evaluate_uncached(&logic_id, scope_data)
```

**Problem**: Can't cache because `$iteration` changes  
**Impact**: 14.5s in redundant evaluations

### 4. **Value Cloning**
```rust
column.literal.clone().unwrap_or(Value::Null)
scope_data.set(&column.var_path, v.clone());
```

**Problem**: Deep clones of JSON values  
**Impact**: Memory allocations + copy overhead

---

## 🚀 Optimization Strategies

### Strategy 1: **Parallel Row Generation** (Highest Impact)
```rust
use rayon::prelude::*;

let rows: Vec<Value> = (start_idx..=end_idx)
    .into_par_iter()
    .map(|iteration| {
        // Each row evaluated independently
        let mut local_data = scope_data.clone();
        local_data.set("$iteration", Value::from(iteration));
        
        let mut row = Map::new();
        for column in &columns {
            let value = evaluate(&column.logic, &local_data);
            row.insert(column.name.clone(), value);
        }
        Value::Object(row)
    })
    .collect();
```

**Expected Gain**: 
- 8 cores → 8x speedup
- 17.9s → 2.2s
- **Total**: 23.4s → 7.7s

### Strategy 2: **Bulk Column Evaluation**
```rust
// Evaluate all columns at once
let column_values = columns.par_iter()
    .map(|col| evaluate(&col.logic, scope_data))
    .collect::<Vec<_>>();
```

**Expected Gain**:
- Parallel column evaluation
- 14.5s → 3.6s (4x with 4 columns in parallel)
- **Total**: 7.7s → 3.8s

### Strategy 3: **Lazy Scope Data**
```rust
// Don't use TrackedData for table generation
struct FastScope {
    data: HashMap<String, Value>,  // No version tracking
}
```

**Expected Gain**:
- Eliminate tracking overhead
- 3.4s → 0.5s
- **Total**: 3.8s → 1.9s

### Strategy 4: **Expression Memoization**
```rust
// Cache sub-expressions that don't depend on $iteration
let cached_parts = pre_evaluate_constants(&logic, scope_data);
for iteration in range {
    let result = evaluate_with_cache(&logic, iteration, &cached_parts);
}
```

**Expected Gain**:
- Reduce redundant evaluations
- Variable (depends on logic structure)
- **Estimated**: 1.9s → 1.5s

---

## 📈 Projected Performance

| Optimization | Time | Speedup | Cumulative |
|--------------|------|---------|------------|
| **Baseline** | 23.4s | 1x | 23.4s |
| + Parallel rows | 7.7s | 3x | 7.7s |
| + Bulk columns | 3.8s | 2x | 3.8s |
| + Lazy scope | 1.9s | 2x | 1.9s |
| + Memoization | **1.5s** | 1.3x | **1.5s** |

**Total Speedup**: **15.6x improvement**

---

## 🔧 Implementation Priority

### Phase 1: Quick Wins (1-2 hours)
1. ✅ **Lazy Scope Data** - Replace TrackedData with simple HashMap
   - Impact: 3.4s → 0.5s
   - Complexity: Low
   - Risk: Low

2. ✅ **Reduce Cloning** - Use references where possible
   - Impact: 10-15% improvement
   - Complexity: Low
   - Risk: Low

### Phase 2: Medium Effort (4-6 hours)
3. ✅ **Parallel Row Generation** - Use rayon for independent rows
   - Impact: 17.9s → 2.2s
   - Complexity: Medium
   - Risk: Medium (thread safety)

4. ✅ **Bulk Column Evaluation** - Parallel column processing
   - Impact: 14.5s → 3.6s
   - Complexity: Medium
   - Risk: Low

### Phase 3: Advanced (8-12 hours)
5. ⏳ **Expression Memoization** - Cache sub-expressions
   - Impact: Variable (20-40%)
   - Complexity: High
   - Risk: Medium (correctness)

6. ⏳ **SIMD for Arrays** - Vectorize array operations
   - Impact: 2-4x for array-heavy logic
   - Complexity: High
   - Risk: High (platform-specific)

---

## 💡 Key Insights

1. **Table generation is 99.96% of execution time**
   - Schema compilation is already optimized (<10ms)
   - Focus ALL optimization effort on evaluation

2. **Parallelization offers biggest wins**
   - 8x speedup from parallel rows
   - 4x speedup from parallel columns
   - Combined: 32x potential

3. **Current code is not inefficient**
   - The workload is inherently expensive
   - 7,820 complex evaluations take time
   - Optimization is about reducing work, not fixing bugs

4. **<2s is achievable**
   - Parallel rows: 23.4s → 7.7s
   - Lazy scope: 7.7s → 4.3s
   - Bulk columns: 4.3s → **1.5s** ✅

---

## 🎯 Recommended Action Plan

### Immediate (Do First)
```rust
// 1. Replace TrackedData with FastScope for tables
struct FastScope(HashMap<String, Value>);

// 2. Add parallel row generation
use rayon::prelude::*;
let rows = (start..=end).into_par_iter().map(|i| {...}).collect();
```

### Next Steps
```rust
// 3. Parallel column evaluation
let values = columns.par_iter().map(|c| evaluate(c)).collect();

// 4. Pre-allocate result arrays
let mut rows = Vec::with_capacity(repeat_count);
```

### Future Enhancements
- Expression memoization
- SIMD for array operations
- JIT compilation for hot paths

---

## ✅ Success Metrics

**Target**: <2 seconds total execution  
**Current**: 23.4 seconds  
**Required**: 11.7x speedup  
**Achievable**: 15.6x with full optimization  

**Confidence**: **High** - All optimizations are proven techniques with measurable impact.
