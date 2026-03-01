use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ── Users ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub wallet_address: Option<String>,
    pub wins: i32,
    pub losses: i32,
    pub total_wagered_wei: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfile {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub wins: i32,
    pub losses: i32,
}

impl From<User> for PublicUser {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
            bio: u.bio,
            wins: u.wins,
            losses: u.losses,
        }
    }
}

// ── Friendships ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Friendship {
    pub id: Uuid,
    pub requester_id: Uuid,
    pub addressee_id: Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct FriendRequest {
    pub user_id: Uuid,
}

// ── Events ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: Uuid,
    pub category: String,
    pub title: String,
    pub description: Option<String>,
    pub sport: Option<String>,
    pub league: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub external_ids: serde_json::Value,
    pub cached_odds: serde_json::Value,
    pub odds_updated_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// ── Bets ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bet {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub opponent_id: Uuid,
    pub event_id: Option<Uuid>,
    pub question: String,
    pub creator_position: String,
    pub opponent_position: String,
    pub amount_wei: i64,
    pub odds_numerator: i32,
    pub odds_denominator: i32,
    pub reference_odds: Option<serde_json::Value>,
    pub status: String,
    pub on_chain_bet_id: Option<i64>,
    pub contract_address: Option<String>,
    pub assertion_id: Option<String>,
    pub outcome: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBet {
    pub opponent_id: Uuid,
    pub event_id: Option<Uuid>,
    pub question: String,
    pub creator_position: String,
    pub opponent_position: String,
    pub amount_wei: i64,
    pub odds_numerator: i32,
    pub odds_denominator: i32,
    pub reference_odds: Option<serde_json::Value>,
    pub expires_in_hours: Option<i64>,
}

// ── Notifications ──

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub notification_type: String,
    pub payload: serde_json::Value,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

// ── Feed ──

#[derive(Debug, Serialize)]
pub struct FeedItem {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub item_type: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ── Auth (Better Auth session from DB) ──

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Session {
    pub id: String,
    pub token: String,
    #[sqlx(rename = "userId")]
    pub user_id: String,
    #[sqlx(rename = "expiresAt")]
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct AuthUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub image: Option<String>,
}
