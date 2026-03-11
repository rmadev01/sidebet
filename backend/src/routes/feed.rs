use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use sqlx::PgPool;

use crate::models::{FeedItem, User};

/// GET /api/feed — activity feed (friends' settled bets, accepted requests, etc.)
pub async fn get_feed(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    // Get friend IDs
    let friend_ids: Vec<uuid::Uuid> = sqlx::query_scalar(
        r#"
        SELECT CASE
            WHEN requester_id = $1 THEN addressee_id
            ELSE requester_id
        END
        FROM friendships
        WHERE (requester_id = $1 OR addressee_id = $1)
          AND status = 'accepted'
        "#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if friend_ids.is_empty() {
        return Ok(Json(Vec::<FeedItem>::new()));
    }

    // Get recent settled bets involving friends
    let bets = sqlx::query_as::<_, crate::models::Bet>(
        r#"
        SELECT * FROM bets
        WHERE (creator_id = ANY($1) OR opponent_id = ANY($1))
          AND status IN ('settled', 'active')
        ORDER BY updated_at DESC
        LIMIT 30
        "#,
    )
    .bind(&friend_ids)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items: Vec<FeedItem> = bets
        .into_iter()
        .map(|b| FeedItem {
            id: b.id,
            item_type: format!("bet_{}", b.status),
            payload: serde_json::json!({
                "bet_id": b.id,
                "question": b.question,
                "creator_id": b.creator_id,
                "opponent_id": b.opponent_id,
                "amount": b.amount,
                "status": b.status,
                "outcome": b.outcome,
            }),
            created_at: b.updated_at,
        })
        .collect();

    Ok(Json(items))
}

/// GET /api/feed/open — all open bets available for anyone to take
pub async fn get_open_bets(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
) -> Result<impl IntoResponse, StatusCode> {
    let rows = sqlx::query_as::<_, OpenBetRow>(
        r#"
        SELECT b.*, u.username AS creator_username, u.display_name AS creator_display_name
        FROM bets b
        JOIN users u ON u.id = b.creator_id
        WHERE b.status = 'open'
          AND b.expires_at > NOW()
          AND b.creator_id != $1
        ORDER BY b.created_at DESC
        LIMIT 50
        "#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows))
}

/// Extended bet row with creator info for the open feed
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct OpenBetRow {
    pub id: uuid::Uuid,
    pub creator_id: uuid::Uuid,
    pub opponent_id: Option<uuid::Uuid>,
    pub event_id: Option<uuid::Uuid>,
    pub question: String,
    pub creator_position: String,
    pub opponent_position: String,
    pub amount: i64,
    pub odds_numerator: i32,
    pub odds_denominator: i32,
    pub reference_odds: Option<serde_json::Value>,
    pub status: String,
    pub winner_id: Option<uuid::Uuid>,
    pub outcome: Option<String>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    // Extra fields from JOIN
    pub creator_username: String,
    pub creator_display_name: String,
}
