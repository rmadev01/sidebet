use axum::extract::{Extension, Request};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User;

/// Extract session token from cookie and validate against the Better Auth
/// session table in PostgreSQL. Injects the authenticated `User` into
/// request extensions.
pub async fn auth_middleware(
    Extension(pool): Extension<PgPool>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let cookie_header = req
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let token = parse_cookie(cookie_header, "better-auth.session_token")
        .or_else(|| parse_cookie(cookie_header, "__Secure-better-auth.session_token"));

    let token = match token {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Query Better Auth's session + user tables
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
        Some(r) => r,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Find the corresponding sidebet user by auth_user_id parsed as UUID.
    let auth_uuid = stable_user_uuid(&auth_user_id);

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(auth_uuid)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = match user {
        Some(u) => u,
        None => {
            // Auto-provision: choose a stable, conflict-free username.
            let username_seed = email.split('@').next().unwrap_or("user");
            let username = next_available_username(&pool, username_seed).await?;
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
            .fetch_one(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
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
        if let Some(value) = pair.strip_prefix(&format!("{name}=")) {
            Some(value.to_string())
        } else {
            None
        }
    })
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
