use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{PgConnection, PgPool};
use std::sync::Arc;
use std::time::{Duration as StdDuration, Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::config::Config;
use crate::models::Event;
use crate::services::odds::{self, SgoEvent};

#[derive(Clone, Default)]
pub struct SyncThrottle {
    last_sync_started_at: Arc<Mutex<Option<Instant>>>,
}

impl SyncThrottle {
    pub async fn allow(&self, cooldown: StdDuration) -> bool {
        let mut last_sync_started_at = self.last_sync_started_at.lock().await;
        if let Some(last_sync) = *last_sync_started_at {
            if last_sync.elapsed() < cooldown {
                return false;
            }
        }

        *last_sync_started_at = Some(Instant::now());
        true
    }
}

#[derive(Deserialize)]
pub struct EventFilters {
    pub category: Option<String>,
    pub league: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct SyncQuery {
    pub leagues: Option<String>,
}

const DEFAULT_LEAGUES: &[&str] = &["NBA", "NFL", "MLB", "NHL"];

fn league_to_category(league_id: &str) -> String {
    league_id.to_lowercase()
}

fn sport_id_to_sport(sport_id: &str) -> String {
    sport_id.to_lowercase()
}

fn build_title(sgo: &SgoEvent) -> String {
    if let Some(ref teams) = sgo.teams {
        let home = teams
            .home
            .as_ref()
            .and_then(|team| team.names.as_ref())
            .and_then(|names| {
                names
                    .long
                    .as_deref()
                    .or(names.medium.as_deref())
                    .or(names.short.as_deref())
            })
            .unwrap_or("TBD");
        let away = teams
            .away
            .as_ref()
            .and_then(|team| team.names.as_ref())
            .and_then(|names| {
                names
                    .long
                    .as_deref()
                    .or(names.medium.as_deref())
                    .or(names.short.as_deref())
            })
            .unwrap_or("TBD");

        format!("{away} @ {home}")
    } else {
        sgo.event_id
            .as_deref()
            .unwrap_or("Unknown Event")
            .to_string()
    }
}

fn derive_status(sgo: &SgoEvent) -> &'static str {
    if let Some(ref status) = sgo.status {
        if status.cancelled.unwrap_or(false) {
            return "cancelled";
        }
        if status.finalized.unwrap_or(false) || status.ended.unwrap_or(false) {
            return "ended";
        }
        if status.live.unwrap_or(false) {
            return "live";
        }
        if status.started.unwrap_or(false) {
            return "in_progress";
        }
    }

    "upcoming"
}

async fn acquire_lock(connection: &mut PgConnection, key: &str) -> Result<bool, StatusCode> {
    sqlx::query_scalar::<_, bool>("SELECT pg_try_advisory_lock(hashtext($1))")
        .bind(key)
        .fetch_one(connection)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn release_lock(connection: &mut PgConnection, key: &str) {
    let _ = sqlx::query("SELECT pg_advisory_unlock(hashtext($1))")
        .bind(key)
        .execute(&mut *connection)
        .await;
}

pub async fn sync_events(
    Extension(pool): Extension<PgPool>,
    Extension(cfg): Extension<Config>,
    Extension(sync_throttle): Extension<SyncThrottle>,
    Query(query): Query<SyncQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    if cfg.sportsgameodds_api_key.is_empty() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    if !sync_throttle
        .allow(StdDuration::from_secs(cfg.events_sync_cooldown_secs))
        .await
    {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let league_strs: Vec<&str> = if let Some(ref leagues) = query.leagues {
        leagues.split(',').map(|value| value.trim()).collect()
    } else {
        DEFAULT_LEAGUES.to_vec()
    };

    let sgo_events = odds::fetch_sportsgameodds_events(&cfg.sportsgameodds_api_key, &league_strs)
        .await
        .map_err(|error| {
            tracing::error!("SportsGameOdds sync failed: {error}");
            StatusCode::BAD_GATEWAY
        })?;

    let mut connection = pool
        .acquire()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let lock_key = "sidebet_events_sync";
    if !acquire_lock(&mut connection, lock_key).await? {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let result = sync_events_inner(&mut connection, &sgo_events).await;
    release_lock(&mut connection, lock_key).await;
    result.map(|synced| {
        Json(serde_json::json!({
            "synced": synced,
            "leagues": league_strs,
        }))
    })
}

async fn sync_events_inner(
    connection: &mut PgConnection,
    sgo_events: &[SgoEvent],
) -> Result<u32, StatusCode> {
    let mut synced = 0u32;

    for sgo in sgo_events {
        let sgo_id = match sgo.event_id.as_deref() {
            Some(id) => id,
            None => continue,
        };

        let title = build_title(sgo);
        let category = sgo
            .league_id
            .as_deref()
            .map(league_to_category)
            .unwrap_or_else(|| "other".into());
        let sport = sgo.sport_id.as_deref().map(sport_id_to_sport);
        let league = sgo.league_id.as_deref().map(|value| value.to_string());
        let status = derive_status(sgo);
        let starts_at = sgo
            .status
            .as_ref()
            .and_then(|value| value.starts_at.as_deref())
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc));
        let external_ids = serde_json::json!({ "sportsgameodds": sgo_id });
        let cached_odds = serde_json::to_value(&sgo.odds).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO events (id, category, title, sport, league, starts_at, external_ids, cached_odds, odds_updated_at, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9)
            ON CONFLICT ((external_ids->>'sportsgameodds'))
            WHERE external_ids ? 'sportsgameodds'
            DO UPDATE SET
                title = EXCLUDED.title,
                category = EXCLUDED.category,
                sport = EXCLUDED.sport,
                league = EXCLUDED.league,
                starts_at = EXCLUDED.starts_at,
                cached_odds = EXCLUDED.cached_odds,
                odds_updated_at = NOW(),
                status = EXCLUDED.status
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&category)
        .bind(&title)
        .bind(sport.as_deref())
        .bind(league.as_deref())
        .bind(starts_at)
        .bind(&external_ids)
        .bind(&cached_odds)
        .bind(status)
        .execute(&mut *connection)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        synced += 1;
    }

    Ok(synced)
}

pub async fn list_events(
    Extension(pool): Extension<PgPool>,
    Query(filters): Query<EventFilters>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut query = String::from("SELECT * FROM events WHERE 1=1");
    let mut params: Vec<String> = Vec::new();

    if let Some(ref category) = filters.category {
        params.push(category.to_lowercase());
        query.push_str(&format!(" AND category = ${}", params.len()));
    }
    if let Some(ref league) = filters.league {
        params.push(league.to_uppercase());
        query.push_str(&format!(" AND UPPER(league) = ${}", params.len()));
    }
    if let Some(ref status) = filters.status {
        params.push(status.clone());
        query.push_str(&format!(" AND status = ${}", params.len()));
    } else {
        query.push_str(" AND status NOT IN ('ended', 'cancelled')");
    }
    query.push_str(" ORDER BY starts_at ASC NULLS LAST LIMIT 50");

    let mut built = sqlx::query_as::<_, Event>(&query);
    for param in &params {
        built = built.bind(param);
    }

    let events = built
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(events))
}

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
        Some(event) => Ok(Json(event)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn get_event_odds(
    Extension(pool): Extension<PgPool>,
    Extension(cfg): Extension<Config>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let is_stale = event
        .odds_updated_at
        .map(|value| Utc::now() - value > chrono::Duration::minutes(15))
        .unwrap_or(true);

    let cached_odds = if is_stale && !cfg.sportsgameodds_api_key.is_empty() {
        refresh_event_odds(&pool, &cfg, &event).await?
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

async fn refresh_event_odds(
    pool: &PgPool,
    cfg: &Config,
    event: &Event,
) -> Result<serde_json::Value, StatusCode> {
    let mut connection = pool
        .acquire()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let lock_key = format!("sidebet_event_odds:{}", event.id);
    if !acquire_lock(&mut connection, &lock_key).await? {
        return Ok(event.cached_odds.clone());
    }

    let result = refresh_event_odds_inner(&mut connection, cfg, event).await;
    release_lock(&mut connection, &lock_key).await;
    result
}

async fn refresh_event_odds_inner(
    connection: &mut PgConnection,
    cfg: &Config,
    event: &Event,
) -> Result<serde_json::Value, StatusCode> {
    let sgo_id = event
        .external_ids
        .get("sportsgameodds")
        .and_then(|value| value.as_str());

    if let Some(sgo_id) = sgo_id {
        match odds::fetch_sportsgameodds_event(&cfg.sportsgameodds_api_key, sgo_id).await {
            Ok(Some(sgo_event)) => {
                let odds_json = serde_json::to_value(&sgo_event.odds).unwrap_or_default();
                let status = derive_status(&sgo_event);
                sqlx::query(
                    "UPDATE events SET cached_odds = $1, odds_updated_at = NOW(), status = $2 WHERE id = $3",
                )
                .bind(&odds_json)
                .bind(status)
                .bind(event.id)
                .execute(connection)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                return Ok(odds_json);
            }
            Ok(None) => return Ok(event.cached_odds.clone()),
            Err(error) => {
                tracing::warn!("Failed to refresh odds for event {}: {error}", event.id);
                return Ok(event.cached_odds.clone());
            }
        }
    }

    if event.category == "politics" {
        match odds::fetch_polymarket_events().await {
            Ok(poly_events) => {
                let odds_json = serde_json::to_value(&poly_events).unwrap_or_default();
                sqlx::query("UPDATE events SET cached_odds = $1, odds_updated_at = NOW() WHERE id = $2")
                    .bind(&odds_json)
                    .bind(event.id)
                    .execute(connection)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                return Ok(odds_json);
            }
            Err(error) => {
                tracing::warn!("Failed to refresh Polymarket odds: {error}");
            }
        }
    }

    Ok(event.cached_odds.clone())
}
