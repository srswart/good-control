//! WebAuthn helpers for web builds.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use js_sys::Promise;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
function base64urlToBuffer(value) {
  const padding = '='.repeat((4 - (value.length % 4)) % 4);
  const base64 = (value + padding).replace(/-/g, '+').replace(/_/g, '/');
  const raw = atob(base64);
  const buffer = new ArrayBuffer(raw.length);
  const bytes = new Uint8Array(buffer);
  for (let i = 0; i < raw.length; i++) {
    bytes[i] = raw.charCodeAt(i);
  }
  return buffer;
}

function bufferToBase64url(buffer) {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  const base64 = btoa(binary).replace(/\x2b/g, '-').replace(/\x2f/g, '_').replace(/=+$/, '');
  return base64;
}

function normalizePublicKeyOptions(options) {
  const publicKey = options.publicKey || options.public_key;
  if (!publicKey) {
    throw new Error('Missing publicKey options');
  }
  publicKey.challenge = base64urlToBuffer(publicKey.challenge);
  if (publicKey.user && publicKey.user.id) {
    publicKey.user.id = base64urlToBuffer(publicKey.user.id);
  }
  if (publicKey.excludeCredentials) {
    publicKey.excludeCredentials = publicKey.excludeCredentials.map((cred) => ({
      ...cred,
      id: base64urlToBuffer(cred.id),
    }));
  }
  if (publicKey.allowCredentials) {
    publicKey.allowCredentials = publicKey.allowCredentials.map((cred) => ({
      ...cred,
      id: base64urlToBuffer(cred.id),
    }));
  }
  return publicKey;
}

export async function registerPasskey(optionsJson) {
  const options = JSON.parse(optionsJson);
  const publicKey = normalizePublicKeyOptions(options);
  const credential = await navigator.credentials.create({ publicKey });
  const response = credential.response;
  const result = {
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    response: {
      attestationObject: bufferToBase64url(response.attestationObject),
      clientDataJSON: bufferToBase64url(response.clientDataJSON),
    },
    clientExtensionResults: credential.getClientExtensionResults(),
  };
  return JSON.stringify(result);
}

export async function authenticatePasskey(optionsJson) {
  const options = JSON.parse(optionsJson);
  const publicKey = normalizePublicKeyOptions(options);
  const credential = await navigator.credentials.get({ publicKey });
  const response = credential.response;
  const result = {
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    response: {
      authenticatorData: bufferToBase64url(response.authenticatorData),
      clientDataJSON: bufferToBase64url(response.clientDataJSON),
      signature: bufferToBase64url(response.signature),
      userHandle: response.userHandle ? bufferToBase64url(response.userHandle) : null,
    },
    clientExtensionResults: credential.getClientExtensionResults(),
  };
  return JSON.stringify(result);
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = registerPasskey)]
    fn register_passkey(options_json: String) -> Promise;

    #[wasm_bindgen(js_name = authenticatePasskey)]
    fn authenticate_passkey(options_json: String) -> Promise;
}

#[cfg(target_arch = "wasm32")]
pub async fn webauthn_register(options_json: &str) -> Result<String, String> {
    let promise = register_passkey(options_json.to_string());
    let value = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Passkey registration failed: {:?}", e))?;
    Ok(value.as_string().unwrap_or_default())
}

#[cfg(target_arch = "wasm32")]
pub async fn webauthn_authenticate(options_json: &str) -> Result<String, String> {
    let promise = authenticate_passkey(options_json.to_string());
    let value = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Passkey authentication failed: {:?}", e))?;
    Ok(value.as_string().unwrap_or_default())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn webauthn_register(_options_json: &str) -> Result<String, String> {
    Err("Passkeys are only supported on the web runtime.".to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn webauthn_authenticate(_options_json: &str) -> Result<String, String> {
    Err("Passkeys are only supported on the web runtime.".to_string())
}
