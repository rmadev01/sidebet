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

const ALLOWED_CATEGORIES: &[&str] = &["nba", "nhl", "nfl", "mlb", "politics"];

pub async fn create_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(body): Json<CreateBet>,
) -> Result<impl IntoResponse, StatusCode> {
    validate_create_bet(&user, &body)?;

    let event_id = body.event_id.ok_or(StatusCode::BAD_REQUEST)?;
    let event = sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = $1")
        .bind(event_id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    if !ALLOWED_CATEGORIES.contains(&event.category.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !matches!(event.status.as_str(), "upcoming" | "live") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let status = if body.opponent_id.is_some() { "proposed" } else { "open" };
    let expires_at = Utc::now() + chrono::Duration::hours(body.expires_in_hours.unwrap_or(24));
    let question_for_notify = body.question.clone();
    let amount_for_notify = body.amount;

    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let new_balance = debit_user_balance(&mut tx, user.id, body.amount).await?;
    record_transaction(&mut tx, user.id, "bet_placed", -body.amount, new_balance, None).await?;

    let bet = sqlx::query_as::<_, Bet>(
        r#"
        INSERT INTO bets (
            id, creator_id, opponent_id, event_id, question,
            creator_position, opponent_position, amount,
            odds_numerator, odds_denominator, reference_odds,
            status, expires_at, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
            $12, $13, NOW(), NOW()
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
    .bind(status)
    .bind(expires_at)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(opponent_id) = body.opponent_id {
        notifications::create_notification(
            &pool,
            opponent_id,
            "bet_received",
            serde_json::json!({
                "bet_id": bet.id,
                "from_username": user.username,
                "question": question_for_notify,
                "amount": amount_for_notify,
            }),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok((StatusCode::CREATED, Json(bet)))
}

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

    let mut built = sqlx::query_as::<_, Bet>(&query).bind(user.id);
    for param in &string_params {
        built = built.bind(param);
    }

    let bets = built
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(bets))
}

pub async fn get_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let bet = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match bet {
        Some(bet) if bet.status == "open" || bet.creator_id == user.id || bet.opponent_id == Some(user.id) => {
            Ok(Json(bet))
        }
        Some(_) => Err(StatusCode::FORBIDDEN),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn accept_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let existing = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let existing = match existing {
        Some(bet) if bet.opponent_id == Some(user.id) && bet.status == "proposed" => bet,
        _ => return Err(StatusCode::NOT_FOUND),
    };

    let new_balance = debit_user_balance(&mut tx, user.id, existing.amount).await?;
    record_transaction(&mut tx, user.id, "bet_placed", -existing.amount, new_balance, Some(existing.id)).await?;

    let bet = sqlx::query_as::<_, Bet>(
        r#"
        UPDATE bets SET status = 'active', updated_at = NOW()
        WHERE id = $1 AND opponent_id = $2 AND status = 'proposed'
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::CONFLICT)?;

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    notifications::create_notification(
        &pool,
        bet.creator_id,
        "bet_accepted",
        serde_json::json!({
            "bet_id": bet.id,
            "from_username": user.username,
        }),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(bet))
}

pub async fn decline_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let bet = sqlx::query_as::<_, Bet>(
        r#"
        UPDATE bets SET status = 'declined', updated_at = NOW()
        WHERE id = $1 AND opponent_id = $2 AND status = 'proposed'
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let new_balance = credit_user_balance(&mut tx, bet.creator_id, bet.amount).await?;
    record_transaction(&mut tx, bet.creator_id, "bet_refund", bet.amount, new_balance, Some(bet.id)).await?;

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn cancel_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let bet = sqlx::query_as::<_, Bet>(
        r#"
        UPDATE bets SET status = 'cancelled', updated_at = NOW()
        WHERE id = $1 AND creator_id = $2 AND status IN ('proposed', 'open')
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let new_balance = credit_user_balance(&mut tx, user.id, bet.amount).await?;
    record_transaction(&mut tx, user.id, "bet_refund", bet.amount, new_balance, Some(id)).await?;

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn settle_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
    Json(body): Json<SettleBetRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let bet = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let bet = match bet {
        Some(bet) if bet.status == "active" => bet,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let opponent_id = bet.opponent_id.ok_or(StatusCode::BAD_REQUEST)?;
    if bet.creator_id != user.id && opponent_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    if body.outcome != "disputed" {
        return Err(StatusCode::BAD_REQUEST);
    }

    let disputed_bet = sqlx::query_as::<_, Bet>(
        "UPDATE bets SET status = 'disputed', outcome = 'disputed', resolved_at = NOW(), updated_at = NOW() WHERE id = $1 AND status = 'active' RETURNING *",
    )
    .bind(bet.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::CONFLICT)?;

    let creator_balance = credit_user_balance(&mut tx, bet.creator_id, bet.amount).await?;
    record_transaction(&mut tx, bet.creator_id, "bet_refund", bet.amount, creator_balance, Some(bet.id)).await?;

    let opponent_balance = credit_user_balance(&mut tx, opponent_id, bet.amount).await?;
    record_transaction(&mut tx, opponent_id, "bet_refund", bet.amount, opponent_balance, Some(bet.id)).await?;

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    notifications::create_notification(
        &pool,
        bet.creator_id,
        "bet_refund",
        serde_json::json!({
            "bet_id": bet.id,
            "question": bet.question,
            "reason": "disputed",
        }),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    notifications::create_notification(
        &pool,
        opponent_id,
        "bet_refund",
        serde_json::json!({
            "bet_id": bet.id,
            "question": bet.question,
            "reason": "disputed",
        }),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(disputed_bet))
}

pub async fn take_open_bet(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let bet = sqlx::query_as::<_, Bet>("SELECT * FROM bets WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let bet = match bet {
        Some(bet)
            if bet.status == "open" && bet.opponent_id.is_none() && bet.expires_at > Utc::now() =>
        {
            bet
        }
        _ => return Err(StatusCode::NOT_FOUND),
    };

    if bet.creator_id == user.id {
        return Err(StatusCode::BAD_REQUEST);
    }

    let new_balance = debit_user_balance(&mut tx, user.id, bet.amount).await?;
    record_transaction(&mut tx, user.id, "bet_placed", -bet.amount, new_balance, Some(bet.id)).await?;

    let updated = sqlx::query_as::<_, Bet>(
        "UPDATE bets SET opponent_id = $1, status = 'active', updated_at = NOW() WHERE id = $2 AND status = 'open' AND expires_at > NOW() RETURNING *",
    )
    .bind(user.id)
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::CONFLICT)?;

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    notifications::create_notification(
        &pool,
        bet.creator_id,
        "bet_accepted",
        serde_json::json!({
            "bet_id": bet.id,
            "from_username": user.username,
            "question": bet.question,
        }),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated))
}

fn validate_create_bet(user: &User, body: &CreateBet) -> Result<(), StatusCode> {
    if body.opponent_id == Some(user.id)
        || body.amount <= 0
        || body.odds_numerator <= 0
        || body.odds_denominator <= 0
        || body.creator_position.trim().is_empty()
        || body.opponent_position.trim().is_empty()
        || body.creator_position == body.opponent_position
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(())
}

async fn debit_user_balance(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    amount: i64,
) -> Result<i64, StatusCode> {
    sqlx::query_scalar::<_, i64>(
        "UPDATE users SET coin_balance = coin_balance - $1, total_wagered = total_wagered + $1, updated_at = NOW() WHERE id = $2 AND coin_balance >= $1 RETURNING coin_balance",
    )
    .bind(amount)
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::PAYMENT_REQUIRED)
}

async fn credit_user_balance(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    amount: i64,
) -> Result<i64, StatusCode> {
    sqlx::query_scalar::<_, i64>(
        "UPDATE users SET coin_balance = coin_balance + $1, updated_at = NOW() WHERE id = $2 RETURNING coin_balance",
    )
    .bind(amount)
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn record_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    tx_type: &str,
    amount: i64,
    balance_after: i64,
    reference_id: Option<Uuid>,
) -> Result<(), StatusCode> {
    sqlx::query(
        "INSERT INTO transactions (id, user_id, type, amount, balance_after, reference_id, created_at) VALUES ($1, $2, $3, $4, $5, $6, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(tx_type)
    .bind(amount)
    .bind(balance_after)
    .bind(reference_id)
    .execute(&mut **tx)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}
