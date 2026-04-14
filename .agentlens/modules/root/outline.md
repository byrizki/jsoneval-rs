# Outline

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

Symbol maps for 12 large files in this module.

## bindings/csharp-example/Program.cs (618 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 10 | mod | JsonEvalBenchmark | pub |
| 13 | class | Program | (internal) |
| 16 | method | Main | (private) |
| 58 | method | PrintComparisonResults | (private) |
| 92 | method | BuildRelease | (private) |
| 131 | method | GetLibraryFileName | (private) |
| 141 | method | RunCommand | (private) |
| 194 | class | BenchmarkResult | (private) |
| 197 | const | Success | pub |
| 198 | const | TotalMs | pub |
| 199 | const | ParsingMs | pub |
| 200 | const | EvaluationMs | pub |
| 201 | const | Scenario | pub |
| 202 | const | EvaluatedSchema | pub |
| 203 | const | DifferenceCount | pub |
| 205 | method | RunCSharpBenchmark | (private) |
| 226 | method | FileNotFoundException | (private) |
| 230 | method | FileNotFoundException | (private) |
| 256 | method | JsonEvalException | (private) |
| 269 | method | JsonEvalException | (private) |
| 349 | method | RunRustBenchmark | (private) |
| 459 | method | PrintComparisonResults | (private) |
| 538 | method | ParseDuration | (private) |
| 557 | method | FindDifferences | (private) |

## bindings/react-native/packages/react-native/android/src/main/cpp/json-eval-rn.cpp (909 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 30 | fn | stringToJstring | (private) |
| 56 | fn | resolvePromise | pub |
| 62 | fn | rejectPromise | pub |
| 76 | fn | runAsyncWithPromise | pub |
| 117 | method | stringToJstring | (internal) |
| 165 | method | stringToJstring | (internal) |
| 187 | method | stringToJstring | (internal) |
| 620 | method | stringToJstring | (internal) |

## bindings/react-native/packages/react-native/cpp/json-eval-bridge.cpp (1466 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 19 | method | json_eval_new | (internal) |
| 21 | method | json_eval_new_from_msgpack | (internal) |
| 22 | method | json_eval_evaluate | (internal) |
| 23 | method | json_eval_get_evaluated_schema_msgpack | (internal) |
| 24 | method | json_eval_validate | (internal) |
| 25 | method | json_eval_evaluate_dependents | (internal) |
| 26 | method | json_eval_get_evaluated_schema | (internal) |
| 27 | method | json_eval_get_schema_value | (internal) |
| 28 | method | json_eval_get_schema_value_array | (internal) |
| 29 | method | json_eval_get_schema_value_object | (internal) |
| 30 | method | json_eval_get_evaluated_schema_without_params | (internal) |
| 31 | method | json_eval_get_evaluated_schema_by_path | (internal) |
| 32 | method | json_eval_get_evaluated_schema_by_paths | (internal) |
| 33 | method | json_eval_get_schema_by_path | (internal) |
| 34 | method | json_eval_get_schema_by_paths | (internal) |
| 35 | method | json_eval_resolve_layout | (internal) |
| 36 | method | json_eval_compile_and_run_logic | (internal) |
| 37 | method | json_eval_compile_logic | (internal) |
| 38 | method | json_eval_run_logic | (internal) |
| 39 | method | json_eval_reload_schema | (internal) |
| 40 | method | json_eval_reload_schema_msgpack | (internal) |
| 41 | method | json_eval_reload_schema_from_cache | (internal) |
| 42 | method | json_eval_new_from_cache | (internal) |
| 43 | method | json_eval_validate_paths | (internal) |
| 44 | method | json_eval_evaluate_logic_pure | (internal) |
| 47 | method | json_eval_evaluate_subform | (internal) |
| 48 | method | json_eval_validate_subform | (internal) |
| 49 | method | json_eval_evaluate_dependents_subform | (internal) |
| 50 | method | json_eval_resolve_layout_subform | (internal) |
| 51 | method | json_eval_get_evaluated_schema_subform | (internal) |
| 52 | method | json_eval_get_schema_value_subform | (internal) |
| 53 | method | json_eval_get_schema_value_array_subform | (internal) |
| 54 | method | json_eval_get_schema_value_object_subform | (internal) |
| 55 | method | json_eval_get_evaluated_schema_without_params_subform | (internal) |
| 56 | method | json_eval_get_evaluated_schema_by_path_subform | (internal) |
| 57 | method | json_eval_get_evaluated_schema_by_paths_subform | (internal) |
| 58 | method | json_eval_get_schema_by_path_subform | (internal) |
| 59 | method | json_eval_get_schema_by_paths_subform | (internal) |
| 60 | method | json_eval_get_subform_paths | (internal) |
| 61 | method | json_eval_has_subform | (internal) |
| 62 | method | json_eval_set_timezone_offset | (internal) |
| 64 | method | json_eval_free | (internal) |
| 66 | method | json_eval_cancel | (internal) |
| 67 | method | json_eval_free_result | (internal) |
| 69 | method | json_eval_free_string | (internal) |
| 72 | mod | jsoneval | pub |
| 242 | method | JsonEvalBridge::compileLogic | pub |
| 1411 | method | JsonEvalBridge::dispose | pub |
| 1437 | method | JsonEvalBridge::setTimezoneOffset | pub |
| 1452 | method | JsonEvalBridge::cancel | pub |

## bindings/react-native/packages/react-native/cpp/json-eval-bridge.h (648 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 8 | mod | jsoneval | pub |
| 14 | class | JsonEvalBridge | pub |
| 327 | method | compileLogic | (internal) |
| 619 | method | setTimezoneOffset | (internal) |
| 628 | method | dispose | (internal) |
| 634 | method | cancel | (internal) |

## bindings/react-native/packages/react-native/lib/typescript/index.d.ts (666 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 4 | interface | SchemaValueItem | pub |
| 24 | interface | ValidationError | pub |
| 43 | interface | ValidationResult | pub |
| 52 | interface | DependentChange | pub |
| 69 | interface | JSONEvalOptions | pub |
| 80 | interface | EvaluateOptions | pub |
| 91 | interface | ValidatePathsOptions | pub |
| 102 | interface | EvaluateDependentsOptions | pub |
| 117 | interface | EvaluateSubformOptions | pub |
| 130 | interface | ValidateSubformOptions | pub |
| 141 | interface | EvaluateDependentsSubformOptions | pub |
| 158 | interface | ResolveLayoutSubformOptions | pub |
| 167 | interface | GetEvaluatedSchemaSubformOptions | pub |
| 176 | interface | GetSchemaValueSubformOptions | pub |
| 183 | interface | GetEvaluatedSchemaByPathSubformOptions | pub |
| 194 | interface | GetEvaluatedSchemaByPathsSubformOptions | pub |
| 207 | interface | GetSchemaByPathSubformOptions | pub |
| 216 | interface | GetSchemaByPathsSubformOptions | pub |

## bindings/web/examples/nextjs/components/InsuranceForm.tsx (577 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 226 | fn | handleDateChange | (private) |
| 265 | fn | handleSmokerChange | (private) |
| 308 | fn | handleOccupationChange | (private) |

## examples/benchmark.rs (544 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 10 | fn | print_help | (private) |
| 44 | fn | main | (private) |

## tests/array_tests.rs (560 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 10 | fn | test_array_map | (private) |
| 30 | fn | test_array_filter | (private) |
| 50 | fn | test_array_reduce | (private) |
| 66 | fn | test_array_quantifiers | (private) |
| 93 | fn | test_array_merge | (private) |
| 113 | fn | test_array_in | (private) |
| 145 | fn | test_array_operations_edge_cases | (private) |
| 193 | fn | test_array_operations_with_objects | (private) |
| 234 | fn | test_array_operations_errors | (private) |
| 261 | fn | test_array_sum | (private) |
| 286 | fn | test_array_for_loop | (private) |
| 306 | fn | test_real_world_table_processing | (private) |
| 343 | fn | test_nested_array_operations | (private) |
| 378 | fn | test_array_operations_with_nulls | (private) |
| 417 | fn | test_array_conditional_operations | (private) |
| 450 | fn | test_array_with_large_dataset | (private) |
| 487 | fn | test_array_grouping_simulation | (private) |
| 529 | fn | test_array_edge_case_empty_operations | (private) |

## tests/json_eval_tests.rs (920 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 8 | fn | create_test_schema | (private) |
| 13 | fn | test_evaluate_basic | (private) |
| 33 | fn | test_evaluate_with_context | (private) |
| 59 | fn | test_validate_all_rules_pass | (private) |
| 74 | fn | test_validate_required_field_missing | (private) |
| 113 | fn | test_validate_min_max_value | (private) |
| 201 | fn | test_validate_skip_hidden_fields | (private) |
| 222 | fn | test_validate_with_path_filter | (private) |
| 290 | fn | test_evaluate_dependents_basic | (private) |
| 350 | fn | test_evaluate_dependents_with_clear | (private) |
| 420 | fn | test_evaluate_dependents_transitive | (private) |
| 499 | fn | test_evaluate_dependents_no_data_update | (private) |
| 560 | fn | test_evaluate_dependents_output_structure | (private) |
| 741 | fn | test_evaluate_dependents_dot_notation | (private) |
| 820 | fn | test_evaluate_dependents_with_dot_notation_input | (private) |
| 863 | fn | test_evaluate_dependents_dot_vs_schema_path | (private) |

## tests/table_tests.rs (681 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 10 | fn | test_valueat_basic | (private) |
| 43 | fn | test_valueat_edge_cases | (private) |
| 84 | fn | test_maxat | (private) |
| 104 | fn | test_indexat | (private) |
| 145 | fn | test_match | (private) |
| 178 | fn | test_match_range | (private) |
| 204 | fn | test_choose | (private) |
| 230 | fn | test_find_index | (private) |
| 276 | fn | test_table_operations_with_references | (private) |
| 302 | fn | test_table_operations_performance | (private) |
| 335 | fn | test_table_operations_errors | (private) |
| 365 | fn | test_findindex_preprocessing | (private) |
| 393 | fn | test_findindex_with_and | (private) |
| 434 | fn | test_combined_table_operations | (private) |
| 481 | fn | test_mapoptions | (private) |
| 509 | fn | test_mapoptionsif | (private) |
| 584 | fn | test_mapoptions_with_ref | (private) |
| 614 | fn | test_mapoptionsif_with_ref | (private) |
| 649 | fn | test_mapoptionsif_with_evaluation_blocks | (private) |

## tests/test_evaluate_others.rs (704 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 5 | fn | test_options_url_dynamic_template_evaluation | (private) |
| 60 | fn | test_options_url_template_evaluation | (private) |
| 107 | fn | test_options_url_template_with_number_params | (private) |
| 146 | fn | test_options_url_without_template_unchanged | (private) |
| 185 | fn | test_layout_metadata_injection | (private) |
| 263 | fn | test_layout_metadata_parent_hidden | (private) |
| 349 | fn | test_hide_layout_propagation | (private) |
| 470 | fn | test_direct_layout_elements_have_metadata | (private) |
| 556 | fn | test_json_pointer_ref_conversion | (private) |
| 617 | fn | test_multiple_options_templates_in_schema | (private) |
| 676 | fn | test_options_template_collected_at_parse_time | (private) |

## tests/test_subforms.rs (659 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 5 | fn | test_subform_detection_and_creation | (private) |
| 54 | fn | test_subform_schema_structure | (private) |
| 120 | fn | test_evaluate_subform | (private) |
| 176 | fn | test_validate_subform | (private) |
| 248 | fn | test_evaluate_dependents_subform | (private) |
| 304 | fn | test_resolve_layout_subform | (private) |
| 346 | fn | test_multiple_subforms | (private) |
| 393 | fn | test_subform_isolation | (private) |
| 433 | fn | test_get_schema_value_subform | (private) |
| 464 | fn | test_get_evaluated_schema_without_params_subform | (private) |
| 501 | fn | test_nonexistent_subform_error | (private) |
| 521 | fn | test_nested_subform_key | (private) |
| 562 | fn | test_evaluate_dependents_subform_array_iteration | (private) |

