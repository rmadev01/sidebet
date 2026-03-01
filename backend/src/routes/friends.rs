use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{FriendRequest, Friendship, PublicUser, User};

/// GET /api/friends — list accepted friends
pub async fn list_friends(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    let friends = sqlx::query_as::<_, User>(
        r#"
        SELECT u.* FROM users u
        JOIN friendships f ON
            (f.requester_id = u.id OR f.addressee_id = u.id)
        WHERE
            (f.requester_id = $1 OR f.addressee_id = $1)
            AND u.id != $1
            AND f.status = 'accepted'
        ORDER BY u.username
        "#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let public: Vec<PublicUser> = friends.into_iter().map(PublicUser::from).collect();
    Ok(Json(public))
}

/// GET /api/friends/requests — incoming pending requests
pub async fn incoming_requests(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    let requests = sqlx::query_as::<_, Friendship>(
        r#"
        SELECT * FROM friendships
        WHERE addressee_id = $1 AND status = 'pending'
        ORDER BY created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(requests))
}

/// POST /api/friends/request — { user_id }
pub async fn send_request(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(body): Json<FriendRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if body.user_id == user.id {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check if friendship already exists
    let existing = sqlx::query_as::<_, Friendship>(
        r#"
        SELECT * FROM friendships
        WHERE (requester_id = $1 AND addressee_id = $2)
           OR (requester_id = $2 AND addressee_id = $1)
        "#,
    )
    .bind(user.id)
    .bind(body.user_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if existing.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let friendship = sqlx::query_as::<_, Friendship>(
        r#"
        INSERT INTO friendships (id, requester_id, addressee_id, status, created_at)
        VALUES ($1, $2, $3, 'pending', NOW())
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(body.user_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(friendship)))
}

/// POST /api/friends/:id/accept
pub async fn accept_request(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query(
        r#"
        UPDATE friendships SET status = 'accepted'
        WHERE id = $1 AND addressee_id = $2 AND status = 'pending'
        "#,
    )
    .bind(id)
    .bind(user.id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

/// POST /api/friends/:id/decline
pub async fn decline_request(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM friendships WHERE id = $1 AND addressee_id = $2 AND status = 'pending'",
    )
    .bind(id)
    .bind(user.id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

/// DELETE /api/friends/:id
pub async fn remove_friend(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query(
        r#"
        DELETE FROM friendships
        WHERE id = $1
          AND (requester_id = $2 OR addressee_id = $2)
          AND status = 'accepted'
        "#,
    )
    .bind(id)
    .bind(user.id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}
