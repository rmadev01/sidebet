use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
    timestamp: chrono::DateTime<Utc>,
}

/// GET /health
pub async fn health(Extension(pool): Extension<PgPool>) -> impl IntoResponse {
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
        .is_ok();

    let status = if db_ok { "ok" } else { "degraded" };
    let database = if db_ok { "ok" } else { "error" };
    let response_status = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        response_status,
        Json(HealthResponse {
            status,
            database,
            timestamp: Utc::now(),
        }),
    )
}
