use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::PgPool;

use crate::models::{Notification, User};

/// GET /api/notifications — unread notifications
pub async fn get_notifications(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    let notifications = sqlx::query_as::<_, Notification>(
        r#"
        SELECT * FROM notifications
        WHERE user_id = $1 AND read = false
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(notifications))
}

/// POST /api/notifications/read — mark all as read
pub async fn mark_read(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    sqlx::query("UPDATE notifications SET read = true WHERE user_id = $1 AND read = false")
        .bind(user.id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
