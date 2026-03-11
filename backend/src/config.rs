use std::env;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

impl AppEnv {
    fn from_env() -> Self {
        match env::var("APP_ENV")
            .unwrap_or_else(|_| "development".into())
            .to_ascii_lowercase()
            .as_str()
        {
            "prod" | "production" => Self::Production,
            _ => Self::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub app_env: AppEnv,
    pub database_url: String,
    pub db_max_connections: u32,
    pub db_acquire_timeout_secs: u64,
    pub request_timeout_secs: u64,
    pub max_request_body_bytes: usize,
    pub events_sync_cooldown_secs: u64,
    pub host: String,
    pub port: u16,
    pub frontend_url: String,
    pub sportsgameodds_api_key: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let app_env = AppEnv::from_env();
        let frontend_url =
            env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".into());

        Self {
            app_env,
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            db_max_connections: env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(20),
            db_acquire_timeout_secs: env::var("DB_ACQUIRE_TIMEOUT_SECS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(5),
            request_timeout_secs: env::var("REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(15),
            max_request_body_bytes: env::var("MAX_REQUEST_BODY_BYTES")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(65_536),
            events_sync_cooldown_secs: env::var("EVENTS_SYNC_COOLDOWN_SECS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(60),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("PORT must be a number"),
            frontend_url,
            sportsgameodds_api_key: env::var("SPORTSGAMEODDS_API_KEY").unwrap_or_default(),
        }
    }

    pub fn log_startup_warnings(&self) {
        if !self.app_env.is_production() {
            tracing::warn!("Running backend in development mode");
        }

        if self.sportsgameodds_api_key.is_empty() {
            tracing::warn!(
                "SPORTSGAMEODDS_API_KEY is not set; odds refresh endpoints will return cached values only"
            );
        }
    }

    pub fn allowed_frontend_origins(&self) -> Vec<String> {
        let mut origins: Vec<String> = self
            .frontend_url
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        if origins.is_empty() {
            origins.push("http://localhost:5173".to_string());
        }

        if !self.app_env.is_production() {
            for default_origin in [
                "http://localhost:5173",
                "http://localhost:5174",
                "http://localhost:5175",
                "http://127.0.0.1:5173",
                "http://127.0.0.1:5174",
                "http://127.0.0.1:5175",
            ] {
                if !origins.iter().any(|origin| origin == default_origin) {
                    origins.push(default_origin.to_string());
                }
            }
        }

        origins
    }
}
