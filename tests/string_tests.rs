use json_eval_rs::*;
use serde_json::json;

/// String operation tests - concatenation, substr, search, etc.
#[cfg(test)]
mod string_tests {
    use super::*;

    #[test]
    fn test_string_concatenation() {
        let mut engine = RLogic::new();
        let data = json!({"first": "Hello", "last": "World"});

        // Basic concatenation
        let logic_id = engine.compile(&json!({"cat": [{"var": "first"}, " ", {"var": "last"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("Hello World"));

        // Concatenate with numbers
        let logic_id = engine.compile(&json!({"cat": ["Value: ", 42]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("Value: 42"));

        // Empty concatenation
        let logic_id = engine.compile(&json!({"cat": []})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(""));
    }

    #[test]
    fn test_string_substr() {
        let mut engine = RLogic::new();
        let data = json!({"text": "Hello World"});

        // Basic substring
        let logic_id = engine.compile(&json!({"substr": [{"var": "text"}, 6, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("World"));

        // Substring from start
        let logic_id = engine.compile(&json!({"substr": [{"var": "text"}, 0, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("Hello"));

        // Substring to end
        let logic_id = engine.compile(&json!({"substr": [{"var": "text"}, 6]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("World"));
    }

    #[test]
    fn test_string_length() {
        let mut engine = RLogic::new();
        let data = json!({
            "array": [1, 2, 3, 4, 5],
            "object": {"a": 1, "b": 2, "c": 3}
        });

        // String length
        let logic_id = engine.compile(&json!({"length": "hello"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(5.0));

        // Len function (same as length)
        let logic_id = engine.compile(&json!({"len": "hello world"})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(11.0));

        // Array length
        let logic_id = engine.compile(&json!({"length": {"var": "array"}})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(5.0));

        // Object length
        let logic_id = engine.compile(&json!({"length": {"var": "object"}})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(3.0));
    }

    #[test]
    fn test_string_search() {
        let mut engine = RLogic::new();
        let data = json!({"text": "Hello World, hello universe"});

        // Basic search (engine returns 1-based index)
        let logic_id = engine.compile(&json!({"search": ["World", {"var": "text"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(7.0));

        // Search with start position
        let logic_id = engine.compile(&json!({"search": ["hello", {"var": "text"}, 8]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(14.0)); // Second "hello" starts at position 14

        // Case insensitive search
        let logic_id = engine.compile(&json!({"search": ["HELLO", {"var": "text"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1.0));

        // Not found
        let logic_id = engine.compile(&json!({"search": ["notfound", {"var": "text"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));
    }

    #[test]
    fn test_string_extraction() {
        let mut engine = RLogic::new();
        let data = json!({"text": "Hello World"});

        // Left extraction
        let logic_id = engine.compile(&json!({"left": [{"var": "text"}, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("Hello"));

        // Left with default length
        let logic_id = engine.compile(&json!({"left": [{"var": "text"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("H"));

        // Right extraction
        let logic_id = engine.compile(&json!({"right": [{"var": "text"}, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("World"));

        // Right with default length
        let logic_id = engine.compile(&json!({"right": [{"var": "text"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("d"));
    }

    #[test]
    fn test_string_mid() {
        let mut engine = RLogic::new();
        let data = json!({"text": "Hello World"});

        // Mid extraction
        let logic_id = engine.compile(&json!({"mid": [{"var": "text"}, 6, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(" Worl"));

        // Mid from position 1 (0-indexed)
        let logic_id = engine.compile(&json!({"mid": [{"var": "text"}, 1, 4]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("Hell"));
    }

    #[test]
    fn test_string_split() {
        let mut engine = RLogic::new();
        let data = json!({"csv": "a,b,c,d,e"});

        // Split text
        let logic_id = engine.compile(&json!({"splitvalue": [{"var": "csv"}, ","]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(["a", "b", "c", "d", "e"]));

        // Split text and get specific index
        let logic_id = engine.compile(&json!({"splittext": [{"var": "csv"}, ",", 2]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("c"));
    }

    #[test]
    fn test_string_operations_edge_cases() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Empty string operations
        let logic_id = engine.compile(&json!({"length": ""})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0));

        let logic_id = engine.compile(&json!({"cat": ["", ""]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(""));

        // Substr edge cases
        let logic_id = engine.compile(&json!({"substr": ["hello", 10, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(""));

        let logic_id = engine.compile(&json!({"substr": ["hello", -1, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("o"));

        // Search edge cases
        let logic_id = engine.compile(&json!({"search": ["", "hello"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(1.0));

        let logic_id = engine.compile(&json!({"search": ["x", ""]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null)); // Can't find "x" in empty string

        // Left/Right with out of bounds
        let logic_id = engine.compile(&json!({"left": ["hi", 10]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("hi"));

        let logic_id = engine.compile(&json!({"right": ["hi", 10]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("hi"));
    }

    #[test]
    fn test_string_operations_with_non_strings() {
        let mut engine = RLogic::new();
        let data = json!({"number": 42, "bool": true, "null": null, "array": [1,2,3]});

        // Concat with mixed types
        let logic_id = engine.compile(&json!({"cat": [{"var": "number"}, "-", {"var": "bool"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("42-true"));

        // Length of non-string
        let logic_id = engine.compile(&json!({"length": {"var": "array"}})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(3.0));

        // Search in non-string (should work for arrays via string conversion)
        let logic_id = engine.compile(&json!({"search": ["2", {"var": "array"}]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null)); // Array stringifies to "[1,2,3]"
    }

    #[test]
    fn test_string_operations_errors() {
        let mut engine = RLogic::new();
        let data = json!({});

        // Substr with non-string
        let logic_id = engine.compile(&json!({"substr": [null, 0, 5]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("")); // null becomes empty string

        // Search with null
        let logic_id = engine.compile(&json!({"search": ["test", null]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(null));

        // Length of null
        let logic_id = engine.compile(&json!({"length": null})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(0.0));
    }

    #[test]
    fn test_string_unicode() {
        let mut engine = RLogic::new();
        let data = json!({"text": "Hello 世界 🌍"});

        // Unicode length
        let logic_id = engine.compile(&json!({"length": {"var": "text"}})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!(17.0));

        // Unicode concatenation
        let logic_id = engine.compile(&json!({"cat": ["Hello ", "世界 🌍"]})).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("Hello 世界 🌍"));
    }

    #[test]
    fn test_string_complex_operations() {
        let mut engine = RLogic::new();
        let data = json!({
            "sentence": "The quick brown fox jumps over the lazy dog",
            "words": ["The", "quick", "brown", "fox"]
        });

        // Extract word from sentence
        let logic_id = engine.compile(&json!({
            "mid": [
                {"var": "sentence"},
                {"search": ["fox", {"var": "sentence"}]},
                3
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("fox"));

        // Build sentence from words
        let logic_id = engine.compile(&json!({
            "cat": [
                {"var": "words.0"}, " ",
                {"var": "words.1"}, " ",
                {"var": "words.2"}, " ",
                {"var": "words.3"}
            ]
        })).unwrap();
        let result = engine.run(&logic_id, &data).unwrap();
        assert_eq!(result, json!("The quick brown fox"));
    }
}
