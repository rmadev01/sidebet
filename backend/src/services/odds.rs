use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fetch events from SportsGameOdds API v2
pub async fn fetch_sportsgameodds_events(
    api_key: &str,
    league_ids: &[&str],
) -> Result<Vec<SgoEvent>, OddsError> {
    let client = Client::new();
    let league_id_param = league_ids.join(",");
    let resp = client
        .get("https://api.sportsgameodds.com/v2/events/")
        .header("X-Api-Key", api_key)
        .query(&[
            ("leagueID", league_id_param.as_str()),
            ("oddsAvailable", "true"),
            ("started", "false"),
            ("limit", "50"),
        ])
        .send()
        .await
        .map_err(|e| OddsError::Fetch(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(OddsError::Fetch(format!("Status: {}", resp.status())));
    }

    let body: SgoResponse = resp
        .json()
        .await
        .map_err(|e| OddsError::Parse(e.to_string()))?;

    Ok(body.data.unwrap_or_default())
}

/// Fetch a single event by ID from SportsGameOdds API v2
pub async fn fetch_sportsgameodds_event(
    api_key: &str,
    event_id: &str,
) -> Result<Option<SgoEvent>, OddsError> {
    let client = Client::new();
    let resp = client
        .get("https://api.sportsgameodds.com/v2/events/")
        .header("X-Api-Key", api_key)
        .query(&[("eventIDs", event_id)])
        .send()
        .await
        .map_err(|e| OddsError::Fetch(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(OddsError::Fetch(format!("Status: {}", resp.status())));
    }

    let body: SgoResponse = resp
        .json()
        .await
        .map_err(|e| OddsError::Parse(e.to_string()))?;

    Ok(body.data.and_then(|mut v| {
        if v.is_empty() {
            None
        } else {
            Some(v.remove(0))
        }
    }))
}

/// Fetch political prediction markets from Polymarket (Gamma API)
pub async fn fetch_polymarket_events() -> Result<Vec<PolymarketEvent>, OddsError> {
    let client = Client::new();
    let resp = client
        .get("https://gamma-api.polymarket.com/events")
        .query(&[("tag", "politics"), ("active", "true")])
        .send()
        .await
        .map_err(|e| OddsError::Fetch(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(OddsError::Fetch(format!("Status: {}", resp.status())));
    }

    let events: Vec<PolymarketEvent> = resp
        .json()
        .await
        .map_err(|e| OddsError::Parse(e.to_string()))?;

    Ok(events)
}

/// Fetch token price from Polymarket CLOB
pub async fn fetch_polymarket_price(token_id: &str) -> Result<f64, OddsError> {
    let client = Client::new();
    let url = format!("https://clob.polymarket.com/price?token_id={token_id}");
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| OddsError::Fetch(e.to_string()))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| OddsError::Parse(e.to_string()))?;

    let price = body["price"]
        .as_f64()
        .unwrap_or(body.as_f64().unwrap_or(0.5));

    Ok(price)
}

/// Convert Polymarket price (0-1 probability) to decimal odds
pub fn probability_to_decimal_odds(prob: f64) -> f64 {
    if prob <= 0.0 {
        return 100.0;
    }
    1.0 / prob
}

/// Convert decimal odds to implied probability
pub fn decimal_odds_to_probability(odds: f64) -> f64 {
    if odds <= 0.0 {
        return 0.0;
    }
    1.0 / odds
}

// ── Types ──

#[derive(Debug, Deserialize)]
struct SgoResponse {
    success: Option<bool>,
    data: Option<Vec<SgoEvent>>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SgoEvent {
    #[serde(rename = "eventID")]
    pub event_id: Option<String>,
    #[serde(rename = "sportID")]
    pub sport_id: Option<String>,
    #[serde(rename = "leagueID")]
    pub league_id: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub status: Option<SgoStatus>,
    pub teams: Option<SgoTeams>,
    #[serde(default)]
    pub odds: Option<HashMap<String, SgoOdd>>,
    pub results: Option<serde_json::Value>,
    #[serde(default)]
    pub players: Option<HashMap<String, SgoPlayer>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SgoStatus {
    pub starts_at: Option<String>,
    pub started: Option<bool>,
    pub ended: Option<bool>,
    pub finalized: Option<bool>,
    pub cancelled: Option<bool>,
    pub live: Option<bool>,
    pub odds_available: Option<bool>,
    pub display_long: Option<String>,
    pub display_short: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SgoTeams {
    pub home: Option<SgoTeam>,
    pub away: Option<SgoTeam>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SgoTeam {
    #[serde(rename = "teamID")]
    pub team_id: Option<String>,
    pub names: Option<SgoTeamNames>,
    pub score: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SgoTeamNames {
    pub long: Option<String>,
    pub medium: Option<String>,
    pub short: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SgoOdd {
    #[serde(rename = "oddID")]
    pub odd_id: Option<String>,
    #[serde(rename = "opposingOddID")]
    pub opposing_odd_id: Option<String>,
    pub market_name: Option<String>,
    #[serde(rename = "statID")]
    pub stat_id: Option<String>,
    #[serde(rename = "betTypeID")]
    pub bet_type_id: Option<String>,
    #[serde(rename = "sideID")]
    pub side_id: Option<String>,
    #[serde(rename = "periodID")]
    pub period_id: Option<String>,
    pub book_odds: Option<String>,
    pub fair_odds: Option<String>,
    pub book_spread: Option<String>,
    pub book_over_under: Option<String>,
    pub started: Option<bool>,
    pub ended: Option<bool>,
    pub cancelled: Option<bool>,
    pub scoring_supported: Option<bool>,
    #[serde(default)]
    pub by_bookmaker: Option<HashMap<String, SgoBookmakerOdd>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SgoBookmakerOdd {
    pub odds: Option<String>,
    pub spread: Option<String>,
    pub over_under: Option<String>,
    pub available: Option<bool>,
    pub last_updated_at: Option<String>,
    pub deeplink: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SgoPlayer {
    #[serde(rename = "playerID")]
    pub player_id: Option<String>,
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[serde(rename = "teamID")]
    pub team_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolymarketEvent {
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub slug: Option<String>,
    pub markets: Option<Vec<PolymarketMarket>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolymarketMarket {
    pub id: Option<String>,
    pub question: Option<String>,
    #[serde(rename = "outcomePrices")]
    pub outcome_prices: Option<String>,
    #[serde(rename = "clobTokenIds")]
    pub clob_token_ids: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum OddsError {
    #[error("Failed to fetch: {0}")]
    Fetch(String),
    #[error("Failed to parse: {0}")]
    Parse(String),
}
