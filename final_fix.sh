#!/bin/bash
# Final comprehensive fix for all type issues

cd "$(dirname "$0")"

echo "Applying final type fixes..."

# Fix evaluator.rs type issues
cat > /tmp/fix_types.py << 'PYEOF'
import re

with open('src/rlogic/evaluator.rs', 'r') as f:
    content = f.read()

# Fix specific type issues
fixes = [
    # Fix division issues - convert to f64
    (r'days / 365\.25', r'(days as f64) / 365.25'),
    (r'days / (\d+)\.0', r'(days as f64) / \1.0'),
    (r'days / \((\d+) as f64\)', r'(days as f64) / \1.0'),
    
    # Fix total_months type
    (r'\(total_months\)', r'(total_months as i64)'),
    
    # Remove .to_f64() calls on f64 types
    (r'self\.to_f64\(value\)\.to_f64\(\)\.unwrap_or\(0\.0\)', r'self.to_f64(value)'),
    (r'\(num_a - num_b\)\.to_f64\(\)\.unwrap_or\(0\.0\)', r'(num_a - num_b)'),
    
    # Fix .is_zero() calls
    (r'if n\.is_zero\(\)', r'if *n == 0.0'),
    (r'if !val\.is_zero\(\)', r'if val != 0.0'),
    
    # Fix integer literals being passed to f64_to_json
    (r'self\.f64_to_json\((\d+)\)', r'self.f64_to_json(\1.0)'),
]

for pattern, replacement in fixes:
    content = re.sub(pattern, replacement, content)

with open('src/rlogic/evaluator.rs', 'w') as f:
    f.write(content)

print("Type fixes applied!")
PYEOF

python3 /tmp/fix_types.py

echo "Rebuilding..."
cargo build --release 2>&1 | grep -E "^(error\[|warning:.*unused|   Compiling|    Finished)" | head -50
