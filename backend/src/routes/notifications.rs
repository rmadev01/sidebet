use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use sqlx::PgPool;

use crate::{models::User, services::notifications};

/// GET /api/notifications — unread notifications
pub async fn get_notifications(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    let notifications = notifications::get_unread(&pool, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(notifications))
}

/// POST /api/notifications/read — mark all as read
pub async fn mark_read(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    notifications::mark_read(&pool, user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
