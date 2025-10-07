use json_eval_rs::JSONEval;
use serde_json::json;

#[test]
fn collects_evaluation_entries() {
    let schema_json = json!({
        "$evaluation": {"logic": "root"},
        "$table": {"name": "root"},
        "properties": {
            "field": {
                "$evaluation": {"logic": {"var": "field"}},
                "$table": ["row1", "row2"]
            },
            "nested": {
                "items": [{
                    "$table": {"name": "nested-items"},
                    "$evaluation": {
                        "logic": {"$ref": "#/properties/field"}
                    }
                }]
            }
        }
    });
    let schema = schema_json.to_string();

    let eval = JSONEval::new(&schema, None, None).expect("schema should parse");
    let keys: Vec<_> = eval.evaluations.keys().cloned().collect();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"#/$evaluation".to_string()));
    assert!(keys.contains(&"#/properties/field/$evaluation".to_string()));
    assert!(keys.contains(&"#/properties/nested/items/0/$evaluation".to_string()));

    // Logic IDs stored for evaluations
    for key in keys {
        assert!(eval.evaluations.contains_key(&key));
    }

    // Tables collected
    assert!(eval.tables.contains_key("#/$table"));
    assert!(eval.tables.contains_key("#/properties/field/$table"));
    assert!(eval
        .tables
        .contains_key("#/properties/nested/items/0/$table"));

    // Dependencies captured and normalized
    let deps = eval
        .dependencies
        .get("#/properties/field/$evaluation")
        .expect("field evaluation should have dependencies");
    assert!(deps.contains(&"field".to_string()));

    let nested_deps = eval
        .dependencies
        .get("#/properties/nested/items/0/$evaluation")
        .expect("nested evaluation should have dependencies");
    assert!(nested_deps.contains(&"field".to_string()));

    // Sorted evaluations should be populated
    assert!(!eval.sorted_evaluations.is_empty());
    println!("Sorted evaluations: {:?}", eval.sorted_evaluations);
}

#[test]
fn reload_schema_updates_evaluations() {
    let schema1 = json!({
        "properties": {"a": {"$evaluation": {"logic": "a"}}}
    })
    .to_string();
    let schema2 = json!({
        "properties": {"b": {"$evaluation": {"logic": {"var": "b"}}}}
    })
    .to_string();

    let mut eval = JSONEval::new(&schema1, None, None).expect("first schema parses");
    assert_eq!(eval.evaluations.len(), 1);
    assert!(eval.evaluations.contains_key("#/properties/a/$evaluation"));

    eval.reload_schema(&schema2, None, None)
        .expect("reload should succeed");

    assert_eq!(eval.evaluations.len(), 1);
    assert!(eval.evaluations.contains_key("#/properties/b/$evaluation"));
    assert!(eval.dependencies.contains_key("#/properties/b/$evaluation"));

    // Sorted evaluations should be updated
    assert!(!eval.sorted_evaluations.is_empty());
}

#[test]
fn topological_sort_with_tables() {
    let schema_json = json!({
        "properties": {
            "tableA": {
                "$table": {"name": "A"},
                "$evaluation": {"logic": {"var": "input"}},
                "columns": {
                    "col1": {
                        "$evaluation": {"logic": {"$ref": "#/properties/tableB"}}
                    }
                }
            },
            "tableB": {
                "$table": {"name": "B"},
                "$evaluation": {"logic": "base"}
            },
            "standalone": {
                "$evaluation": {"logic": {"$ref": "#/properties/tableA"}}
            }
        }
    });
    let schema = schema_json.to_string();

    let eval = JSONEval::new(&schema, None, None).expect("schema should parse");

    // Should have sorted evaluations with proper ordering
    assert!(!eval.sorted_evaluations.is_empty());

    // Table paths should be included in sorted list
    let sorted_vec: Vec<_> = eval.sorted_evaluations.iter().cloned().collect();
    println!("Sorted order: {:?}", sorted_vec);

    // tableB should come before tableA (dependency order)
    // Note: sorted list contains table paths like "#/properties/tableB"
    let table_b_pos = sorted_vec.iter().position(|x| x == "#/properties/tableB");
    let table_a_pos = sorted_vec.iter().position(|x| x == "#/properties/tableA");

    assert!(table_b_pos.is_some(), "tableB should be in sorted list");
    assert!(table_a_pos.is_some(), "tableA should be in sorted list");

    if let (Some(b_pos), Some(a_pos)) = (table_b_pos, table_a_pos) {
        assert!(
            b_pos < a_pos,
            "tableB (pos {}) should be sorted before tableA (pos {})",
            b_pos,
            a_pos
        );
    }

    // standalone should come after tableA since it depends on it
    let standalone_pos = sorted_vec
        .iter()
        .position(|x| x == "#/properties/standalone/$evaluation");
    if let (Some(a_pos), Some(s_pos)) = (table_a_pos, standalone_pos) {
        assert!(a_pos < s_pos, "tableA should be sorted before standalone");
    }
}

#[test]
fn test_decimal_precision() {
    let schema_json = json!({
        "$params": {
            "a": {
                "$evaluation": {
                    "-": [
                      1,
                      {
                        "*": [
                          0.003,
                          1
                        ]
                      }
                    ]
                  }
            }
        },
        "data": {
            "properties": {
                "result": {
                    "$evaluation": {
                        "$ref": "#/$params/a"
                    }
                }
            }
        }
    });
    let schema = schema_json.to_string();
    let data = json!({});

    let mut eval = JSONEval::new(&schema, None, None).expect("schema should parse");

    // Verify the evaluation was parsed
    assert!(
        eval.evaluations.contains_key("#/data/properties/result"),
        "Should have result evaluation"
    );
    assert!(
        eval.evaluations.contains_key("#/$params/a"),
        "Should have params.a evaluation"
    );

    // Evaluate the schema
    let result = eval
        .evaluate(&data.to_string(), None)
        .expect("evaluation should succeed");

    println!(
        "Evaluated schema: {}",
        serde_json::to_string_pretty(&result).unwrap()
    );

    // The evaluated schema returns the data with evaluated values
    // Check that the result is 0.997 (1 - 0.003 * 1)
    let result_value = result
        .get("a")
        .expect("'a' field should exist in evaluated result");
    let result_num = result_value.as_f64().expect("'a' should be a number");

    println!("Result value 'a': {}", result_num);
    assert!(
        (result_num - 0.997).abs() < 1e-10,
        "Expected 0.997, got {}. This proves that 1 - (0.003 * 1) = 0.997 with decimal precision!",
        result_num
    );
}
