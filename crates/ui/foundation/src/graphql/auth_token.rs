//! Auth token storage helpers.

#[cfg(target_arch = "wasm32")]
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

#[cfg(target_arch = "wasm32")]
pub fn auth_token() -> Option<String> {
    local_storage()
        .and_then(|storage| storage.get_item("app_auth_token").ok().flatten())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn auth_token() -> Option<String> {
    std::env::var("APP__AUTH_TOKEN").ok()
}

#[cfg(target_arch = "wasm32")]
pub fn set_auth_token(token: Option<&str>) {
    if let Some(storage) = local_storage() {
        if let Some(value) = token {
            let _ = storage.set_item("app_auth_token", value);
        } else {
            let _ = storage.remove_item("app_auth_token");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_auth_token(token: Option<&str>) {
    if let Some(value) = token {
        unsafe {
            std::env::set_var("APP__AUTH_TOKEN", value);
        }
    } else {
        unsafe {
            std::env::remove_var("APP__AUTH_TOKEN");
        }
    }
}
