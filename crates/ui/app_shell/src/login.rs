//! Login and passkey registration UI.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use ui_foundation::graphql::{graphql_endpoint, GraphQLClient, GraphQLError};
use ui_foundation::graphql::auth_token::set_auth_token;

use crate::routes::Route;
use crate::session::{Session, use_session};
use crate::passkey_web::{webauthn_authenticate, webauthn_register};

#[derive(Debug, Deserialize, Serialize)]
struct PasskeyChallengeResponse {
    #[serde(rename = "beginPasskeyAuthentication")]
    challenge: PasskeyChallenge,
}

#[derive(Debug, Deserialize, Serialize)]
struct PasskeyRegistrationResponse {
    #[serde(rename = "beginPasskeyRegistration")]
    challenge: PasskeyChallenge,
}

#[derive(Debug, Deserialize, Serialize)]
struct PasskeyChallenge {
    #[serde(rename = "challengeId")]
    challenge_id: String,
    #[serde(rename = "optionsJson")]
    options_json: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FinishAuthResponse {
    #[serde(rename = "finishPasskeyAuthentication")]
    login: LoginResponse,
}

#[derive(Debug, Deserialize, Serialize)]
struct PasswordLoginResponse {
    login: LoginResponse,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginResponse {
    user: LoginUser,
    token: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginUser {
    id: String,
    username: String,
    roles: Vec<LoginRole>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginRole {
    name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FinishRegistrationResponse {
    #[serde(rename = "finishPasskeyRegistration")]
    ok: bool,
}

fn handle_login_response(
    response: LoginResponse,
    session_signal: &mut Signal<Session>,
) {
    let roles = response.user.roles.into_iter().map(|role| role.name).collect();
    set_auth_token(Some(&response.token));
    session_signal.set(Session::authenticated(
        response.user.id,
        response.user.username,
        roles,
        response.token,
    ));
}

#[component]
pub fn LoginView() -> Element {
    let session_signal = use_session();
    let navigator = dioxus_router::use_navigator();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let mut use_passkey = use_signal(|| false);

    // If already authenticated, redirect to home
    use_effect(move || {
        if session_signal.read().is_authenticated {
            navigator.push(Route::Home {});
        }
    });

    // Password login handler
    let on_password_login = move |_| {
        let username_value = username();
        let password_value = password();
        let mut session_signal = session_signal.clone();
        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());
            let query = r#"
                mutation Login($username: String!, $password: String!) {
                    login(username: $username, password: $password) {
                        token
                        user {
                            id
                            username
                            roles { name }
                        }
                    }
                }
            "#;
            let variables = serde_json::json!({
                "username": username_value,
                "password": password_value,
            });

            match client.query::<PasswordLoginResponse>(query, Some(variables)).await {
                Ok(response) => {
                    handle_login_response(response.login, &mut session_signal);
                }
                Err(err) => {
                    error.set(Some(format!("Login failed: {}", err)));
                }
            }

            loading.set(false);
        });
    };

    // Passkey login handler
    let on_passkey_login = move |_| {
        let username_value = username();
        let mut session_signal = session_signal.clone();
        spawn(async move {
            loading.set(true);
            error.set(None);

            let client = GraphQLClient::new(graphql_endpoint());
            let begin_query = r#"
                mutation BeginPasskeyAuth($username: String!) {
                    beginPasskeyAuthentication(usernameOrEmail: $username) {
                        challengeId
                        optionsJson
                    }
                }
            "#;
            let variables = serde_json::json!({ "username": username_value });

            let challenge = match client.query::<PasskeyChallengeResponse>(begin_query, Some(variables)).await {
                Ok(response) => response.challenge,
                Err(err) => {
                    error.set(Some(format!("Login failed: {}", err)));
                    loading.set(false);
                    return;
                }
            };

            let credential_json = match webauthn_authenticate(&challenge.options_json).await {
                Ok(json) => json,
                Err(err) => {
                    error.set(Some(err));
                    loading.set(false);
                    return;
                }
            };

            let finish_query = r#"
                mutation FinishPasskeyAuth($challengeId: String!, $credentialJson: String!) {
                    finishPasskeyAuthentication(challengeId: $challengeId, credentialJson: $credentialJson) {
                        token
                        user {
                            id
                            username
                            roles { name }
                        }
                    }
                }
            "#;
            let variables = serde_json::json!({
                "challengeId": challenge.challenge_id,
                "credentialJson": credential_json,
            });

            match client.query::<FinishAuthResponse>(finish_query, Some(variables)).await {
                Ok(response) => {
                    handle_login_response(response.login, &mut session_signal);
                }
                Err(err) => {
                    error.set(Some(format!("Login failed: {}", err)));
                }
            }

            loading.set(false);
        });
    };

    rsx! {
        div {
            class: "max-w-md mx-auto mt-16 bg-white rounded-xl shadow-lg p-8",
            h2 { class: "text-2xl font-semibold mb-4 text-gray-800", "Sign in" }

            input {
                class: "w-full border rounded px-3 py-2 mb-4",
                r#type: "text",
                placeholder: "Username or email",
                value: "{username}",
                oninput: move |evt| username.set(evt.value()),
            }

            if !use_passkey() {
                // Password login (default)
                input {
                    class: "w-full border rounded px-3 py-2 mb-4",
                    r#type: "password",
                    placeholder: "Password",
                    value: "{password}",
                    oninput: move |evt| password.set(evt.value()),
                }
                button {
                    class: "w-full bg-blue-600 text-white rounded py-2 text-sm font-medium",
                    disabled: loading(),
                    onclick: on_password_login,
                    if loading() { "Signing in..." } else { "Sign in" }
                }
                p {
                    class: "text-xs text-gray-500 mt-4 text-center cursor-pointer hover:text-blue-600",
                    onclick: move |_| use_passkey.set(true),
                    "Use passkey instead"
                }
            } else {
                // Passkey login
                p { class: "text-sm text-gray-600 mb-4", "Use your passkey to sign in." }
                button {
                    class: "w-full bg-blue-600 text-white rounded py-2 text-sm font-medium",
                    disabled: loading(),
                    onclick: on_passkey_login,
                    if loading() { "Signing in..." } else { "Sign in with passkey" }
                }
                p {
                    class: "text-xs text-gray-500 mt-4 text-center cursor-pointer hover:text-blue-600",
                    onclick: move |_| use_passkey.set(false),
                    "Use password instead"
                }
            }

            if let Some(message) = error() {
                p { class: "text-sm text-red-600 mt-4", "{message}" }
            }
        }
    }
}

#[component]
pub fn PasskeyRegisterButton() -> Element {
    let session_signal = use_session();
    let mut error = use_signal(|| None::<String>);
    let mut success = use_signal(|| false);
    let mut loading = use_signal(|| false);

    let on_register = move |_| {
        let mut session_signal = session_signal.clone();
        spawn(async move {
            loading.set(true);
            error.set(None);
            success.set(false);

            let client = GraphQLClient::new(graphql_endpoint());
            let begin_query = r#"
                mutation BeginPasskeyRegistration {
                    beginPasskeyRegistration {
                        challengeId
                        optionsJson
                    }
                }
            "#;

            let challenge = match client.query::<PasskeyRegistrationResponse>(begin_query, None).await {
                Ok(response) => response.challenge,
                Err(err) => {
                    error.set(Some(format!("Passkey registration failed: {}", err)));
                    loading.set(false);
                    return;
                }
            };

            let credential_json = match webauthn_register(&challenge.options_json).await {
                Ok(json) => json,
                Err(err) => {
                    error.set(Some(err));
                    loading.set(false);
                    return;
                }
            };

            let finish_query = r#"
                mutation FinishPasskeyRegistration($challengeId: String!, $credentialJson: String!) {
                    finishPasskeyRegistration(challengeId: $challengeId, credentialJson: $credentialJson)
                }
            "#;
            let variables = serde_json::json!({
                "challengeId": challenge.challenge_id,
                "credentialJson": credential_json,
            });

            match client.query::<FinishRegistrationResponse>(finish_query, Some(variables)).await {
                Ok(_) => {
                    success.set(true);
                }
                Err(GraphQLError::Http { status, .. }) if status == 401 || status == 403 => {
                    session_signal.set(Session::new());
                    set_auth_token(None);
                }
                Err(err) => {
                    error.set(Some(format!("Passkey registration failed: {}", err)));
                }
            }

            loading.set(false);
        });
    };

    rsx! {
        div {
            class: "flex flex-col items-end",
            button {
                class: "text-xs text-white bg-emerald-600 rounded px-3 py-2",
                disabled: loading(),
                onclick: on_register,
                if loading() { "Registering..." } else { "Register passkey" }
            }
            if success() {
                span { class: "text-xs text-emerald-200 mt-1", "Passkey registered!" }
            }
            if let Some(message) = error() {
                span { class: "text-xs text-red-200 mt-1", "{message}" }
            }
        }
    }
}
