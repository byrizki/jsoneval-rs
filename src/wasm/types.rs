//! WASM type definitions

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use crate::JSONEval;

/// Validation error for JavaScript
#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
pub struct ValidationError {
    path: String,
    rule_type: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

#[wasm_bindgen]
impl ValidationError {
    #[wasm_bindgen(getter)]
    pub fn path(&self) -> String {
        self.path.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn rule_type(&self) -> String {
        self.rule_type.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn code(&self) -> Option<String> {
        self.code.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn pattern(&self) -> Option<String> {
        self.pattern.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn field_value(&self) -> Option<String> {
        self.field_value.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> JsValue {
        self.data.as_ref()
            .and_then(|d| super::to_value(d).ok())
            .unwrap_or(JsValue::NULL)
    }
}

/// Validation result for JavaScript
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct ValidationResult {
    has_error: bool,
    errors: Vec<ValidationError>,
}

#[wasm_bindgen]
impl ValidationResult {
    #[wasm_bindgen(getter)]
    pub fn has_error(&self) -> bool {
        self.has_error
    }

    #[wasm_bindgen(getter)]
    pub fn errors(&self) -> Vec<ValidationError> {
        self.errors.clone()
    }

    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        super::to_value(&self)
    }
}

/// WebAssembly wrapper for JSONEval
#[wasm_bindgen]
pub struct JSONEvalWasm {
    pub(super) inner: JSONEval,
}

// Helper function to create ValidationError from internal type
pub(super) fn create_validation_error(
    path: String,
    error: &crate::ValidationError,
) -> ValidationError {
    ValidationError {
        path,
        rule_type: error.rule_type.clone(),
        message: error.message.clone(),
        code: error.code.clone(),
        pattern: error.pattern.clone(),
        field_value: error.field_value.clone(),
        data: error.data.clone(),
    }
}

// Helper function to create ValidationResult from internal type
pub(super) fn create_validation_result(has_error: bool, errors: Vec<ValidationError>) -> ValidationResult {
    ValidationResult {
        has_error,
        errors,
    }
}
