# Outline

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

Symbol maps for 8 large files in this module.

## src/jsoneval/core.rs (646 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 18 | fn | clone | (private) |
| 55 | fn | new | pub |
| 137 | fn | new_subform | pub |
| 209 | fn | new_from_msgpack | pub |
| 305 | fn | with_parsed_schema | pub |
| 366 | fn | reload_schema | pub |
| 453 | fn | set_timezone_offset | pub |
| 480 | fn | reload_schema_msgpack | pub |
| 560 | fn | reload_schema_parsed | pub |
| 632 | fn | reload_schema_from_cache | pub |

## src/jsoneval/dependents.rs (1596 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 20 | fn | evaluate_dependents | pub |
| 226 | fn | run_re_evaluate_pass | (private) |
| 494 | fn | run_schema_default_value_pass | (private) |
| 611 | fn | run_subform_pass | (private) |
| 981 | fn | evaluate_dependent_value_static | pub |
| 1019 | fn | check_readonly_for_dependents | pub |
| 1074 | fn | collect_readonly_fixes | pub |
| 1136 | fn | check_hidden_field | pub |
| 1183 | fn | collect_hidden_fields | pub |
| 1268 | fn | recursive_hide_effect | pub |
| 1341 | fn | process_dependents_queue | pub |
| 1585 | fn | subform_field_key | (private) |

## src/jsoneval/eval_cache.rs (682 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 7 | struct | VersionTracker | pub |
| 12 | fn | new | pub |
| 19 | fn | get | pub |
| 24 | fn | bump | pub |
| 36 | fn | merge_from | pub |
| 46 | fn | merge_from_params | pub |
| 57 | fn | any_bumped_with_prefix | pub |
| 66 | fn | any_newly_bumped_with_prefix | pub |
| 73 | fn | versions | pub |
| 80 | struct | CacheEntry | pub |
| 92 | struct | SubformItemCache | pub |
| 103 | fn | new | pub |
| 115 | struct | EvalCache | pub |
| 136 | fn | default | (private) |
| 142 | fn | new | pub |
| 155 | fn | clear | pub |
| 169 | fn | prune_subform_caches | pub |
| 179 | fn | invalidate_params_tables_for_item | pub |
| 196 | fn | needs_full_evaluation | pub |
| 201 | fn | mark_evaluated | pub |
| 205 | fn | ensure_active_item_cache | pub |
| 211 | fn | set_active_item | pub |
| 216 | fn | clear_active_item | pub |
| 221 | fn | store_snapshot_and_diff_versions | pub |
| 232 | fn | get_active_snapshot | pub |
| 243 | fn | diff_active_item | pub |
| 269 | fn | bump_data_version | pub |
| 282 | fn | bump_params_version | pub |
| 292 | fn | check_cache | pub |
| 359 | fn | check_table_cache | pub |
| 393 | fn | validate_entry | (private) |
| 443 | fn | store_cache | pub |
| 566 | fn | diff_and_update_versions | pub |
| 578 | fn | diff_and_update_versions_internal | (private) |
| 654 | fn | traverse_and_bump | (private) |

## src/jsoneval/evaluate.rs (865 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 26 | fn | items_same_input_identity | (private) |
| 46 | fn | evaluate | pub |
| 76 | fn | evaluate_internal_with_new_data | pub |
| 179 | fn | invalidate_subform_caches_on_structural_change | pub |
| 277 | fn | evaluate_internal_pre_diffed | pub |
| 301 | fn | evaluate_internal | pub |
| 666 | fn | evaluate_others | pub |
| 817 | fn | evaluate_options_templates | (private) |
| 847 | fn | evaluate_template | (private) |

## src/jsoneval/getters.rs (701 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 12 | fn | is_effective_hidden | pub |
| 61 | fn | prune_hidden_values | (private) |
| 104 | fn | resolve_static_markers_in_value | (private) |
| 128 | fn | get_evaluated_schema | pub |
| 138 | fn | get_resolved_layout | pub |
| 165 | fn | get_evaluated_schema_resolved | pub |
| 170 | struct | ResolveEntry | (private) |
| 280 | fn | resolve_static_markers_at_path | (private) |
| 317 | fn | get_schema_value_by_path | pub |
| 325 | fn | get_schema_value | pub |
| 426 | fn | get_schema_value_array | pub |
| 479 | fn | get_schema_value_object | pub |
| 523 | fn | get_evaluated_schema_without_params | pub |
| 532 | fn | get_evaluated_schema_msgpack | pub |
| 538 | fn | get_evaluated_schema_by_path | pub |
| 543 | fn | get_evaluated_schema_by_paths | pub |
| 583 | fn | get_schema_by_path | pub |
| 591 | fn | get_schema_by_paths | pub |
| 626 | fn | insert_at_path | pub |
| 657 | fn | flatten_object | pub |
| 679 | fn | convert_to_format | pub |

## src/jsoneval/subform_methods.rs (904 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 24 | fn | resolve_subform_path | (private) |
| 84 | fn | normalize_to_subform_key | (private) |
| 104 | fn | resolve_subform_path_alias | pub |
| 136 | fn | with_item_cache_swap | (private) |
| 562 | fn | evaluate_subform | pub |
| 583 | fn | evaluate_subform_item | (private) |
| 611 | fn | validate_subform | pub |
| 655 | fn | evaluate_dependents_subform | pub |
| 718 | fn | resolve_layout_subform | pub |
| 732 | fn | get_evaluated_schema_subform | pub |
| 760 | fn | get_schema_value_subform | pub |
| 770 | fn | get_schema_value_array_subform | pub |
| 780 | fn | get_schema_value_object_subform | pub |
| 790 | fn | get_evaluated_schema_without_params_subform | pub |
| 803 | fn | get_evaluated_schema_by_path_subform | pub |
| 818 | fn | get_evaluated_schema_by_paths_subform | pub |
| 839 | fn | get_schema_by_path_subform | pub |
| 851 | fn | get_schema_by_paths_subform | pub |
| 869 | fn | get_resolved_layout_subform | pub |
| 882 | fn | get_evaluated_schema_resolved_subform | pub |
| 895 | fn | get_subform_paths | pub |
| 900 | fn | has_subform | pub |

## src/jsoneval/table_evaluate.rs (568 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 24 | fn | evaluate_table | pub |
| 42 | fn | evaluate_table_inner | (private) |

## src/jsoneval/validation.rs (601 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 14 | fn | validate | pub |
| 91 | fn | validate_pre_set | pub |
| 127 | fn | validate_field | pub |
| 185 | fn | get_field_data | pub |
| 203 | fn | validate_rule | pub |
| 447 | fn | dep_fails_schema_rules | pub |
| 520 | fn | rule_value_fails | (private) |

