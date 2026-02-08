//! GraphQL endpoint resolution.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

#[cfg(target_arch = "wasm32")]
fn window_value(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let value = js_sys::Reflect::get(&window, &JsValue::from_str(key)).ok()?;
    value.as_string()
}

#[cfg(target_arch = "wasm32")]
fn window_config_value(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let config = js_sys::Reflect::get(&window, &JsValue::from_str("__APP_CONFIG__")).ok()?;
    let value = js_sys::Reflect::get(&config, &JsValue::from_str(key)).ok()?;
    value.as_string()
}

/// Resolve the GraphQL endpoint from environment overrides or runtime config.
pub fn graphql_endpoint() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(value) = window_value("__APP_GRAPHQL_ENDPOINT") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }

        if let Some(value) = window_config_value("graphqlEndpoint") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    if let Some(value) = option_env!("APP__GRAPHQL_ENDPOINT") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(value) = std::env::var("APP__GRAPHQL_ENDPOINT") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    "http://localhost:3000/graphql".to_string()
}
