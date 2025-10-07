# Table Calculation Redesign Proposals

## 🎯 Problem Statement

Current approach evaluates **7,820 logic expressions sequentially** taking ~32 seconds.

**Root causes**:
1. Sequential row-by-row evaluation
2. Column dependencies prevent parallelization
3. Repeated scope mutations (TrackedData overhead)
4. No expression reuse across iterations

---

## 🚀 Proposal 1: Compiled Table Templates (Recommended)

### Concept
Pre-compile table logic into optimized templates that can be evaluated in bulk.

### Architecture
```rust
struct CompiledTableTemplate {
    // Pre-analyzed column dependencies
    dependency_graph: Vec<Vec<usize>>,  // Which columns depend on which
    
    // Compiled logic for each column
    column_logic: Vec<LogicId>,
    
    // Execution plan (topologically sorted)
    execution_order: Vec<usize>,
    
    // Parallelizable batches
    parallel_batches: Vec<Vec<usize>>,  // Columns that can run in parallel
}

impl CompiledTableTemplate {
    fn evaluate_row(&self, iteration: i64, base_data: &Value) -> Map<String, Value> {
        let mut row = Map::new();
        
        // Execute in batches (parallel within batch)
        for batch in &self.parallel_batches {
            let results: Vec<_> = batch.par_iter()
                .map(|&col_idx| {
                    let logic = &self.column_logic[col_idx];
                    // Evaluate with row context
                    evaluate_with_iteration(logic, base_data, iteration, &row)
                })
                .collect();
            
            // Merge results into row
            for (col_idx, value) in batch.iter().zip(results) {
                row.insert(self.column_names[*col_idx].clone(), value);
            }
        }
        
        row
    }
    
    fn evaluate_table(&self, start: i64, end: i64, base_data: &Value) -> Vec<Value> {
        // Parallel row generation
        (start..=end)
            .into_par_iter()
            .map(|iteration| {
                Value::Object(self.evaluate_row(iteration, base_data))
            })
            .collect()
    }
}
```

### Benefits
- ✅ **Parallel column evaluation** (independent columns in same batch)
- ✅ **Parallel row generation** (rows are independent)
- ✅ **No TrackedData overhead** (immutable base_data)
- ✅ **Pre-computed dependencies** (one-time analysis)

### Expected Performance
```
Current: 32s (sequential)
With parallel columns (4 batches): 8s (4x)
With parallel rows (8 cores): 1s (8x)
Total: 32s → 1s (32x improvement)
```

---

## 🚀 Proposal 2: Vectorized Evaluation

### Concept
Evaluate entire columns at once using vectorized operations.

### Architecture
```rust
struct VectorizedColumn {
    logic: LogicId,
    dependencies: Vec<String>,
}

impl VectorizedColumn {
    fn evaluate_vector(
        &self,
        iterations: &[i64],
        base_data: &Value,
        previous_columns: &HashMap<String, Vec<Value>>
    ) -> Vec<Value> {
        // SIMD-friendly bulk evaluation
        iterations.par_iter()
            .map(|&iteration| {
                let mut ctx = base_data.clone();
                ctx["$iteration"] = iteration.into();
                
                // Inject previous column values for this iteration
                for (col_name, col_values) in previous_columns {
                    ctx[col_name] = col_values[iteration as usize].clone();
                }
                
                evaluate(&self.logic, &ctx)
            })
            .collect()
    }
}

fn evaluate_table_vectorized(
    columns: &[VectorizedColumn],
    start: i64,
    end: i64,
    base_data: &Value
) -> Vec<Value> {
    let iterations: Vec<i64> = (start..=end).collect();
    let mut column_results = HashMap::new();
    
    // Evaluate columns in dependency order
    for column in columns {
        let values = column.evaluate_vector(&iterations, base_data, &column_results);
        column_results.insert(column.name.clone(), values);
    }
    
    // Transpose: column-major → row-major
    (0..iterations.len())
        .map(|row_idx| {
            let mut row = Map::new();
            for (col_name, col_values) in &column_results {
                row.insert(col_name.clone(), col_values[row_idx].clone());
            }
            Value::Object(row)
        })
        .collect()
}
```

### Benefits
- ✅ **Column-wise parallelization**
- ✅ **Better cache locality** (process all iterations of one column together)
- ✅ **SIMD opportunities** (vectorize within column)
- ✅ **Reduced overhead** (fewer scope mutations)

### Expected Performance
```
Current: 32s
With vectorization: 8-12s (3-4x improvement)
```

---

## 🚀 Proposal 3: Lazy Evaluation with Memoization

### Concept
Only evaluate what's needed and cache sub-expressions.

### Architecture
```rust
struct LazyTable {
    rows: Vec<LazyRow>,
}

struct LazyRow {
    iteration: i64,
    columns: HashMap<String, LazyColumn>,
}

enum LazyColumn {
    NotEvaluated(LogicId),
    Evaluating,
    Evaluated(Value),
}

impl LazyRow {
    fn get_column(&mut self, name: &str, base_data: &Value) -> Result<&Value, String> {
        match self.columns.get_mut(name) {
            Some(LazyColumn::Evaluated(v)) => Ok(v),
            Some(LazyColumn::NotEvaluated(logic_id)) => {
                // Evaluate on demand
                let mut ctx = base_data.clone();
                ctx["$iteration"] = self.iteration.into();
                
                let value = evaluate(logic_id, &ctx)?;
                self.columns.insert(name.to_string(), LazyColumn::Evaluated(value));
                
                if let Some(LazyColumn::Evaluated(v)) = self.columns.get(name) {
                    Ok(v)
                } else {
                    unreachable!()
                }
            }
            _ => Err("Circular dependency".into())
        }
    }
}
```

### Benefits
- ✅ **Skip unused columns** (if schema has conditionals)
- ✅ **Memoization** (evaluate once, use many times)
- ✅ **Detect circular dependencies**

### Expected Performance
```
Current: 32s
If 30% columns unused: 22s (1.4x improvement)
```

---

## 🚀 Proposal 4: Expression Factoring

### Concept
Extract common sub-expressions and evaluate them once.

### Architecture
```rust
struct FactoredExpression {
    // Common sub-expressions shared across columns
    common_exprs: Vec<(String, LogicId)>,
    
    // Column-specific logic (references common exprs)
    column_logic: Vec<LogicId>,
}

fn factor_table_expressions(columns: &[ColumnPlan]) -> FactoredExpression {
    // Analyze all column logic to find common patterns
    let mut common = Vec::new();
    let mut expr_map = HashMap::new();
    
    for column in columns {
        // Find repeated sub-expressions
        let sub_exprs = extract_sub_expressions(&column.logic);
        for expr in sub_exprs {
            *expr_map.entry(expr).or_insert(0) += 1;
        }
    }
    
    // Extract expressions used 2+ times
    for (expr, count) in expr_map {
        if count >= 2 {
            common.push(("$common_{}".format(common.len()), expr));
        }
    }
    
    // Rewrite column logic to reference common expressions
    // ...
}

fn evaluate_factored_row(
    factored: &FactoredExpression,
    iteration: i64,
    base_data: &Value
) -> Map<String, Value> {
    let mut ctx = base_data.clone();
    ctx["$iteration"] = iteration.into();
    
    // Evaluate common expressions once
    for (name, logic) in &factored.common_exprs {
        let value = evaluate(logic, &ctx);
        ctx[name] = value;
    }
    
    // Evaluate columns (can now reference common exprs)
    let mut row = Map::new();
    for (col_name, logic) in &factored.column_logic {
        row.insert(col_name.clone(), evaluate(logic, &ctx));
    }
    
    row
}
```

### Benefits
- ✅ **Reduce redundant evaluations**
- ✅ **Works with existing architecture**
- ✅ **Automatic optimization**

### Expected Performance
```
Current: 32s
If 40% expressions are common: 19s (1.7x improvement)
```

---

## 🎯 Recommended Approach: **Hybrid Strategy**

Combine multiple proposals for maximum impact:

### Phase 1: Quick Wins (1-2 days)
1. **Expression Factoring** (Proposal 4)
   - Extract common sub-expressions
   - Expected: 32s → 19s

2. **Remove TrackedData from tables**
   - Use plain HashMap for row context
   - Expected: 19s → 16s

### Phase 2: Architectural Change (3-5 days)
3. **Compiled Table Templates** (Proposal 1)
   - Pre-analyze dependencies
   - Parallel column batches
   - Expected: 16s → 4s

4. **Parallel Row Generation**
   - Use Arc<RwLock<>> for thread safety
   - Expected: 4s → **0.5-1s**

### Phase 3: Advanced (Optional, 1-2 weeks)
5. **Vectorized Evaluation** (Proposal 2)
   - Column-major processing
   - SIMD for numeric operations
   - Expected: Further 2-3x improvement

---

## 📊 Performance Projection

| Phase | Optimization | Time | Cumulative |
|-------|-------------|------|------------|
| **Baseline** | Current | 32s | 32s |
| Phase 1.1 | Expression factoring | 19s | 19s |
| Phase 1.2 | Remove TrackedData | 16s | 16s |
| Phase 2.1 | Compiled templates | 4s | 4s |
| Phase 2.2 | Parallel rows | **1s** | **1s** |
| Phase 3 | Vectorization | **0.3-0.5s** | **0.3-0.5s** |

**Target <2s**: ✅ **Achievable in Phase 2**

---

## 🔧 Implementation Priority

### Start Here (Highest ROI)
```rust
// 1. Expression Factoring
fn optimize_table_logic(table: &TablePlan) -> OptimizedTable {
    let common_exprs = extract_common_expressions(&table.columns);
    let factored_columns = rewrite_with_common_exprs(&table.columns, &common_exprs);
    OptimizedTable { common_exprs, factored_columns }
}

// 2. Lightweight Row Context
struct RowContext {
    data: HashMap<String, Value>,  // No version tracking
}

impl RowContext {
    fn evaluate_column(&mut self, logic: &LogicId, engine: &RLogic) -> Value {
        engine.evaluate_raw(logic, &Value::Object(self.data.clone()))
            .unwrap_or(Value::Null)
    }
}
```

### Then Add (Medium Effort)
```rust
// 3. Dependency Analysis
struct ColumnDependencies {
    independent: Vec<usize>,      // Can run in parallel
    batches: Vec<Vec<usize>>,     // Dependency-ordered batches
}

fn analyze_dependencies(columns: &[Column]) -> ColumnDependencies {
    // Build dependency graph
    // Topological sort
    // Group independent columns
}
```

### Finally (High Impact)
```rust
// 4. Parallel Execution
fn evaluate_table_parallel(
    template: &CompiledTableTemplate,
    range: RangeInclusive<i64>,
    base_data: Arc<Value>
) -> Vec<Value> {
    range.into_par_iter()
        .map(|iteration| {
            template.evaluate_row(iteration, &base_data)
        })
        .collect()
}
```

---

## 💡 Key Insights

1. **Don't optimize the evaluator** - It's already fast
2. **Optimize the table generation strategy** - This is where 99% of time is spent
3. **Parallel > Sequential** - 8 cores = 8x speedup potential
4. **Batch > Individual** - Reduce per-operation overhead
5. **Compile > Interpret** - Pre-compute what you can

---

## 🎯 Recommendation

**Implement Hybrid Strategy Phase 1 + 2**:
- Expression factoring (easy, 1.7x gain)
- Compiled templates (medium, 4x gain)
- Parallel rows (medium, 8x gain)

**Total**: 32s → **1s** (32x improvement)

**Effort**: 3-5 days  
**Risk**: Medium (requires refactoring)  
**Payoff**: ✅ **Achieves <2s target**

Would you like me to start implementing any of these proposals?
