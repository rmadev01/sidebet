use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Bet, CreateBet, User};

#[derive(Deserialize)]
pub struct BetFilters {
    pub status: Option<String>,
    pub role: Option<String>,
}

/// POST /api/bets — create a bet and send to opponent
pub async fn create_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(body): Json<CreateBet>,
) -> Result<impl IntoResponse, StatusCode> {
    if body.opponent_id == user.id {
        return Err(StatusCode::BAD_REQUEST);
    }

    let expires_hours = body.expires_in_hours.unwrap_or(24);
    let expires_at = Utc::now() + chrono::Duration::hours(expires_hours);

    let bet = sqlx::query_as::<_, Bet>(
        r#"
        INSERT INTO bets (
            id, creator_id, opponent_id, event_id, question,
            creator_position, opponent_position, amount_wei,
            odds_numerator, odds_denominator, reference_odds,
            status, expires_at, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
            'proposed', $12, NOW(), NOW()
        )
        RETURNING *
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(body.opponent_id)
    .bind(body.event_id)
    .bind(&body.question)
    .bind(&body.creator_position)
    .bind(&body.opponent_position)
    .bind(body.amount_wei)
    .bind(body.odds_numerator)
    .bind(body.odds_denominator)
    .bind(&body.reference_odds)
    .bind(expires_at)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create notification for opponent
    let _ = sqlx::query(
        r#"
        INSERT INTO notifications (id, user_id, type, payload, read, created_at)
        VALUES ($1, $2, 'bet_received', $3, false, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(body.opponent_id)
    .bind(serde_json::json!({
        "bet_id": bet.id,
        "from_username": user.username,
        "question": body.question,
        "amount_wei": body.amount_wei,
    }))
    .execute(&pool)
    .await;

    Ok((StatusCode::CREATED, Json(bet)))
}

/// GET /api/bets
pub async fn list_bets(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Query(filters): Query<BetFilters>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut query = String::from("SELECT * FROM bets WHERE (creator_id = $1 OR opponent_id = $1)");
    let mut bind_idx = 2;
    let mut string_params: Vec<String> = Vec::new();

    if let Some(ref status) = filters.status {
        string_params.push(status.clone());
        query.push_str(&format!(" AND status = ${bind_idx}"));
        bind_idx += 1;
    }

    if let Some(ref role) = filters.role {
        match role.as_str() {
            "creator" => query.push_str(&format!(" AND creator_id = $1")),
            "opponent" => query.push_str(&format!(" AND opponent_id = $1")),
            _ => {}
        }
    }

    query.push_str(" ORDER BY created_at DESC LIMIT 50");

    let mut q = sqlx::query_as::<_, Bet>(&query).bind(user.id);
    for p in &string_params {
        q = q.bind(p);
    }

    let bets = q
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(bets))
}

/// GET /api/bets/:id
pub async fn get_bet(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let bet = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match bet {
        Some(b) => Ok(Json(b)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// POST /api/bets/:id/accept
pub async fn accept_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query(
        r#"
        UPDATE bets SET status = 'accepted', updated_at = NOW()
        WHERE id = $1 AND opponent_id = $2 AND status = 'proposed'
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

    // Notify the creator
    let bet = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _ = sqlx::query(
        r#"
        INSERT INTO notifications (id, user_id, type, payload, read, created_at)
        VALUES ($1, $2, 'bet_accepted', $3, false, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(bet.creator_id)
    .bind(serde_json::json!({
        "bet_id": bet.id,
        "from_username": user.username,
    }))
    .execute(&pool)
    .await;

    Ok(Json(bet))
}

/// POST /api/bets/:id/decline
pub async fn decline_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query(
        r#"
        UPDATE bets SET status = 'declined', updated_at = NOW()
        WHERE id = $1 AND opponent_id = $2 AND status = 'proposed'
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

/// POST /api/bets/:id/cancel — creator only, before accepted
pub async fn cancel_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query(
        r#"
        UPDATE bets SET status = 'cancelled', updated_at = NOW()
        WHERE id = $1 AND creator_id = $2 AND status = 'proposed'
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

/// POST /api/bets/:id/settle — trigger settlement
pub async fn settle_bet(
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let bet = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let bet = match bet {
        Some(b) if b.status == "active" => b,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    // Mark as settling (contract interaction would happen here)
    let _ = sqlx::query(
        "UPDATE bets SET status = 'settling', updated_at = NOW() WHERE id = $1",
    )
    .bind(bet.id)
    .execute(&pool)
    .await;

    Ok(Json(serde_json::json!({
        "bet_id": bet.id,
        "status": "settling",
        "message": "Settlement initiated. UMA assertion will be created on-chain.",
    })))
}
