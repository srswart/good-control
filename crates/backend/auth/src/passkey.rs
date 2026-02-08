//! Passkey (WebAuthn) support.

use crate::error::{AuthError, Result};
use crate::user::{SqliteUserRepository, User, UserRepository};
use persistence::DbPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct PasskeyRecord {
    id: String,
    user_id: i64,
    passkey_json: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
struct PasskeyChallengeRecord {
    id: String,
    user_id: Option<i64>,
    challenge_type: String,
    state_json: String,
    expires_at: String,
    created_at: String,
}

fn webauthn() -> Result<Webauthn> {
    let rp_id = std::env::var("APP__WEBAUTHN_RP_ID")
        .map_err(|_| AuthError::Configuration("APP__WEBAUTHN_RP_ID is required".to_string()))?;
    let rp_origin = std::env::var("APP__WEBAUTHN_RP_ORIGIN")
        .map_err(|_| AuthError::Configuration("APP__WEBAUTHN_RP_ORIGIN is required".to_string()))?;
    let rp_name = std::env::var("APP__WEBAUTHN_RP_NAME")
        .unwrap_or_else(|_| "Inventory".to_string());

    let origin = Url::parse(&rp_origin)
        .map_err(|e| AuthError::Configuration(format!("Invalid APP__WEBAUTHN_RP_ORIGIN: {}", e)))?;

    let builder = WebauthnBuilder::new(&rp_id, &origin)
        .map_err(|e| AuthError::Configuration(format!("Invalid WebAuthn config: {}", e)))?
        .rp_name(&rp_name);

    builder
        .build()
        .map_err(|e| AuthError::Configuration(format!("Invalid WebAuthn config: {}", e)))
}

fn expiry_timestamp(seconds: i64) -> String {
    (chrono::Utc::now() + chrono::Duration::seconds(seconds)).to_rfc3339()
}

async fn store_challenge(
    pool: &DbPool,
    user_id: Option<i64>,
    challenge_type: &str,
    state_json: &str,
) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let expires_at = expiry_timestamp(300);

    sqlx::query(
        r#"INSERT INTO passkey_challenges (id, user_id, challenge_type, state_json, expires_at)
           VALUES (?, ?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(user_id)
    .bind(challenge_type)
    .bind(state_json)
    .bind(&expires_at)
    .execute(pool.inner())
    .await?;

    Ok(id)
}

async fn take_challenge(
    pool: &DbPool,
    challenge_id: &str,
    challenge_type: &str,
) -> Result<PasskeyChallengeRecord> {
    let record = sqlx::query_as::<_, PasskeyChallengeRecord>(
        r#"SELECT id, user_id, challenge_type, state_json, expires_at, created_at
           FROM passkey_challenges WHERE id = ? AND challenge_type = ?"#,
    )
    .bind(challenge_id)
    .bind(challenge_type)
    .fetch_optional(pool.inner())
    .await?
    .ok_or(AuthError::InvalidToken)?;

    let expires = chrono::DateTime::parse_from_rfc3339(&record.expires_at)
        .map_err(|_| AuthError::InvalidToken)?;
    if chrono::Utc::now() > expires.with_timezone(&chrono::Utc) {
        sqlx::query("DELETE FROM passkey_challenges WHERE id = ?")
            .bind(&record.id)
            .execute(pool.inner())
            .await?;
        return Err(AuthError::TokenExpired);
    }

    sqlx::query("DELETE FROM passkey_challenges WHERE id = ?")
        .bind(&record.id)
        .execute(pool.inner())
        .await?;

    Ok(record)
}

async fn load_passkeys(pool: &DbPool, user_id: i64) -> Result<Vec<(String, Passkey)>> {
    let records = sqlx::query_as::<_, PasskeyRecord>(
        r#"SELECT id, user_id, passkey_json, created_at, updated_at
           FROM passkeys WHERE user_id = ?"#,
    )
    .bind(user_id)
    .fetch_all(pool.inner())
    .await?;

    let mut passkeys = Vec::new();
    for record in records {
        let passkey: Passkey = serde_json::from_str(&record.passkey_json)
            .map_err(|e| AuthError::Internal(format!("Failed to parse passkey: {}", e)))?;
        passkeys.push((record.id, passkey));
    }

    Ok(passkeys)
}

async fn save_passkey(pool: &DbPool, user_id: i64, passkey: &Passkey) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    let passkey_json = serde_json::to_string(passkey)
        .map_err(|e| AuthError::Internal(format!("Failed to serialize passkey: {}", e)))?;

    sqlx::query(
        r#"INSERT INTO passkeys (id, user_id, passkey_json)
           VALUES (?, ?, ?)"#,
    )
    .bind(&id)
    .bind(user_id)
    .bind(passkey_json)
    .execute(pool.inner())
    .await?;

    Ok(())
}

async fn update_passkey(pool: &DbPool, passkey_id: &str, passkey: &Passkey) -> Result<()> {
    let passkey_json = serde_json::to_string(passkey)
        .map_err(|e| AuthError::Internal(format!("Failed to serialize passkey: {}", e)))?;

    sqlx::query(
        r#"UPDATE passkeys SET passkey_json = ?, updated_at = datetime('now') WHERE id = ?"#,
    )
    .bind(passkey_json)
    .bind(passkey_id)
    .execute(pool.inner())
    .await?;

    Ok(())
}

pub async fn begin_passkey_registration(pool: &DbPool, user: &User) -> Result<(String, CreationChallengeResponse)> {
    let webauthn = webauthn()?;
    let existing = load_passkeys(pool, user.id).await?;
    let exclude = if existing.is_empty() {
        None
    } else {
        Some(existing.into_iter().map(|(_, p)| p.cred_id().clone()).collect())
    };

    let user_uuid = Uuid::from_u128(user.id as u128);
    let (ccr, reg_state) = webauthn
        .start_passkey_registration(
            user_uuid,
            &user.username,
            &user.username,
            exclude,
        )
        .map_err(|e| AuthError::Internal(format!("Failed to start passkey registration: {}", e)))?;

    let state_json = serde_json::to_string(&reg_state)
        .map_err(|e| AuthError::Internal(format!("Failed to serialize passkey state: {}", e)))?;

    let challenge_id = store_challenge(pool, Some(user.id), "registration", &state_json).await?;

    Ok((challenge_id, ccr))
}

pub async fn finish_passkey_registration(
    pool: &DbPool,
    user: &User,
    challenge_id: &str,
    credential_json: &str,
) -> Result<()> {
    let webauthn = webauthn()?;
    let challenge = take_challenge(pool, challenge_id, "registration").await?;

    let state: PasskeyRegistration = serde_json::from_str(&challenge.state_json)
        .map_err(|e| AuthError::Internal(format!("Failed to parse passkey state: {}", e)))?;
    let reg: RegisterPublicKeyCredential = serde_json::from_str(credential_json)
        .map_err(|e| AuthError::InvalidToken)?;

    let passkey = webauthn
        .finish_passkey_registration(&reg, &state)
        .map_err(|e| AuthError::InvalidToken)?;

    let existing = load_passkeys(pool, user.id).await?;
    if existing
        .iter()
        .any(|(_, existing_passkey)| existing_passkey.cred_id() == passkey.cred_id())
    {
        return Err(AuthError::InvalidToken);
    }

    save_passkey(pool, user.id, &passkey).await?;
    Ok(())
}

pub async fn begin_passkey_authentication(
    pool: &DbPool,
    username_or_email: &str,
) -> Result<(String, RequestChallengeResponse)> {
    let webauthn = webauthn()?;
    let user_repo = SqliteUserRepository::new(pool.clone());

    let user = if let Some(user) = user_repo.find_by_username(username_or_email).await? {
        Some(user)
    } else {
        user_repo.find_by_email(username_or_email).await?
    }
    .ok_or(AuthError::InvalidCredentials)?;

    if !user.is_active {
        return Err(AuthError::InvalidCredentials);
    }

    let passkeys = load_passkeys(pool, user.id).await?;
    if passkeys.is_empty() {
        return Err(AuthError::InvalidCredentials);
    }

    let creds: Vec<Passkey> = passkeys.into_iter().map(|(_, p)| p).collect();

    let (rcr, auth_state) = webauthn
        .start_passkey_authentication(&creds)
        .map_err(|e| AuthError::Internal(format!("Failed to start passkey auth: {}", e)))?;

    let state_json = serde_json::to_string(&auth_state)
        .map_err(|e| AuthError::Internal(format!("Failed to serialize passkey state: {}", e)))?;

    let challenge_id = store_challenge(pool, Some(user.id), "authentication", &state_json).await?;

    Ok((challenge_id, rcr))
}

pub async fn finish_passkey_authentication(
    pool: &DbPool,
    challenge_id: &str,
    credential_json: &str,
) -> Result<User> {
    let webauthn = webauthn()?;
    let challenge = take_challenge(pool, challenge_id, "authentication").await?;

    let user_id = challenge.user_id.ok_or(AuthError::InvalidCredentials)?;
    let user_repo = SqliteUserRepository::new(pool.clone());
    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    let state: PasskeyAuthentication = serde_json::from_str(&challenge.state_json)
        .map_err(|e| AuthError::Internal(format!("Failed to parse passkey state: {}", e)))?;
    let reg: PublicKeyCredential = serde_json::from_str(credential_json)
        .map_err(|_| AuthError::InvalidToken)?;

    let mut passkeys = load_passkeys(pool, user.id).await?;
    let creds: Vec<Passkey> = passkeys.iter().map(|(_, p)| p.clone()).collect();

    let auth_result = webauthn
        .finish_passkey_authentication(&reg, &state)
        .map_err(|_| AuthError::InvalidCredentials)?;

    for (passkey_id, mut passkey) in passkeys {
        if let Some(updated) = passkey.update_credential(&auth_result) {
            if updated {
                update_passkey(pool, &passkey_id, &passkey).await?;
            }
            break;
        }
    }

    Ok(user)
}
