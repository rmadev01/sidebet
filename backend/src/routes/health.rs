use axum::{extract::Extension, response::IntoResponse, Json};
use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;

use crate::config::Config;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
    chain: ChainHealth,
    timestamp: chrono::DateTime<Utc>,
}

#[derive(Serialize)]
struct ChainHealth {
    configured: bool,
    rpc_url_set: bool,
    contract_address_set: bool,
}

/// GET /health
pub async fn health(
    Extension(pool): Extension<PgPool>,
    Extension(cfg): Extension<Config>,
) -> impl IntoResponse {
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
        .is_ok();

    let chain = ChainHealth {
        configured: !cfg.rpc_url.is_empty() && !cfg.contract_address.is_empty(),
        rpc_url_set: !cfg.rpc_url.is_empty(),
        contract_address_set: !cfg.contract_address.is_empty(),
    };

    let status = if db_ok { "ok" } else { "degraded" };
    let database = if db_ok { "ok" } else { "error" };

    Json(HealthResponse {
        status,
        database,
        chain,
        timestamp: Utc::now(),
    })
}
