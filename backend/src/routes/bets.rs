use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Bet, CreateBet, Event, SettleBetRequest, User};
use crate::services::notifications;

#[derive(Deserialize)]
pub struct BetFilters {
    pub status: Option<String>,
    pub role: Option<String>,
}

/// Allowed event categories for betting
const ALLOWED_CATEGORIES: &[&str] = &["nba", "politics"];

/// POST /api/bets — create a bet tied to a known event
pub async fn create_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(body): Json<CreateBet>,
) -> Result<impl IntoResponse, StatusCode> {
    if body.opponent_id == user.id {
        return Err(StatusCode::BAD_REQUEST);
    }

    if body.amount <= 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check creator has enough coins
    if user.coin_balance < body.amount {
        return Err(StatusCode::PAYMENT_REQUIRED);
    }

    // Require an event — no custom/arbitrary bets
    let event_id = body.event_id.ok_or(StatusCode::BAD_REQUEST)?;

    // Validate event exists and is in an allowed category
    let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    if !ALLOWED_CATEGORIES.contains(&event.category.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    if event.status != "upcoming" && event.status != "live" {
        return Err(StatusCode::BAD_REQUEST);
    }

    let expires_hours = body.expires_in_hours.unwrap_or(24);
    let expires_at = Utc::now() + chrono::Duration::hours(expires_hours);

    // Deduct coins from creator
    let new_balance = sqlx::query_scalar::<_, i64>(
        "UPDATE users SET coin_balance = coin_balance - $1, total_wagered = total_wagered + $1, updated_at = NOW() WHERE id = $2 AND coin_balance >= $1 RETURNING coin_balance",
    )
    .bind(body.amount)
    .bind(user.id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::PAYMENT_REQUIRED)?;

    // Record transaction
    let _ = sqlx::query(
        "INSERT INTO transactions (id, user_id, type, amount, balance_after, created_at) VALUES ($1, $2, 'bet_placed', $3, $4, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(-body.amount)
    .bind(new_balance)
    .execute(&pool)
    .await;

    let question_for_notify = body.question.clone();
    let amount_for_notify = body.amount;
    let bet = sqlx::query_as::<_, Bet>(
        r#"
        INSERT INTO bets (
            id, creator_id, opponent_id, event_id, question,
            creator_position, opponent_position, amount,
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
    .bind(event_id)
    .bind(&body.question)
    .bind(&body.creator_position)
    .bind(&body.opponent_position)
    .bind(body.amount)
    .bind(body.odds_numerator)
    .bind(body.odds_denominator)
    .bind(&body.reference_odds)
    .bind(expires_at)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Notify opponent
    let _ = notifications::create_notification(
        &pool,
        body.opponent_id,
        "bet_received",
        serde_json::json!({
            "bet_id": bet.id,
            "from_username": user.username,
            "question": question_for_notify,
            "amount": amount_for_notify,
        }),
    )
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
    let mut string_params: Vec<String> = Vec::new();

    if let Some(ref status) = filters.status {
        string_params.push(status.clone());
        query.push_str(&format!(" AND status = ${}", string_params.len() + 1));
    }

    if let Some(ref role) = filters.role {
        match role.as_str() {
            "creator" => query.push_str(" AND creator_id = $1"),
            "opponent" => query.push_str(" AND opponent_id = $1"),
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

/// POST /api/bets/:id/accept — opponent accepts, status → 'active'
pub async fn accept_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let existing = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let existing = match existing {
        Some(b) if b.opponent_id == user.id && b.status == "proposed" => b,
        _ => return Err(StatusCode::NOT_FOUND),
    };

    // Check opponent has enough coins
    if user.coin_balance < existing.amount {
        return Err(StatusCode::PAYMENT_REQUIRED);
    }

    // Deduct coins from opponent
    let new_balance = sqlx::query_scalar::<_, i64>(
        "UPDATE users SET coin_balance = coin_balance - $1, total_wagered = total_wagered + $1, updated_at = NOW() WHERE id = $2 AND coin_balance >= $1 RETURNING coin_balance",
    )
    .bind(existing.amount)
    .bind(user.id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::PAYMENT_REQUIRED)?;

    // Record transaction
    let _ = sqlx::query(
        "INSERT INTO transactions (id, user_id, type, amount, balance_after, reference_id, created_at) VALUES ($1, $2, 'bet_placed', $3, $4, $5, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(-existing.amount)
    .bind(new_balance)
    .bind(existing.id)
    .execute(&pool)
    .await;

    let result = sqlx::query(
        r#"
        UPDATE bets SET status = 'active', updated_at = NOW()
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

    let bet = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Notify creator
    let _ = notifications::create_notification(
        &pool,
        bet.creator_id,
        "bet_accepted",
        serde_json::json!({
            "bet_id": bet.id,
            "from_username": user.username,
        }),
    )
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

    // Refund creator's coins
    let bet = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_balance = sqlx::query_scalar::<_, i64>(
        "UPDATE users SET coin_balance = coin_balance + $1, updated_at = NOW() WHERE id = $2 RETURNING coin_balance",
    )
    .bind(bet.amount)
    .bind(bet.creator_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _ = sqlx::query(
        "INSERT INTO transactions (id, user_id, type, amount, balance_after, reference_id, created_at) VALUES ($1, $2, 'bet_refund', $3, $4, $5, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(bet.creator_id)
    .bind(bet.amount)
    .bind(new_balance)
    .bind(bet.id)
    .execute(&pool)
    .await;

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

    // Refund creator's coins
    let new_balance = sqlx::query_scalar::<_, i64>(
        "UPDATE users SET coin_balance = coin_balance + $1, updated_at = NOW() WHERE id = $2 RETURNING coin_balance",
    )
    .bind(
        sqlx::query_scalar::<_, i64>("SELECT amount FROM bets WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    )
    .bind(user.id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _ = sqlx::query(
        "INSERT INTO transactions (id, user_id, type, amount, balance_after, reference_id, created_at) VALUES ($1, $2, 'bet_refund', $3, $4, $5, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(
        sqlx::query_scalar::<_, i64>("SELECT amount FROM bets WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    )
    .bind(new_balance)
    .bind(id)
    .execute(&pool)
    .await;

    Ok(StatusCode::OK)
}

/// POST /api/bets/:id/settle — server-side settlement
pub async fn settle_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<SettleBetRequest>,
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

    // Only creator or opponent can trigger settlement
    if bet.creator_id != user.id && bet.opponent_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let winner_id = match body.winner.as_str() {
        "creator" => bet.creator_id,
        "opponent" => bet.opponent_id,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let loser_id = if winner_id == bet.creator_id {
        bet.opponent_id
    } else {
        bet.creator_id
    };

    // Total pot = creator stake + opponent stake (both have already been deducted)
    let total_pot = bet.amount * 2;

    // Award winnings to winner
    let winner_new_balance = sqlx::query_scalar::<_, i64>(
        "UPDATE users SET coin_balance = coin_balance + $1, wins = wins + 1, updated_at = NOW() WHERE id = $2 RETURNING coin_balance",
    )
    .bind(total_pot)
    .bind(winner_id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Record winner transaction
    let _ = sqlx::query(
        "INSERT INTO transactions (id, user_id, type, amount, balance_after, reference_id, created_at) VALUES ($1, $2, 'bet_won', $3, $4, $5, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(winner_id)
    .bind(total_pot)
    .bind(winner_new_balance)
    .bind(bet.id)
    .execute(&pool)
    .await;

    // Update loser stats
    let _ = sqlx::query("UPDATE users SET losses = losses + 1, updated_at = NOW() WHERE id = $1")
        .bind(loser_id)
        .execute(&pool)
        .await;

    // Update bet
    let outcome = if winner_id == bet.creator_id {
        "creator_wins"
    } else {
        "opponent_wins"
    };

    let settled_bet = sqlx::query_as::<_, Bet>(
        "UPDATE bets SET status = 'settled', winner_id = $1, outcome = $2, resolved_at = NOW(), updated_at = NOW() WHERE id = $3 RETURNING *",
    )
    .bind(winner_id)
    .bind(outcome)
    .bind(bet.id)
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Notify both parties
    let _ = notifications::create_notification(
        &pool,
        winner_id,
        "bet_won",
        serde_json::json!({
            "bet_id": bet.id,
            "question": bet.question,
            "amount_won": total_pot,
        }),
    )
    .await;

    let _ = notifications::create_notification(
        &pool,
        loser_id,
        "bet_lost",
        serde_json::json!({
            "bet_id": bet.id,
            "question": bet.question,
            "amount_lost": bet.amount,
        }),
    )
    .await;

    Ok(Json(settled_bet))
}
