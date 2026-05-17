use wasm_bindgen::prelude::*;

/// Check Favnir source code and return diagnostics as a JS value.
/// Returns an array of { code, message, line, col } objects.
#[wasm_bindgen]
pub fn fav_check(source: &str) -> JsValue {
    let diagnostics = fav_core::check_source(source);
    serde_wasm_bindgen::to_value(&diagnostics).unwrap_or(JsValue::NULL)
}
