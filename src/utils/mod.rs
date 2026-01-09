use serde_json::Value;
use std::cell::RefCell;


// Timing infrastructure
thread_local! {
    static TIMING_ENABLED: RefCell<bool> = RefCell::new(std::env::var("JSONEVAL_TIMING").is_ok());
    static TIMING_DATA: RefCell<Vec<(String, std::time::Duration)>> = RefCell::new(Vec::new());
}

/// Check if timing is enabled
#[inline]
pub fn is_timing_enabled() -> bool {
    TIMING_ENABLED.with(|enabled| *enabled.borrow())
}

/// Enable timing programmatically (in addition to JSONEVAL_TIMING environment variable)
pub fn enable_timing() {
    TIMING_ENABLED.with(|enabled| {
        *enabled.borrow_mut() = true;
    });
}

/// Disable timing
pub fn disable_timing() {
    TIMING_ENABLED.with(|enabled| {
        *enabled.borrow_mut() = false;
    });
}

/// Record timing data
#[inline]
pub fn record_timing(label: &str, duration: std::time::Duration) {
    if is_timing_enabled() {
        TIMING_DATA.with(|data| {
            data.borrow_mut().push((label.to_string(), duration));
        });
    }
}

/// Print timing summary
pub fn print_timing_summary() {
    if !is_timing_enabled() {
        return;
    }

    TIMING_DATA.with(|data| {
        let timings = data.borrow();
        if timings.is_empty() {
            return;
        }

        eprintln!("\n📊 Timing Summary (JSONEVAL_TIMING enabled)");
        eprintln!("{}", "=".repeat(60));

        let mut total = std::time::Duration::ZERO;
        for (label, duration) in timings.iter() {
            eprintln!("{:40} {:>12?}", label, duration);
            total += *duration;
        }

        eprintln!("{}", "=".repeat(60));
        eprintln!("{:40} {:>12?}", "TOTAL", total);
        eprintln!();
    });
}

/// Clear timing data
pub fn clear_timing_data() {
    TIMING_DATA.with(|data| {
        data.borrow_mut().clear();
    });
}

/// Macro for timing a block of code
#[macro_export]
macro_rules! time_block {
    ($label:expr, $block:block) => {{
        let _start = if $crate::utils::is_timing_enabled() {
            Some(std::time::Instant::now())
        } else {
            None
        };
        let result = $block;
        if let Some(start) = _start {
            $crate::utils::record_timing($label, start.elapsed());
        }
        result
    }};
}

/// Clean floating point noise from JSON values
/// Converts values very close to zero (< 1e-10) to exactly 0
pub fn clean_float_noise(value: Value) -> Value {
    const EPSILON: f64 = 1e-10;

    match value {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.abs() < EPSILON {
                    // Clean near-zero values to exactly 0
                    Value::Number(serde_json::Number::from(0))
                } else if f.fract().abs() < EPSILON {
                    // Clean whole numbers: 33.0 → 33
                    Value::Number(serde_json::Number::from(f.round() as i64))
                } else {
                    Value::Number(n)
                }
            } else {
                Value::Number(n)
            }
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(clean_float_noise).collect()),
        Value::Object(obj) => Value::Object(
            obj.into_iter()
                .map(|(k, v)| (k, clean_float_noise(v)))
                .collect(),
        ),
        _ => value,
    }
}
