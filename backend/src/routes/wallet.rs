use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{DailyBonusResponse, Transaction, User, WalletBalance};

const DAILY_BONUS_AMOUNT: i64 = 100;

/// GET /api/wallet/balance
pub async fn get_balance(Extension(user): Extension<User>) -> impl IntoResponse {
    let bonus_available = user
        .last_daily_bonus
        .map(|t| Utc::now() - t > Duration::hours(24))
        .unwrap_or(true);

    Json(WalletBalance {
        coin_balance: user.coin_balance,
        last_daily_bonus: user.last_daily_bonus,
        bonus_available,
    })
}

#[derive(Deserialize, Default)]
pub struct TransactionFilters {
    pub limit: Option<i64>,
}

/// GET /api/wallet/transactions
pub async fn get_transactions(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    axum::extract::Query(filters): axum::extract::Query<TransactionFilters>,
) -> Result<impl IntoResponse, StatusCode> {
    let limit = filters.limit.unwrap_or(50).min(100);

    let txns = sqlx::query_as::<_, Transaction>(
        "SELECT * FROM transactions WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(user.id)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(txns))
}

/// POST /api/wallet/daily-bonus — claim 100 coins per day
pub async fn claim_daily_bonus(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_balance = sqlx::query_scalar::<_, i64>(
        r#"
        UPDATE users
        SET coin_balance = coin_balance + $1,
            last_daily_bonus = NOW(),
            updated_at = NOW()
        WHERE id = $2
          AND (
            last_daily_bonus IS NULL
            OR last_daily_bonus < NOW() - INTERVAL '24 hours'
          )
        RETURNING coin_balance
        "#,
    )
    .bind(DAILY_BONUS_AMOUNT)
    .bind(user.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::TOO_MANY_REQUESTS)?;

    sqlx::query(
        "INSERT INTO transactions (id, user_id, type, amount, balance_after, created_at) VALUES ($1, $2, 'daily_bonus', $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(DAILY_BONUS_AMOUNT)
    .bind(new_balance)
    .execute(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DailyBonusResponse {
        coins_awarded: DAILY_BONUS_AMOUNT,
        new_balance,
    }))
}
