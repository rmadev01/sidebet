use axum::extract::{Extension, Request};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use http::header::{COOKIE, ORIGIN, REFERER};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::models::User;

pub async fn auth_middleware(
    Extension(pool): Extension<PgPool>,
    Extension(cfg): Extension<Config>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    enforce_origin(&cfg, req.headers(), req.method().as_str())?;

    let token = extract_session_token(req.headers()).ok_or(StatusCode::UNAUTHORIZED)?;

    let row = sqlx::query_as::<_, (String, String, String)>(
        r#"
        SELECT s."userId", u."email", u."name"
        FROM "session" s
        JOIN "user" u ON u."id" = s."userId"
        WHERE s."token" = $1 AND s."expiresAt" > NOW()
        "#,
    )
    .bind(&token)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (auth_user_id, email, name) = match row {
        Some(row) => row,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let auth_uuid = stable_user_uuid(&auth_user_id);

    let user = match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(auth_uuid)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        Some(user) => user,
        None => provision_user(&pool, auth_uuid, &email, &name).await?,
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

async fn provision_user(
    pool: &PgPool,
    auth_uuid: Uuid,
    email: &str,
    name: &str,
) -> Result<User, StatusCode> {
    let username_seed = email.split('@').next().unwrap_or("user");
    let username = next_available_username(pool, username_seed).await?;
    let display_name = if name.trim().is_empty() {
        username.clone()
    } else {
        name.trim().to_string()
    };

    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, display_name, created_at, updated_at)
        VALUES ($1, $2, $3, NOW(), NOW())
        ON CONFLICT (id) DO UPDATE SET updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(auth_uuid)
    .bind(&username)
    .bind(&display_name)
    .fetch_one(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query(
        "INSERT INTO transactions (id, user_id, type, amount, balance_after, created_at) VALUES ($1, $2, 'signup_bonus', 10000, 10000, NOW()) ON CONFLICT DO NOTHING",
    )
    .bind(uuid::Uuid::new_v4())
    .bind(auth_uuid)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(auth_uuid)
        .fetch_one(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(COOKIE).and_then(|value| value.to_str().ok())?;
    parse_cookie(cookie_header, "better-auth.session_token")
        .or_else(|| parse_cookie(cookie_header, "__Secure-better-auth.session_token"))
        .map(|token| normalize_session_token(&token))
}

fn enforce_origin(cfg: &Config, headers: &HeaderMap, method: &str) -> Result<(), StatusCode> {
    if matches!(method, "GET" | "HEAD" | "OPTIONS") {
        return Ok(());
    }

    let allowed = cfg.allowed_frontend_origins();
    if let Some(origin) = header_value(headers, &ORIGIN) {
        if allowed.iter().any(|candidate| candidate == origin) {
            return Ok(());
        }

        return Err(StatusCode::FORBIDDEN);
    }

    if let Some(referer) = header_value(headers, &REFERER) {
        if allowed.iter().any(|candidate| referer.starts_with(candidate)) {
            return Ok(());
        }

        return Err(StatusCode::FORBIDDEN);
    }

    Err(StatusCode::FORBIDDEN)
}

fn header_value<'a>(headers: &'a HeaderMap, name: &'static http::header::HeaderName) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

fn stable_user_uuid(auth_user_id: &str) -> Uuid {
    Uuid::parse_str(auth_user_id).unwrap_or_else(|_| {
        Uuid::new_v5(
            &Uuid::NAMESPACE_URL,
            format!("sidebet:auth-user:{auth_user_id}").as_bytes(),
        )
    })
}

fn normalize_username(seed: &str) -> String {
    let cleaned: String = seed
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();

    let mut normalized = cleaned.trim_matches('_').to_string();
    if normalized.is_empty() {
        normalized = "user".into();
    }

    if normalized.len() > 32 {
        normalized.truncate(32);
    }

    normalized
}

fn with_suffix(base: &str, index: usize) -> String {
    if index == 0 {
        return base.to_string();
    }

    let suffix = format!("_{index}");
    let max_base_len = 32usize.saturating_sub(suffix.len());
    let mut root = base.to_string();
    if root.len() > max_base_len {
        root.truncate(max_base_len);
    }

    format!("{root}{suffix}")
}

async fn next_available_username(pool: &PgPool, seed: &str) -> Result<String, StatusCode> {
    let base = normalize_username(seed);

    for i in 0..1_000 {
        let candidate = with_suffix(&base, i);
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
                .bind(&candidate)
                .fetch_one(pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if !exists {
            return Ok(candidate);
        }
    }

    Err(StatusCode::INTERNAL_SERVER_ERROR)
}

fn parse_cookie(header: &str, name: &str) -> Option<String> {
    header.split(';').find_map(|pair| {
        let pair = pair.trim();
        pair.strip_prefix(&format!("{name}=")).map(|value| value.to_string())
    })
}

fn normalize_session_token(token: &str) -> String {
    token.split('.').next().unwrap_or(token).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cookie_by_name() {
        let header = "a=1; better-auth.session_token=abc123; b=2";
        assert_eq!(
            parse_cookie(header, "better-auth.session_token"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn strips_signed_cookie_suffix_before_session_lookup() {
        let token = normalize_session_token("abc123.signature%2Bpayload");
        assert_eq!(token, "abc123");
    }

    #[test]
    fn stable_uuid_is_deterministic_for_non_uuid_ids() {
        let a = stable_user_uuid("user_123");
        let b = stable_user_uuid("user_123");
        assert_eq!(a, b);
    }

    #[test]
    fn normalizes_username_and_enforces_limit() {
        let normalized = normalize_username("  A B.C+D@x  ");
        assert_eq!(normalized, "a_b_c_d_x");

        let very_long = normalize_username("abcdefghijklmnopqrstuvwxyz1234567890");
        assert_eq!(very_long.len(), 32);
    }
}
