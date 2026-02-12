use json_eval_rs::rlogic::RLogic;
use serde_json::json;

#[test]
fn test_match_with_index() {
    let rlogic = RLogic::new();
    let data = json!({
        "my_table": [
            {"id": 1, "val": "a"},
            {"id": 2, "val": "b"},
            {"id": 3, "val": "a"}
        ]
    });

    // 1. Run without index
    let logic_match = json!({
        "match": [
            {"var": "my_table"},
            "a", "val"
        ]
    });
    
    let result = rlogic.evaluate(&logic_match, &data).unwrap();
    assert_eq!(result, json!(0)); // First "a" is at index 0

    // 2. Index the table
    rlogic.index_table("my_table", &data["my_table"]);

    // 3. Run with index (should be same result)
    let result_indexed = rlogic.evaluate(&logic_match, &data).unwrap();
    assert_eq!(result_indexed, json!(0));

    // 4. Test multiple conditions
    let logic_match_multi = json!({
        "match": [
            {"var": "my_table"},
            "a", "val",
            3, "id"
        ]
    });
    // Should match row 2 (id=3, val=a)
    let result_multi = rlogic.evaluate(&logic_match_multi, &data).unwrap();
    assert_eq!(result_multi, json!(2));
}

#[test]
fn test_indexat_with_index() {
    let rlogic = RLogic::new();
    let data = json!({
        "my_table": [
            {"id": 10, "val": "x"},
            {"id": 20, "val": "y"},
            {"id": 30, "val": "z"}
        ]
    });

    rlogic.index_table("my_table", &data["my_table"]);

    // IndexAt(lookup_val, table, field)
    let logic_indexat = json!({
        "indexat": [
            20,
            {"var": "my_table"},
            "id"
        ]
    });

    let result = rlogic.evaluate(&logic_indexat, &data).unwrap();
    assert_eq!(result, json!(1)); // id=20 is at index 1

    // Test not found
    let logic_not_found = json!({
        "indexat": [
            99,
            {"var": "my_table"},
            "id"
        ]
    });
    let result_nf = rlogic.evaluate(&logic_not_found, &data).unwrap();
    assert_eq!(result_nf, json!(-1));
}

#[test]
fn test_index_performance_proxy() {
    // This is a proxy test to ensure large tables don't crash and logic works
    let rlogic = RLogic::new();
    let size = 10_000;
    let mut rows = Vec::with_capacity(size);
    for i in 0..size {
        rows.push(json!({"id": i, "k": "v"}));
    }
    let data = json!({ "large_table": rows });

    // Index it
    rlogic.index_table("large_table", &data["large_table"]);

    let target_id = size - 1;
    let logic = json!({
        "match": [
            {"var": "large_table"},
            target_id, "id"
        ]
    });

    let result = rlogic.evaluate(&logic, &data).unwrap();
    assert_eq!(result, json!((target_id)));
}
