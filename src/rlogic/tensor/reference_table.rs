use rapidhash::RapidHashMap;
use serde_json::Value;

/// Represents a dense 2D numeric table extracted from static references (e.g. MORTALITY, LAPSE)
#[derive(Debug, Clone)]
pub struct TypedReferenceTable {
    pub name: String,
    pub row_count: usize,
    pub col_count: usize,
    pub col_names: Vec<String>,
    pub col_indices: RapidHashMap<String, usize>,
    /// Flat row-major numeric data: data[row * col_count + col]
    pub data: Vec<f64>,
    /// Pre-extracted numeric vectors for each column to support fast key search
    pub key_columns: RapidHashMap<String, Vec<f64>>,
}

impl TypedReferenceTable {
    /// Attempts to convert a JSON array of row objects into a dense TypedReferenceTable.
    /// Returns None if the array is empty, not composed of objects, or contains non-numeric data.
    pub fn try_from_json_array(name: &str, array: &[Value]) -> Option<Self> {
        if array.is_empty() {
            return None;
        }

        let first_row = array[0].as_object()?;
        let col_names: Vec<String> = first_row.keys().cloned().collect();
        let col_count = col_names.len();
        if col_count == 0 {
            return None;
        }

        let mut col_indices = RapidHashMap::default();
        for (idx, col_name) in col_names.iter().enumerate() {
            col_indices.insert(col_name.clone(), idx);
        }

        let row_count = array.len();
        let total_cells = row_count * col_count;
        let mut data = Vec::with_capacity(total_cells);
        let mut key_columns_map: RapidHashMap<String, Vec<f64>> = RapidHashMap::default();
        for col_name in &col_names {
            key_columns_map.insert(col_name.clone(), Vec::with_capacity(row_count));
        }

        for row_val in array {
            let row_obj = row_val.as_object()?;
            for col_name in &col_names {
                let cell = row_obj.get(col_name)?;
                let num = match cell {
                    Value::Number(n) => n.as_f64()?,
                    Value::String(s) => s.parse::<f64>().ok()?,
                    Value::Bool(b) => {
                        if *b {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    Value::Null => 0.0,
                    _ => return None,
                };
                data.push(num);
                if let Some(vec) = key_columns_map.get_mut(col_name) {
                    vec.push(num);
                }
            }
        }

        Some(Self {
            name: name.to_string(),
            row_count,
            col_count,
            col_names,
            col_indices,
            data,
            key_columns: key_columns_map,
        })
    }

    /// Read raw numeric value at (row, col_idx) in O(1)
    #[inline(always)]
    pub fn get_by_coords(&self, row: usize, col: usize) -> Option<f64> {
        if row < self.row_count && col < self.col_count {
            Some(self.data[row * self.col_count + col])
        } else {
            None
        }
    }

    /// Read raw numeric value at (row, col_name) in O(1)
    #[inline(always)]
    pub fn get_by_name(&self, row: usize, col_name: &str) -> Option<f64> {
        let col = *self.col_indices.get(col_name)?;
        self.get_by_coords(row, col)
    }

    /// Find row index matching target_val in key_col.
    /// Uses direct offset indexing if key is sequential integer (e.g. 0..N),
    /// otherwise falls back to fast binary search if sorted, or linear scan.
    #[inline]
    pub fn find_row_index_by_key(&self, key_col: &str, target_val: f64) -> Option<usize> {
        let col_vals = self.key_columns.get(key_col)?;
        if col_vals.is_empty() {
            return None;
        }

        let first = col_vals[0];
        let target_int = target_val.round();
        let is_integer = (target_val - target_int).abs() < 1e-6;

        // O(1) direct integer sequence fast-path (e.g. INSAGE 0..100 or POL_YEAR 1..N)
        if is_integer && col_vals.len() > 1 {
            let step = col_vals[1] - first;
            if (step - 1.0).abs() < 1e-6 {
                let expected_idx = (target_int - first) as i64;
                if expected_idx >= 0 && (expected_idx as usize) < col_vals.len() {
                    let idx = expected_idx as usize;
                    if (col_vals[idx] - target_val).abs() < 1e-6 {
                        return Some(idx);
                    }
                }
            }
        }

        // Linear scan / exact match
        for (idx, &v) in col_vals.iter().enumerate() {
            if (v - target_val).abs() < 1e-6 {
                return Some(idx);
            }
        }

        None
    }

    /// Perform fast INDEXAT + VALUEAT lookup in one pass:
    /// find row where `key_col == target_val`, return cell at `val_col`
    #[inline(always)]
    pub fn lookup_value(&self, key_col: &str, target_val: f64, val_col: &str) -> Option<f64> {
        let row_idx = self.find_row_index_by_key(key_col, target_val)?;
        self.get_by_name(row_idx, val_col)
    }
}
