#!/bin/bash
# Complete optimization fixes

cd "$(dirname "$0")"

echo "Fixing remaining HashMap and Decimal issues..."

# Fix cache.rs - remove duplicate imports
sed -i '/^use rustc_hash::FxHashMap as HashMap;$/d' src/rlogic/cache.rs
sed -i '/^use rustc_hash::FxHasher;$/d' src/rlogic/cache.rs
sed -i '/^use std::hash::{BuildHasherDefault, Hash, Hasher};$/d' src/rlogic/cache.rs

# Fix evaluator.rs - replace all Decimal operations with f64
cat > /tmp/fix_evaluator.py << 'PYEOF'
import re

with open('src/rlogic/evaluator.rs', 'r') as f:
    content = f.read()

# Replace Decimal patterns
replacements = [
    (r'Decimal::from\((\d+)\)', r'(\1 as f64)'),
    (r'Decimal::from\(([^)]+)\)', r'(\1 as f64)'),
    (r'result / \((\d+) as f64\)', r'result / \1.0'),
    (r'\((\d+) as f64\)', r'\1.0'),
]

for pattern, replacement in replacements:
    content = re.sub(pattern, replacement, content)

with open('src/rlogic/evaluator.rs', 'w') as f:
    f.write(content)
PYEOF

python3 /tmp/fix_evaluator.py

# Fix compiled.rs - remove Decimal import
sed -i '/use rust_decimal/d' src/rlogic/compiled.rs
sed -i 's/n\.parse::<f64>()\.map(|d| d > 0\.0)\.unwrap_or(false)/n.parse::<f64>().unwrap_or(0.0) > 0.0/g' src/rlogic/compiled.rs

echo "Building to check for errors..."
cargo build --release 2>&1 | tee build.log

if [ $? -eq 0 ]; then
    echo "✅ Build successful! Running tests..."
    cargo run --release 2>&1 | tail -20
else
    echo "❌ Build failed. Checking errors..."
    grep "error\[" build.log | head -20
    echo ""
    echo "Manual fixes needed - see build.log"
fi
