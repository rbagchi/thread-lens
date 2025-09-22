use wasm_bindgen::prelude::*;
use console_error_panic_hook;

pub mod analyzer;
pub mod models;
pub mod parser;

#[wasm_bindgen(start)]
pub fn main_wasm() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    // You can add more Wasm-specific initialization here
    Ok(())
}

#[wasm_bindgen]
pub fn parse_jstack_output_wasm(input: &str) -> Result<JsValue, JsValue> {
    let dump = parser::parse_jstack_output(input).map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string(&dump)
        .map_err(|e| JsValue::from_str(&e.to_string()))
        .map(|s| JsValue::from_str(&s))
}

#[wasm_bindgen]
pub fn find_chronically_blocked_threads_wasm(dumps_json: &str) -> Result<JsValue, JsValue> {
    let dumps: Vec<models::ThreadDump> = serde_json::from_str(dumps_json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let result = analyzer::find_chronically_blocked_threads(&dumps);

    // Convert HashMap<String, (NormalizedThread, usize)> to a serializable format
    let serializable_result: Vec<_> = result.into_iter().map(|(name, (thread, count))| {
        serde_json::json!({
            "name": name,
            "thread": thread,
            "count": count
        })
    }).collect();

    serde_json::to_string(&serializable_result)
        .map_err(|e| JsValue::from_str(&e.to_string()))
        .map(|s| JsValue::from_str(&s))
}
