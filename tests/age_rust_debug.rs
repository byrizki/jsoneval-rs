use chrono::{Datelike, Utc};
use json_eval_rs::JSONEval;
use serde_json::json;

fn age_on_current_utc_date(birth_year: i32, birth_month: u32, birth_day: u32) -> i32 {
    let today = Utc::now().date_naive();
    let birthday_reached = (today.month(), today.day()) >= (birth_month, birth_day);
    today.year() - birth_year - i32::from(!birthday_reached)
}

#[test]
fn test_age_calculation_from_dob() {
    let schema = json!({
        "type": "object",
        "properties": {
            "form": {
                "type": "object",
                "properties": {
                    "user": {
                        "type": "object",
                        "properties": {
                            "dob": {
                                "type": "string",
                                "dependents": [
                                  {
                                    "$ref": "#/form/properties/user/properties/age",
                                    "value": {
                                      "$evaluation": {
                                        "-": [
                                          {
                                            "LEFT": [
                                              {
                                                "NOW": []
                                              },
                                              4
                                            ]
                                          },
                                          {
                                            "LEFT": [
                                              {
                                                "$ref": "$value"
                                              },
                                              4
                                            ]
                                          },
                                          {
                                            "if": [
                                              {
                                                "or": [
                                                  {
                                                    "<": [
                                                      {
                                                        "MID": [
                                                          {
                                                            "NOW": []
                                                          },
                                                          6,
                                                          2
                                                        ]
                                                      },
                                                      {
                                                        "MID": [
                                                          {
                                                            "$ref": "$value"
                                                          },
                                                          6,
                                                          2
                                                        ]
                                                      }
                                                    ]
                                                  },
                                                  {
                                                    "and": [
                                                      {
                                                        "==": [
                                                          {
                                                            "MID": [
                                                              {
                                                                "NOW": []
                                                              },
                                                              6,
                                                              2
                                                            ]
                                                          },
                                                          {
                                                            "MID": [
                                                              {
                                                                "$ref": "$value"
                                                              },
                                                              6,
                                                              2
                                                            ]
                                                          }
                                                        ]
                                                      },
                                                      {
                                                        "<": [
                                                          {
                                                            "MID": [
                                                              {
                                                                "NOW": []
                                                              },
                                                              9,
                                                              2
                                                            ]
                                                          },
                                                          {
                                                            "MID": [
                                                              {
                                                                "$ref": "$value"
                                                              },
                                                              9,
                                                              2
                                                            ]
                                                          }
                                                        ]
                                                      }
                                                    ]
                                                  }
                                                ]
                                              },
                                              1,
                                              0
                                            ]
                                          }
                                        ]
                                      }
                                    }
                                  }
                                ]
                            },
                            "age": { "type": "number" }
                        }
                    }
                }
            }
        }
    });

    let mut data = json!({
        "form": {
            "user": {
                "dob": "",
                "age": 0
            }
        }
    });

    // Create JSONEval instance
    let mut eval = JSONEval::new(&schema.to_string(), None, Some(&data.to_string()))
        .expect("Failed to create JSONEval");

    // Initialize dependencies
    eval.evaluate(&data.to_string(), None, None, None)
        .expect("Initial evaluate failed");

    // Change DOB to trigger dependents
    data["form"]["user"]["dob"] = "1986-05-31".into();

    // Trigger evaluate_dependents on trigger_field
    let deps_result = eval
        .evaluate_dependents(
            &["#/properties/form/properties/user/properties/dob".to_string()],
            Some(&data.to_string()),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    println!("Evaluation dependents result: {}", deps_result);

    let deps_array = deps_result.as_array().expect("deps should be array");
    let age_dep = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("form.user.age"));

    assert!(age_dep.is_some(), "age should be updated");
    let age_val = age_dep.unwrap().get("value").unwrap();
    println!("Computed age value: {}", age_val);
}

#[test]
fn test_age_calculation_datedif() {
    let schema = json!({
        "type": "object",
        "properties": {
            "form": {
                "type": "object",
                "properties": {
                    "user": {
                        "type": "object",
                        "properties": {
                            "dob": {
                                "type": "string",
                                "dependents": [
                                  {
                                    "$ref": "#/form/properties/user/properties/age",
                                    "value": {
                                      "$evaluation": {
                                        "DATEDIF": [
                                          { "$ref": "$value" },
                                          { "NOW": [] },
                                          "Y"
                                        ]
                                      }
                                    }
                                  }
                                ]
                            },
                            "age": { "type": "number" }
                        }
                    }
                }
            }
        }
    });

    let mut data = json!({
        "form": {
            "user": {
                "dob": "",
                "age": 0
            }
        }
    });

    let mut eval = JSONEval::new(&schema.to_string(), None, Some(&data.to_string()))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data.to_string(), None, None, None)
        .expect("Initial evaluate failed");

    data["form"]["user"]["dob"] = "1986-05-31".into();

    let deps_result = eval
        .evaluate_dependents(
            &["#/properties/form/properties/user/properties/dob".to_string()],
            Some(&data.to_string()),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    let deps_array = deps_result.as_array().expect("deps should be array");
    let age_dep = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("form.user.age"));

    assert!(age_dep.is_some(), "age should be updated");
    let age_val = age_dep.unwrap().get("value").unwrap();
    println!("DATEDIF computed age value: {}", age_val);
    assert_eq!(age_val, &json!(age_on_current_utc_date(1986, 5, 31)));
}

#[test]
fn test_age_calculation_opt_b() {
    let schema = json!({
        "type": "object",
        "properties": {
            "form": {
                "type": "object",
                "properties": {
                    "user": {
                        "type": "object",
                        "properties": {
                            "dob": {
                                "type": "string",
                                "dependents": [
                                  {
                                    "$ref": "#/form/properties/user/properties/age",
                                    "value": {
                                      "$evaluation": {
                                        "-": [
                                          { "YEAR": [{ "NOW": [] }] },
                                          { "YEAR": [{ "$ref": "$value" }] },
                                          {
                                            "if": [
                                              {
                                                "or": [
                                                  {
                                                    "<": [
                                                      { "MONTH": [{ "NOW": [] }] },
                                                      { "MONTH": [{ "$ref": "$value" }] }
                                                    ]
                                                  },
                                                  {
                                                    "and": [
                                                      {
                                                        "==": [
                                                          { "MONTH": [{ "NOW": [] }] },
                                                          { "MONTH": [{ "$ref": "$value" }] }
                                                        ]
                                                      },
                                                      {
                                                        "<": [
                                                          { "DAY": [{ "NOW": [] }] },
                                                          { "DAY": [{ "$ref": "$value" }] }
                                                        ]
                                                      }
                                                    ]
                                                  }
                                                ]
                                              },
                                              1,
                                              0
                                            ]
                                          }
                                        ]
                                      }
                                    }
                                  }
                                ]
                            },
                            "age": { "type": "number" }
                        }
                    }
                }
            }
        }
    });

    let mut data = json!({
        "form": {
            "user": {
                "dob": "",
                "age": 0
            }
        }
    });

    let mut eval = JSONEval::new(&schema.to_string(), None, Some(&data.to_string()))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data.to_string(), None, None, None)
        .expect("Initial evaluate failed");

    data["form"]["user"]["dob"] = "1986-05-31".into();

    let deps_result = eval
        .evaluate_dependents(
            &["#/properties/form/properties/user/properties/dob".to_string()],
            Some(&data.to_string()),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    let deps_array = deps_result.as_array().expect("deps should be array");
    let age_dep = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("form.user.age"));

    assert!(age_dep.is_some(), "age should be updated");
    let age_val = age_dep.unwrap().get("value").unwrap();
    println!("Option B computed age value: {}", age_val);
    assert_eq!(age_val, &json!(age_on_current_utc_date(1986, 5, 31)));
}

#[test]
fn test_age_calculation_timezone_case() {
    let schema = json!({
        "type": "object",
        "properties": {
            "form": {
                "type": "object",
                "properties": {
                    "user": {
                        "type": "object",
                        "properties": {
                            "dob": {
                                "type": "string",
                                "dependents": [
                                  {
                                    "$ref": "#/form/properties/user/properties/age",
                                    "value": {
                                      "$evaluation": {
                                        "-": [
                                          { "YEAR": [{ "$ref": "#/form/properties/user/properties/today_mock" }] },
                                          { "YEAR": [{ "$ref": "$value" }] },
                                          {
                                            "if": [
                                              {
                                                "or": [
                                                  {
                                                    "<": [
                                                      { "MONTH": [{ "$ref": "#/form/properties/user/properties/today_mock" }] },
                                                      { "MONTH": [{ "$ref": "$value" }] }
                                                    ]
                                                  },
                                                  {
                                                    "and": [
                                                      {
                                                        "==": [
                                                          { "MONTH": [{ "$ref": "#/form/properties/user/properties/today_mock" }] },
                                                          { "MONTH": [{ "$ref": "$value" }] }
                                                        ]
                                                      },
                                                      {
                                                        "<": [
                                                          { "DAY": [{ "$ref": "#/form/properties/user/properties/today_mock" }] },
                                                          { "DAY": [{ "$ref": "$value" }] }
                                                        ]
                                                      }
                                                    ]
                                                  }
                                                ]
                                              },
                                              1,
                                              0
                                            ]
                                          }
                                        ]
                                      }
                                    }
                                  }
                                ]
                            },
                            "age": { "type": "number" },
                            "today_mock": { "type": "string" }
                        }
                    }
                }
            }
        }
    });

    let mut data = json!({
        "form": {
            "user": {
                "dob": "",
                "age": 0,
                "today_mock": "2026-05-30"
            }
        }
    });

    let mut eval = JSONEval::new(&schema.to_string(), None, Some(&data.to_string()))
        .expect("Failed to create JSONEval");

    eval.evaluate(&data.to_string(), None, None, None)
        .expect("Initial evaluate failed");

    // Case 1: DOB is "1986-05-30T17:00:00.000Z", today_mock is "2026-05-30" (without offset)
    // The naive parser parses DOB as "1986-05-30". Since today is "2026-05-30", they are exactly 40.
    data["form"]["user"]["dob"] = "1986-05-30T17:00:00.000Z".into();
    data["form"]["user"]["today_mock"] = "2026-05-30".into();

    let deps_result = eval
        .evaluate_dependents(
            &["#/properties/form/properties/user/properties/dob".to_string()],
            Some(&data.to_string()),
            None,
            true,
            None,
            None,
            true,
        )
        .expect("evaluate_dependents failed");

    let deps_array = deps_result.as_array().expect("deps should be array");
    let age_dep = deps_array
        .iter()
        .find(|item| item.get("$ref").and_then(|r| r.as_str()) == Some("form.user.age"));

    assert!(age_dep.is_some(), "age should be updated");
    let age_val = age_dep.unwrap().get("value").unwrap();
    println!(
        "Evaluation with DOB 1986-05-30T17:00:00.000Z on 2026-05-30: {}",
        age_val
    );
    assert_eq!(age_val, &json!(40));
}
