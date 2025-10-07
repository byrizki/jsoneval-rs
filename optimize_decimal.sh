#!/bin/bash
# Script to replace remaining Decimal usage with f64

cd "$(dirname "$0")"

# Replace in evaluator.rs
sed -i 's/Decimal::ZERO/0.0_f64/g' src/rlogic/evaluator.rs
sed -i 's/Decimal::ONE/1.0_f64/g' src/rlogic/evaluator.rs
sed -i 's/Decimal::from(/(/g' src/rlogic/evaluator.rs
sed -i 's/self\.to_decimal(/self.to_f64(/g' src/rlogic/evaluator.rs
sed -i 's/self\.decimal_to_json(/self.f64_to_json(/g' src/rlogic/evaluator.rs
sed -i 's/Decimal::from_str("[^"]*")\.unwrap()/365.25/g' src/rlogic/evaluator.rs

# Replace in compiled.rs
sed -i 's/use rust_decimal::Decimal;//g' src/rlogic/compiled.rs
sed -i 's/Decimal::from_str(n)\.map(|d| d > Decimal::ZERO)\.unwrap_or(false)/n.parse::<f64>().map(|d| d > 0.0).unwrap_or(false)/g' src/rlogic/compiled.rs

# Replace hashbrown with rustc-hash HashMap
sed -i 's/use hashbrown::HashMap;/use rustc_hash::FxHashMap as HashMap;/g' src/rlogic/compiled.rs
sed -i 's/use hashbrown::HashMap;/use rustc_hash::FxHashMap as HashMap;/g' src/rlogic/data_wrapper.rs
sed -i 's/use hashbrown::HashMap;/use rustc_hash::FxHashMap as HashMap;/g' src/rlogic/cache.rs

echo "Decimal to f64 replacement complete"
echo "Run: cargo build --release to verify"
