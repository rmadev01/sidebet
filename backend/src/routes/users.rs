use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{PublicUser, UpdateProfile, User};

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

/// GET /api/users/me
pub async fn get_me(Extension(user): Extension<User>) -> impl IntoResponse {
    Json(serde_json::json!({
        "id": user.id,
        "username": user.username,
        "display_name": user.display_name,
        "avatar_url": user.avatar_url,
        "bio": user.bio,
        "wallet_address": user.wallet_address,
        "wins": user.wins,
        "losses": user.losses,
        "total_wagered_wei": user.total_wagered_wei,
        "created_at": user.created_at,
    }))
}

/// PATCH /api/users/me
pub async fn update_me(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(body): Json<UpdateProfile>,
) -> Result<impl IntoResponse, StatusCode> {
    let updated = sqlx::query_as::<_, User>(
        r#"
        UPDATE users SET
            display_name = COALESCE($1, display_name),
            bio = COALESCE($2, bio),
            avatar_url = COALESCE($3, avatar_url),
            wallet_address = COALESCE($4, wallet_address),
            updated_at = NOW()
        WHERE id = $5
        RETURNING *
        "#,
    )
    .bind(&body.display_name)
    .bind(&body.bio)
    .bind(&body.avatar_url)
    .bind(&body.wallet_address)
    .bind(user.id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::to_value(updated).unwrap()))
}

/// GET /api/users/:username
pub async fn get_user_by_username(
    Extension(pool): Extension<PgPool>,
    Path(username): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = $1",
    )
    .bind(&username)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(u) => Ok(Json(serde_json::to_value(PublicUser::from(u)).unwrap())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// GET /api/users/search?q=
pub async fn search_users(
    Extension(pool): Extension<PgPool>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let q = params.q.unwrap_or_default();
    let pattern = format!("%{q}%");

    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username ILIKE $1 OR display_name ILIKE $1 LIMIT 20",
    )
    .bind(&pattern)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let public: Vec<PublicUser> = users.into_iter().map(PublicUser::from).collect();
    Ok(Json(serde_json::to_value(public).unwrap()))
}
