use json_eval_rs::JSONEval;
use serde_json::Value;

#[test]
fn test_get_evaluated_schema_resolves_all_static_arrays_minimal() {
    let schema_str = include_str!("fixtures/minimal_form.json");
    let mut schema_json: Value = serde_json::from_str(schema_str).expect("Valid JSON");

    // Programmatically inflate the PREMIUM_RATES array to >10 items so it gets extracted as a $static_array
    if let Some(rates) = schema_json
        .pointer_mut("/$params/references/PREMIUM_RATES")
        .and_then(|v| v.as_array_mut())
    {
        let first_row = rates[0].clone();
        for i in 0..200 {
            let mut new_row = first_row.clone();
            if let Some(age) = new_row.get_mut("AGE") {
                *age = Value::Number(serde_json::Number::from(20 + i));
            }
            rates.push(new_row);
        }
    }

    let inflated_schema_str = schema_json.to_string();

    // We instantiate JSONEval with the minimal_form
    let mut eval =
        JSONEval::new(&inflated_schema_str, None, None).expect("Should initialize successfully");

    // Perform a first evaluation without any input data
    eval.evaluate("{}", None, None, None)
        .expect("Should evaluate successfully");

    // Fetch the evaluated schema (which includes resolving $static_array markers)
    let full_schema = eval.get_evaluated_schema(false);

    // minimal_form.json has a static array extracted during initialization
    // Let's verify it gets properly put back into the evaluated_schema output
    let premium_rates = full_schema.pointer("/$params/references/PREMIUM_RATES");

    assert!(
        premium_rates.is_some(),
        "PREMIUM_RATES should exist in get_evaluated_schema"
    );
    let premium_rates_val = premium_rates.unwrap();

    assert!(
        premium_rates_val.is_array(),
        "PREMIUM_RATES should be an array resolved from $static_array marker, but got {:?}",
        premium_rates_val
    );

    // Additional basic checks to ensure it's the expected array
    let arr = premium_rates_val.as_array().unwrap();
    assert!(
        arr.len() > 200,
        "Should contain the programmatically inflated rows (200+)"
    );

    let first_row = &arr[0];
    assert_eq!(
        first_row.get("AGE").and_then(Value::as_i64),
        Some(20),
        "First row should have AGE: 20"
    );
}
