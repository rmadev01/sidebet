use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub host: String,
    pub port: u16,
    pub odds_api_key: String,
    pub rpc_url: String,
    pub contract_address: String,
    pub auth_sidecar_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".into()),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()
                .expect("PORT must be a number"),
            odds_api_key: env::var("ODDS_API_KEY").unwrap_or_default(),
            rpc_url: env::var("RPC_URL").unwrap_or_default(),
            contract_address: env::var("SIDEBET_CONTRACT_ADDRESS")
                .unwrap_or_default(),
            auth_sidecar_url: env::var("BETTER_AUTH_URL")
                .unwrap_or_else(|_| "http://localhost:3001".into()),
        }
    }
}
