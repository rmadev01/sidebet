use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::Event;

#[derive(Deserialize)]
pub struct EventFilters {
    pub category: Option<String>,
    pub status: Option<String>,
}

/// GET /api/events
pub async fn list_events(
    Extension(pool): Extension<PgPool>,
    Query(filters): Query<EventFilters>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut query = String::from("SELECT * FROM events WHERE 1=1");
    let mut params: Vec<String> = Vec::new();

    if let Some(ref cat) = filters.category {
        params.push(cat.clone());
        query.push_str(&format!(" AND category = ${}", params.len()));
    }
    if let Some(ref status) = filters.status {
        params.push(status.clone());
        query.push_str(&format!(" AND status = ${}", params.len()));
    }
    query.push_str(" ORDER BY starts_at ASC NULLS LAST LIMIT 50");

    // Build dynamic query
    let mut q = sqlx::query_as::<_, Event>(&query);
    for p in &params {
        q = q.bind(p);
    }

    let events = q
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(events))
}

/// GET /api/events/:id
pub async fn get_event(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match event {
        Some(e) => Ok(Json(e)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// GET /api/events/:id/odds — latest odds (refresh if stale)
pub async fn get_event_odds(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match event {
        Some(e) => Ok(Json(serde_json::json!({
            "event_id": e.id,
            "cached_odds": e.cached_odds,
            "odds_updated_at": e.odds_updated_at,
        }))),
        None => Err(StatusCode::NOT_FOUND),
    }
}
