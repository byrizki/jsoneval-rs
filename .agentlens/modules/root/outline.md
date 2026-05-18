# Outline

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

Symbol maps for 9 large files in this module.

## bindings/csharp/examples/benchmark/Program.cs (618 lines)

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

## bindings/npm/packages/react-native/android/src/main/cpp/json-eval-rn.cpp (938 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 31 | fn | stringToJstring | (private) |
| 57 | fn | resolvePromise | pub |
| 63 | fn | rejectPromise | pub |
| 77 | fn | runAsyncWithPromise | pub |
| 123 | method | stringToJstring | (internal) |
| 171 | method | stringToJstring | (internal) |
| 193 | method | stringToJstring | (internal) |
| 626 | method | stringToJstring | (internal) |

## bindings/npm/examples/nextjs/components/InsuranceForm.tsx (565 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 214 | fn | handleDateChange | (private) |
| 253 | fn | handleSmokerChange | (private) |
| 296 | fn | handleOccupationChange | (private) |

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

## tests/test_evaluate_others.rs (880 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 6 | fn | get_by_pointer | (private) |
| 17 | fn | set_by_pointer | (private) |
| 27 | fn | merge_layout_overlay | (private) |
| 82 | fn | test_options_url_dynamic_template_evaluation | (private) |
| 147 | fn | test_options_url_template_evaluation | (private) |
| 203 | fn | test_options_url_template_with_number_params | (private) |
| 252 | fn | test_options_url_without_template_unchanged | (private) |
| 301 | fn | test_layout_metadata_injection | (private) |
| 389 | fn | test_layout_metadata_parent_hidden | (private) |
| 485 | fn | test_hide_layout_propagation | (private) |
| 616 | fn | test_direct_layout_elements_have_metadata | (private) |
| 712 | fn | test_json_pointer_ref_conversion | (private) |
| 783 | fn | test_multiple_options_templates_in_schema | (private) |
| 852 | fn | test_options_template_collected_at_parse_time | (private) |

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

