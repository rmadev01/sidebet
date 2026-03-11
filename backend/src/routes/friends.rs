use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{FriendRequest, FriendSummary, Friendship, IncomingFriendRequest, PublicUser, User};

/// GET /api/friends — list accepted friends
pub async fn list_friends(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    let friends = sqlx::query_as::<_, FriendSummary>(
        r#"
        SELECT
            f.id AS friendship_id,
            u.id AS user_id,
            u.username,
            u.display_name,
            u.avatar_url,
            u.bio,
            u.wins,
            u.losses
        FROM friendships f
        JOIN users u ON u.id = CASE
            WHEN f.requester_id = $1 THEN f.addressee_id
            ELSE f.requester_id
        END
        WHERE
            (f.requester_id = $1 OR f.addressee_id = $1)
            AND f.status = 'accepted'
        ORDER BY u.username
        "#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(friends))
}

/// GET /api/friends/requests — incoming pending requests
pub async fn incoming_requests(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    let requests = sqlx::query_as::<_, IncomingFriendRequest>(
        r#"
        SELECT
            f.id AS friendship_id,
            f.requester_id,
            u.username,
            u.display_name,
            u.avatar_url,
            u.bio,
            u.wins,
            u.losses,
            f.created_at
        FROM friendships f
        JOIN users u ON u.id = f.requester_id
        WHERE f.addressee_id = $1 AND f.status = 'pending'
        ORDER BY f.created_at DESC
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

    let target = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(body.user_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let existing = sqlx::query_as::<_, Friendship>(
        r#"
        SELECT *
        FROM friendships
        WHERE LEAST(requester_id, addressee_id) = LEAST($1, $2)
          AND GREATEST(requester_id, addressee_id) = GREATEST($1, $2)
        FOR UPDATE
        "#,
    )
    .bind(user.id)
    .bind(body.user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (status_code, friendship_id, status) = match existing {
        Some(friendship) if friendship.status == "accepted" => return Err(StatusCode::CONFLICT),
        Some(friendship) if friendship.requester_id == user.id => return Err(StatusCode::CONFLICT),
        Some(friendship) => {
            sqlx::query(
                "UPDATE friendships SET status = 'accepted' WHERE id = $1 AND status = 'pending'",
            )
            .bind(friendship.id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (StatusCode::OK, friendship.id, "accepted")
        }
        None => {
            let friendship_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO friendships (id, requester_id, addressee_id, status, created_at)
                VALUES ($1, $2, $3, 'pending', NOW())
                "#,
            )
            .bind(friendship_id)
            .bind(user.id)
            .bind(body.user_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            (StatusCode::CREATED, friendship_id, "pending")
        }
    };

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let payload = serde_json::json!({
        "friendship_id": friendship_id,
        "status": status,
        "user": PublicUser::from(target),
    });

    Ok((status_code, Json(payload)))
}

/// POST /api/friends/:id/accept
pub async fn accept_request(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query_as::<_, FriendSummary>(
        r#"
        UPDATE friendships f
        SET status = 'accepted'
        FROM users u
        WHERE f.id = $1
          AND f.addressee_id = $2
          AND f.status = 'pending'
          AND u.id = f.requester_id
        RETURNING
            f.id AS friendship_id,
            u.id AS user_id,
            u.username,
            u.display_name,
            u.avatar_url,
            u.bio,
            u.wins,
            u.losses
        "#,
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Some(friend) => Ok((StatusCode::OK, Json(friend)).into_response()),
        None => Err(StatusCode::NOT_FOUND),
    }
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

    Ok(StatusCode::NO_CONTENT)
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

    Ok(StatusCode::NO_CONTENT)
}
