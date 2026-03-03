use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::models::Event;

#[derive(Deserialize)]
pub struct EventFilters {
    pub category: Option<String>,
    pub status: Option<String>,
}

/// Allowed categories
const ALLOWED_CATEGORIES: &[&str] = &["nba", "politics"];

/// GET /api/events
pub async fn list_events(
    Extension(pool): Extension<PgPool>,
    Query(filters): Query<EventFilters>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut query = String::from("SELECT * FROM events WHERE 1=1");
    let mut params: Vec<String> = Vec::new();

    if let Some(ref cat) = filters.category {
        // Validate category
        if !ALLOWED_CATEGORIES.contains(&cat.as_str()) {
            return Err(StatusCode::BAD_REQUEST);
        }
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

/// GET /api/events/:id/odds — latest odds (refresh if stale >15 min)
pub async fn get_event_odds(
    Extension(pool): Extension<PgPool>,
    Extension(cfg): Extension<Config>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let event = match event {
        Some(e) => e,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Check if cached odds are stale (>15 minutes old)
    let is_stale = event
        .odds_updated_at
        .map(|t| Utc::now() - t > chrono::Duration::minutes(15))
        .unwrap_or(true);

    let cached_odds = if is_stale && !cfg.odds_api_key.is_empty() {
        // Try to refresh odds based on category
        match event.category.as_str() {
            "nba" => {
                match crate::services::odds::fetch_nba_odds(&cfg.odds_api_key).await {
                    Ok(odds_events) => {
                        let odds_json = serde_json::to_value(&odds_events).unwrap_or_default();
                        // Update cache in DB
                        let _ = sqlx::query(
                            "UPDATE events SET cached_odds = $1, odds_updated_at = NOW() WHERE id = $2"
                        )
                        .bind(&odds_json)
                        .bind(id)
                        .execute(&pool)
                        .await;
                        odds_json
                    }
                    Err(e) => {
                        tracing::warn!("Failed to refresh NBA odds: {e}");
                        event.cached_odds.clone()
                    }
                }
            }
            "politics" => match crate::services::odds::fetch_polymarket_events().await {
                Ok(poly_events) => {
                    let odds_json = serde_json::to_value(&poly_events).unwrap_or_default();
                    let _ = sqlx::query(
                        "UPDATE events SET cached_odds = $1, odds_updated_at = NOW() WHERE id = $2",
                    )
                    .bind(&odds_json)
                    .bind(id)
                    .execute(&pool)
                    .await;
                    odds_json
                }
                Err(e) => {
                    tracing::warn!("Failed to refresh Polymarket odds: {e}");
                    event.cached_odds.clone()
                }
            },
            _ => event.cached_odds.clone(),
        }
    } else {
        event.cached_odds.clone()
    };

    Ok(Json(serde_json::json!({
        "event_id": event.id,
        "category": event.category,
        "cached_odds": cached_odds,
        "odds_updated_at": event.odds_updated_at,
    })))
}
