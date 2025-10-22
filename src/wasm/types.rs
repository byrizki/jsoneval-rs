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
        serde_wasm_bindgen::to_value(&self).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// WebAssembly wrapper for JSONEval
#[wasm_bindgen]
pub struct JSONEvalWasm {
    pub(super) inner: JSONEval,
}

// Helper function to create ValidationError from internal type
pub(super) fn create_validation_error(path: String, rule_type: String, message: String) -> ValidationError {
    ValidationError {
        path,
        rule_type,
        message,
    }
}

// Helper function to create ValidationResult from internal type
pub(super) fn create_validation_result(has_error: bool, errors: Vec<ValidationError>) -> ValidationResult {
    ValidationResult {
        has_error,
        errors,
    }
}
