# Global Compiled Logic Store

## Overview

The global compiled logic store allows compiled logic expressions to be shared across different `JSONEval` instances and across FFI boundaries. This implementation uses a thread-safe global cache with ID-based access.

## Architecture

### Key Components

1. **`CompiledLogicId`** - A lightweight identifier (u64) that can be passed across FFI
2. **Global Store** - Thread-safe storage using `DashMap` and atomic counters
3. **Deduplication** - Same logic compiled multiple times returns the same ID (via hashing)

### Module: `compiled_logic_store.rs`

```rust
pub struct CompiledLogicId(u64);

pub fn compile_logic(logic_json: &str) -> Result<CompiledLogicId, String>
pub fn get_compiled_logic(id: CompiledLogicId) -> Option<CompiledLogic>
pub fn get_store_stats() -> CompiledLogicStoreStats
```

## Usage Examples

### Basic Usage

```rust
use json_eval_rs::{JSONEval, CompiledLogicId};
use serde_json::json;

// Create first instance and compile logic
let mut eval1 = JSONEval::new(&schema, None, Some(&data))?;
let logic_id = eval1.compile_logic(r#"{"*": [{"var": "x"}, 2]}"#)?;

// Use the same compiled logic in a different instance
let mut eval2 = JSONEval::new(&schema, None, Some(&data))?;
let result = eval2.run_logic(logic_id, Some(&json!({"x": 10})), None)?;
// result: 20
```

### Cross-Instance Sharing

```rust
// Compile once
let logic_id = eval1.compile_logic(logic_str)?;

// Run on multiple instances with different data
let result1 = eval1.run_logic(logic_id, Some(&data1), None)?;
let result2 = eval2.run_logic(logic_id, Some(&data2), None)?;
let result3 = eval3.run_logic(logic_id, Some(&data3), None)?;
```

### Deduplication

```rust
// Same logic compiled multiple times
let id1 = eval1.compile_logic(r#"{"var": "x"}"#)?;
let id2 = eval2.compile_logic(r#"{"var": "x"}"#)?;

assert_eq!(id1, id2); // Same ID returned!
```

## Benefits

### 1. **FFI-Friendly**
- `CompiledLogicId` is just a `u64` - trivial to pass across FFI
- No need to serialize/deserialize `CompiledLogic`
- Works seamlessly with C, C#, JavaScript, and other languages

### 2. **Memory Efficient**
- Compiled logic is stored once, used many times
- Automatic deduplication via content hashing
- No redundant storage of identical logic

### 3. **Thread-Safe**
- Global store uses `DashMap` for lock-free concurrent access
- Atomic counter for ID generation
- Safe to use from multiple threads

### 4. **Zero-Clone Pattern**
- `run_logic` uses references to data
- No unnecessary cloning during evaluation
- Optimal performance for repeated evaluations

## API Reference

### JSONEval Methods

```rust
impl JSONEval {
    /// Compile logic and return a global ID
    pub fn compile_logic(&self, logic_str: &str) 
        -> Result<CompiledLogicId, String>
    
    /// Run pre-compiled logic by ID
    pub fn run_logic(
        &mut self, 
        logic_id: CompiledLogicId,
        data: Option<&Value>,
        context: Option<&Value>
    ) -> Result<Value, String>
    
    /// Convenience method (compile + run in one step)
    pub fn compile_and_run_logic(
        &mut self,
        logic_str: &str,
        data: Option<&str>,
        context: Option<&str>
    ) -> Result<Value, String>
}
```

### CompiledLogicId

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompiledLogicId(u64);

impl CompiledLogicId {
    pub fn as_u64(&self) -> u64
    pub fn from_u64(id: u64) -> Self
}
```

### Global Functions

```rust
/// Compile logic and store globally
pub fn compile_logic(logic_json: &str) -> Result<CompiledLogicId, String>

/// Retrieve compiled logic by ID
pub fn get_compiled_logic(id: CompiledLogicId) -> Option<CompiledLogic>

/// Get store statistics
pub fn get_store_stats() -> CompiledLogicStoreStats
```

## Implementation Details

### Storage Strategy

```rust
static COMPILED_LOGIC_STORE: Lazy<CompiledLogicStore> = Lazy::new(|| {
    CompiledLogicStore {
        store: DashMap::new(),           // Thread-safe HashMap
        next_id: AtomicU64::new(1),      // Atomic counter
    }
});
```

- **Key**: Hash of logic JSON string (using `AHasher`)
- **Value**: `(CompiledLogicId, CompiledLogic)` tuple
- **ID Generation**: Sequential atomic counter starting from 1

### Hash-Based Deduplication

```rust
let mut hasher = AHasher::default();
logic_json.hash(&mut hasher);
let hash = hasher.finish();

// Check if already compiled
if let Some(entry) = store.get(&hash) {
    return Ok(entry.0); // Return existing ID
}
```

### Thread Safety

- **`DashMap`**: Lock-free concurrent HashMap
- **`AtomicU64`**: Atomic counter for ID generation
- **`Lazy`**: One-time initialization with thread safety

## Performance Characteristics

### Time Complexity

- **`compile_logic`**: O(1) average (hash lookup), O(n) if not cached (compilation)
- **`run_logic`**: O(1) lookup + O(logic_complexity) evaluation
- **Deduplication check**: O(1) average

### Space Complexity

- **Per unique logic**: 1 entry in global store
- **ID overhead**: 8 bytes (u64)
- **Store metadata**: ~40 bytes per entry (DashMap overhead)

## FFI Integration

### C/FFI Example

```c
// C function signature
uint64_t json_eval_compile_logic(void* handle, const char* logic_str);
void* json_eval_run_logic(void* handle, uint64_t logic_id, 
                          const char* data, const char* context);
```

### Usage in Bindings

```rust
#[no_mangle]
pub unsafe extern "C" fn json_eval_compile_logic(
    handle: *mut JSONEvalHandle,
    logic_str: *const c_char,
) -> u64 {
    let eval = &(*handle).inner;
    let logic = CStr::from_ptr(logic_str).to_str().unwrap();
    
    match eval.compile_logic(logic) {
        Ok(id) => id.as_u64(),
        Err(_) => 0, // 0 indicates error
    }
}
```

## Migration Guide

### Old API (Not Recommended for FFI)

```rust
let compiled = CompiledLogic::compile(&logic)?;
let result = evaluator.evaluate(&compiled, &data)?;
```

**Problems**:
- `CompiledLogic` cannot cross FFI boundaries
- Must serialize/deserialize for FFI
- Cannot share between instances

### New API (Recommended)

```rust
// Rust native
let logic_id = eval.compile_logic(logic_str)?;
let result = eval.run_logic(logic_id, Some(&data), None)?;

// FFI-friendly
uint64_t id = json_eval_compile_logic(handle, logic_str);
result_t res = json_eval_run_logic(handle, id, data, context);
```

**Benefits**:
- ID is FFI-safe (u64)
- Logic stored globally
- Can be shared across instances
- Automatic deduplication

## Best Practices

### 1. Compile Once, Run Many

```rust
// ✅ Good - Compile once
let logic_id = eval.compile_logic(logic_str)?;

for data in datasets {
    let result = eval.run_logic(logic_id, Some(&data), None)?;
    // Process result...
}
```

```rust
// ❌ Bad - Compile every time
for data in datasets {
    let result = eval.compile_and_run_logic(logic_str, Some(&data), None)?;
}
```

### 2. Share IDs Across Instances

```rust
// ✅ Good - Share compiled logic
let logic_id = eval1.compile_logic(logic_str)?;

let result1 = eval1.run_logic(logic_id, Some(&data1), None)?;
let result2 = eval2.run_logic(logic_id, Some(&data2), None)?;
```

### 3. Store IDs for Reuse

```rust
struct MyApp {
    validation_logic: CompiledLogicId,
    transform_logic: CompiledLogicId,
}

impl MyApp {
    fn new(eval: &JSONEval) -> Result<Self, String> {
        Ok(Self {
            validation_logic: eval.compile_logic(VALIDATION_LOGIC)?,
            transform_logic: eval.compile_logic(TRANSFORM_LOGIC)?,
        })
    }
    
    fn validate(&self, eval: &mut JSONEval, data: &Value) -> Result<bool, String> {
        let result = eval.run_logic(self.validation_logic, Some(data), None)?;
        Ok(result.as_bool().unwrap_or(false))
    }
}
```

## Statistics and Monitoring

### Get Store Stats

```rust
use json_eval_rs::compiled_logic_store;

let stats = compiled_logic_store::get_store_stats();
println!("Compiled logic count: {}", stats.compiled_count);
println!("Next ID: {}", stats.next_id);
```

### Example Output

```
Compiled logic count: 42
Next ID: 43
```

## Testing

### Unit Tests

```rust
#[test]
fn test_cross_instance_sharing() {
    let schema = json!({"type": "object"}).to_string();
    let data = json!({"x": 10}).to_string();
    
    // Compile with first instance
    let eval1 = JSONEval::new(&schema, None, Some(&data)).unwrap();
    let logic_id = eval1.compile_logic(r#"{"var": "x"}"#).unwrap();
    
    // Use with second instance
    let mut eval2 = JSONEval::new(&schema, None, Some(&data)).unwrap();
    let result = eval2.run_logic(logic_id, Some(&json!({"x": 20})), None).unwrap();
    
    assert_eq!(result, json!(20));
}
```

## Limitations

1. **Global State**: The store is global and persists for the lifetime of the process
2. **No Cleanup**: Compiled logic is never automatically removed (by design for performance)
3. **Memory Usage**: Grows with unique logic expressions compiled
4. **Linear Search for ID**: Getting logic by ID requires linear search (could be optimized with reverse index)

## Future Improvements

1. **Reverse Index**: Add ID → CompiledLogic map for O(1) lookup
2. **LRU Eviction**: Optional LRU cache with size limits
3. **Metrics**: Track hit/miss rates, compilation times
4. **Export/Import**: Serialize store to disk for persistence
5. **Compression**: Compress stored logic to reduce memory usage

## Changelog

### v0.0.24

- ✅ Initial implementation of global compiled logic store
- ✅ Added `CompiledLogicId` type
- ✅ Implemented hash-based deduplication
- ✅ Added thread-safe global storage
- ✅ Updated `compile_logic` and `run_logic` APIs
- ✅ Added comprehensive tests
- ✅ FFI-ready for all bindings
