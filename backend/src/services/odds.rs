use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Fetch NBA odds from The Odds API
pub async fn fetch_nba_odds(api_key: &str) -> Result<Vec<OddsEvent>, OddsError> {
    let client = Client::new();
    let resp = client
        .get("https://api.the-odds-api.com/v4/sports/basketball_nba/odds")
        .query(&[
            ("apiKey", api_key),
            ("regions", "us"),
            ("markets", "h2h,spreads,totals"),
            ("oddsFormat", "decimal"),
        ])
        .send()
        .await
        .map_err(|e| OddsError::Fetch(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(OddsError::Fetch(format!("Status: {}", resp.status())));
    }

    let events: Vec<OddsEvent> = resp
        .json()
        .await
        .map_err(|e| OddsError::Parse(e.to_string()))?;

    Ok(events)
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
    if prob <= 0.0 { return 100.0; }
    1.0 / prob
}

/// Convert decimal odds to implied probability
pub fn decimal_odds_to_probability(odds: f64) -> f64 {
    if odds <= 0.0 { return 0.0; }
    1.0 / odds
}

// ── Types ──

#[derive(Debug, Serialize, Deserialize)]
pub struct OddsEvent {
    pub id: String,
    pub sport_key: Option<String>,
    pub commence_time: Option<String>,
    pub home_team: Option<String>,
    pub away_team: Option<String>,
    pub bookmakers: Option<Vec<Bookmaker>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bookmaker {
    pub key: String,
    pub title: String,
    pub markets: Option<Vec<Market>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Market {
    pub key: String,
    pub outcomes: Option<Vec<Outcome>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Outcome {
    pub name: String,
    pub price: f64,
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
