use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub frontend_url: String,
    pub odds_api_key: String,
    pub auth_sidecar_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("PORT must be a number"),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".into()),
            odds_api_key: env::var("ODDS_API_KEY").unwrap_or_default(),
            auth_sidecar_url: env::var("BETTER_AUTH_URL")
                .unwrap_or_else(|_| "http://localhost:3001".into()),
        }
    }

    pub fn log_startup_warnings(&self) {
        if self.odds_api_key.is_empty() {
            tracing::warn!(
                "ODDS_API_KEY is not set; odds refresh endpoints will return cached values only"
            );
        }
    }
}
