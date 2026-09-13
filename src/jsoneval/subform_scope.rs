use crate::jsoneval::path_utils;
use std::borrow::Cow;

/// Maps isolated-subform local data pointers onto canonical parent-form pointers.
///
/// Scope has no data ownership. It is pure path translation; callers keep one
/// canonical parent `EvalData` and choose this scope only while evaluating a subform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SubformScope {
    local_root: String,
    canonical_root: String,
    active_index: Option<usize>,
}

impl SubformScope {
    /// Creates scope from isolated schema root and canonical parent data root.
    /// `schema_root` may include `#/` and `properties` path segments.
    pub(crate) fn new(
        schema_root: &str,
        canonical_root: &str,
        active_index: Option<usize>,
    ) -> Self {
        let local_root = schema_root
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .filter(|segment| !segment.is_empty())
            .map(|segment| format!("/{segment}"))
            .unwrap_or_default();
        let canonical_root = path_utils::normalize_to_json_pointer(canonical_root).into_owned();

        Self {
            local_root,
            canonical_root,
            active_index,
        }
    }

    /// Maps a local data path such as `/riders/code` to its canonical location.
    /// Absolute parent paths and system roots (`/$params`, `/$context`) pass through.
    pub(crate) fn canonical_path<'a>(&self, path: &'a str) -> Cow<'a, str> {
        let path = path_utils::normalize_to_json_pointer(path);
        let local_prefix = format!("{}/", self.local_root);
        let is_local_root = path == self.local_root;
        let is_local_descendant = path.starts_with(&local_prefix);

        if !is_local_root && !is_local_descendant {
            return path;
        }

        let suffix = path.strip_prefix(&self.local_root).unwrap_or_default();
        let mut canonical = self.canonical_root.clone();
        if let Some(index) = self.active_index {
            canonical.push('/');
            canonical.push_str(&index.to_string());
        }
        canonical.push_str(suffix);
        Cow::Owned(canonical)
    }

    /// Builds evaluator input from canonical parent data plus active local alias.
    ///
    /// RLogic still emits isolated schema-local references. Alias makes those reads
    /// resolve to active canonical item while absolute paths continue into same
    /// parent document.
    pub(crate) fn evaluation_view(&self, canonical_data: &serde_json::Value) -> serde_json::Value {
        let Some(local_key) = self.local_root.strip_prefix('/') else {
            return canonical_data.clone();
        };
        let active_path = self.canonical_path(&self.local_root);
        let Some(active_value) = canonical_data.pointer(&active_path) else {
            return canonical_data.clone();
        };
        let Some(parent) = canonical_data.as_object() else {
            return canonical_data.clone();
        };

        let mut view = parent.clone();
        view.insert(local_key.to_string(), active_value.clone());
        serde_json::Value::Object(view)
    }
}

#[cfg(test)]
mod tests {
    use super::SubformScope;

    #[test]
    fn maps_indexed_local_item_to_canonical_parent_array() {
        let scope = SubformScope::new(
            "#/properties/users/properties/items",
            "/users/items",
            Some(1),
        );

        assert_eq!(
            scope.canonical_path("/items/benefit"),
            "/users/items/1/benefit"
        );
    }

    #[test]
    fn evaluation_view_aliases_active_item_without_changing_parent_lookup() {
        let scope = SubformScope::new("#/items", "/users/items", Some(1));
        let data = serde_json::json!({
            "users": { "items": [{ "amount": 11 }, { "amount": 23 }] }
        });
        let view = scope.evaluation_view(&data);

        assert_eq!(view.pointer("/items/amount"), Some(&serde_json::json!(23)));
        assert_eq!(
            view.pointer("/users/items/1/amount"),
            Some(&serde_json::json!(23))
        );
    }

    #[test]
    fn trailing_schema_separator_keeps_parent_paths_unscoped() {
        let scope = SubformScope::new(
            "#/properties/users/properties/items/",
            "/users/items",
            Some(0),
        );
        assert_eq!(
            scope.canonical_path("/users/profile/age"),
            "/users/profile/age"
        );
    }

    #[test]
    fn leaves_parent_and_system_paths_unchanged() {
        let scope = SubformScope::new("#/items", "/users/items", Some(0));

        assert_eq!(
            scope.canonical_path("/users/profile/age"),
            "/users/profile/age"
        );
        assert_eq!(
            scope.canonical_path("/$params/constants/RATE"),
            "/$params/constants/RATE"
        );
    }

    #[test]
    fn maps_unindexed_collection_root() {
        let scope = SubformScope::new("#/items", "/users/items", None);
        assert_eq!(scope.canonical_path("/items"), "/users/items");
        assert_eq!(
            scope.canonical_path("/items/0/code"),
            "/users/items/0/code"
        );
    }
}
