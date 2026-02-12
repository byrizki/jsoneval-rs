use super::helpers;
use ahash::{AHashMap, AHashSet};
use serde_json::Value;

/// Index for a table (array of objects)
/// Maps column names to value-to-row-indices lookup
#[derive(Debug, Clone, Default)]
pub struct TableIndex {
    /// Map of column name -> (Map of value hash -> Set of row indices)
    columns: AHashMap<String, AHashMap<String, AHashSet<usize>>>,
    /// Total number of rows in the table
    row_count: usize,
}

impl TableIndex {
    /// Create a new index from a table (array of objects)
    pub fn new(data: &Value) -> Option<Self> {
        let arr = data.as_array()?;
        if arr.is_empty() {
            return None;
        }

        // Only index if array contains objects
        if !arr.iter().any(|v| v.is_object()) {
            return None;
        }

        let row_count = arr.len();
        let mut columns: AHashMap<String, AHashMap<String, AHashSet<usize>>> = AHashMap::new();

        for (row_idx, row) in arr.iter().enumerate() {
            if let Value::Object(obj) = row {
                for (col_name, val) in obj {
                    // Generate hash key for the value
                    if let Some(key) = helpers::scalar_hash_key(val) {
                        // Get or create column index
                        let col_index = columns.entry(col_name.clone()).or_default();
                        
                        // Get or create value entry and add row index
                        col_index.entry(key).or_default().insert(row_idx);
                    }
                }
            }
        }

        Some(Self { columns, row_count })
    }

    /// Look up row indices matching a specific column value
    pub fn lookup(&self, col_name: &str, value: &Value) -> Option<&AHashSet<usize>> {
        let col_index = self.columns.get(col_name)?;
        let key = helpers::scalar_hash_key(value)?;
        col_index.get(&key)
    }

    /// Check if a column is indexed
    pub fn has_column(&self, col_name: &str) -> bool {
        self.columns.contains_key(col_name)
    }

    /// Get total row count
    pub fn len(&self) -> usize {
        self.row_count
    }
    
    pub fn is_empty(&self) -> bool {
        self.row_count == 0
    }
}
