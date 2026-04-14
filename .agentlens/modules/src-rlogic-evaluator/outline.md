# Outline

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

Symbol maps for 2 large files in this module.

## src/rlogic/evaluator/array_lookup.rs (842 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 9 | fn | resolve_table_ref | pub |
| 67 | fn | get_table_array | pub |
| 88 | fn | resolve_column_name | pub |
| 113 | fn | eval_valueat | pub |
| 211 | fn | eval_maxat | pub |
| 240 | fn | eval_indexat | pub |
| 330 | fn | eval_match | pub |
| 442 | fn | eval_matchrange | pub |
| 500 | fn | eval_choose | pub |
| 551 | fn | eval_findindex | pub |
| 722 | fn | eval_condition_with_row | pub |

## src/rlogic/evaluator/mod.rs (891 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 31 | struct | TableScope | pub |
| 48 | struct | TableScopeGuard | pub |
| 53 | fn | drop | (private) |
| 71 | struct | Evaluator | pub |
| 82 | fn | new | pub |
| 98 | fn | enter_table_scope | pub |
| 115 | fn | update_table_scope_rows | pub |
| 125 | fn | set_table_scope_row | pub |
| 134 | fn | with_config | pub |
| 140 | fn | set_static_arrays | pub |
| 148 | fn | index_table | pub |
| 157 | fn | clear_indices | pub |
| 166 | fn | evaluate | pub |
| 230 | fn | evaluate_with_internal_context | pub |
| 246 | fn | evaluate_with_context | (private) |
| 836 | fn | eval_var_or_default | (private) |
| 882 | fn | f64_to_json | (private) |
| 888 | fn | default | (private) |

