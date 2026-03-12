use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::models::{PublicUser, UpdateProfile, User};

const MAX_DISPLAY_NAME_LEN: usize = 64;
const MAX_BIO_LEN: usize = 280;
const MAX_AVATAR_URL_LEN: usize = 2048;

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
        "coin_balance": user.coin_balance,
        "wins": user.wins,
        "losses": user.losses,
        "total_wagered": user.total_wagered,
        "created_at": user.created_at,
    }))
}

/// PATCH /api/users/me
pub async fn update_me(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(body): Json<UpdateProfile>,
) -> Result<impl IntoResponse, StatusCode> {
    let update = normalize_profile_update(body)?;

    let updated = sqlx::query_as::<_, User>(
        r#"
        UPDATE users SET
            display_name = COALESCE($1, display_name),
            bio = COALESCE($2, bio),
            avatar_url = COALESCE($3, avatar_url),
            updated_at = NOW()
        WHERE id = $4
        RETURNING *
        "#,
    )
    .bind(&update.display_name)
    .bind(&update.bio)
    .bind(&update.avatar_url)
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
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
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
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Query(params): Query<SearchQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let q = params.q.unwrap_or_default().trim().to_string();
    if q.len() < 2 {
        return Ok(Json(serde_json::json!([])));
    }

    let pattern = format!("%{q}%");

    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT *
        FROM users u
        WHERE u.id != $1
          AND (u.username ILIKE $2 OR u.display_name ILIKE $2)
          AND NOT EXISTS (
              SELECT 1
              FROM friendships f
              WHERE (
                  (f.requester_id = $1 AND f.addressee_id = u.id)
                  OR (f.addressee_id = $1 AND f.requester_id = u.id)
              )
          )
        ORDER BY u.username
        LIMIT 20
        "#,
    )
    .bind(user.id)
    .bind(&pattern)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let public: Vec<PublicUser> = users.into_iter().map(PublicUser::from).collect();
    Ok(Json(serde_json::to_value(public).unwrap()))
}

fn normalize_profile_update(body: UpdateProfile) -> Result<UpdateProfile, StatusCode> {
    let display_name = match body.display_name {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() || trimmed.len() > MAX_DISPLAY_NAME_LEN {
                return Err(StatusCode::BAD_REQUEST);
            }
            Some(trimmed.to_string())
        }
        None => None,
    };

    let bio = match body.bio {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.len() > MAX_BIO_LEN {
                return Err(StatusCode::BAD_REQUEST);
            }
            Some(trimmed.to_string())
        }
        None => None,
    };

    let avatar_url = match body.avatar_url {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.len() > MAX_AVATAR_URL_LEN {
                return Err(StatusCode::BAD_REQUEST);
            }
            Some(trimmed.to_string())
        }
        None => None,
    };

    Ok(UpdateProfile {
        display_name,
        bio,
        avatar_url,
    })
}
