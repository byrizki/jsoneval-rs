# Outline

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

Symbol maps for 1 large files in this module.

## src/rlogic/compiled.rs (1868 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 8 | struct | LogicId | pub |
| 12 | enum | CompiledLogic | pub |
| 177 | fn | compile | pub |
| 200 | fn | compile_operator | (private) |
| 1028 | fn | compile_binary | (private) |
| 1049 | fn | preprocess_table_condition | (private) |
| 1127 | fn | is_simple_ref | pub |
| 1135 | fn | referenced_vars | pub |
| 1144 | fn | flatten_and | (private) |
| 1159 | fn | flatten_or | (private) |
| 1174 | fn | flatten_add | (private) |
| 1189 | fn | flatten_multiply | (private) |
| 1205 | fn | flatten_cat | (private) |
| 1221 | fn | has_forward_reference | pub |
| 1226 | fn | check_forward_reference | (private) |
| 1394 | fn | contains_iteration_plus_positive | (private) |
| 1417 | fn | normalize_ref_path | (private) |
| 1445 | fn | collect_vars | pub |
| 1633 | struct | CompiledLogicStore | pub |
| 1640 | fn | new | pub |
| 1653 | fn | compile | pub |
| 1676 | fn | get | pub |
| 1681 | fn | remove | pub |
| 1687 | fn | get_dependencies | pub |
| 1693 | fn | default | (private) |
| 1703 | fn | is_ok | (private) |
| 1708 | fn | test_compile_literals | (private) |
| 1734 | fn | test_compile_variables | (private) |
| 1742 | fn | test_compile_logical | (private) |
| 1757 | fn | test_compile_comparison | (private) |
| 1772 | fn | test_compile_arithmetic | (private) |
| 1785 | fn | test_compile_array_ops | (private) |
| 1800 | fn | test_compile_string_ops | (private) |
| 1813 | fn | test_compile_math_ops | (private) |
| 1826 | fn | test_compile_date_ops | (private) |
| 1836 | fn | test_compile_table_ops | (private) |
| 1855 | fn | test_compile_util_ops | (private) |
| 1864 | fn | test_compile_unknown | (private) |

