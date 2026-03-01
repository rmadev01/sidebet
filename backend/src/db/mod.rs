use axum::extract::{Extension, Request};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use sqlx::PgPool;

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
    let row = sqlx::query_as::<_, (String, String)>(
        r#"
        SELECT s."userId", u."email"
        FROM "session" s
        JOIN "user" u ON u."id" = s."userId"
        WHERE s."token" = $1 AND s."expiresAt" > NOW()
        "#,
    )
    .bind(&token)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (auth_user_id, _email) = match row {
        Some(r) => r,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Find the corresponding sidebet user (linked by auth user id).
    // We store the Better Auth user id in a column or use email linkage.
    // For simplicity, we match on auth_user_id stored in users table or
    // auto-create on first authenticated request.
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id::text = $1 OR username = $1",
    )
    .bind(&auth_user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = match user {
        Some(u) => u,
        None => {
            // Auto-provision a sidebet user for the authenticated auth user
            let id = uuid::Uuid::parse_str(&auth_user_id).unwrap_or_else(|_| uuid::Uuid::new_v4());
            sqlx::query_as::<_, User>(
                r#"
                INSERT INTO users (id, username, display_name, created_at, updated_at)
                VALUES ($1, $2, $3, NOW(), NOW())
                RETURNING *
                "#,
            )
            .bind(id)
            .bind(&auth_user_id)
            .bind("New User")
            .fetch_one(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
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
