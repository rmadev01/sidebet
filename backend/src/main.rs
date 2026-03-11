mod config;
mod db;
mod models;
mod routes;
mod services;
mod ws;

use axum::http::HeaderValue;
use axum::{
    BoxError,
    error_handling::HandleErrorLayer,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post},
    Extension, Router,
};
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tower::ServiceBuilder;
use tower::limit::ConcurrencyLimitLayer;
use tower::timeout::TimeoutLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static MIGRATOR: Migrator = sqlx::migrate!("./src/db/migrations");

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        error!("backend failed to start: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    // Init tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sidebet_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = config::Config::from_env();
    cfg.log_startup_warnings();

    // Database pool
    let pool = PgPoolOptions::new()
        .max_connections(cfg.db_max_connections)
        .acquire_timeout(Duration::from_secs(cfg.db_acquire_timeout_secs))
        .connect(&cfg.database_url)
        .await
        .map_err(|error| format!("failed to connect to database: {error}"))?;

    // Run migrations at startup
    run_migrations(&pool).await?;

    tracing::info!("Migrations applied successfully");

    // WebSocket broadcast
    let ws_broadcast = ws::create_broadcast();
    let sync_throttle = routes::events::SyncThrottle::default();

    // CORS — must allow credentials for cookie-based auth
    // Note: can't use Any for methods/headers when allow_credentials is true
    let frontend_origins: Vec<HeaderValue> = cfg
        .allowed_frontend_origins()
        .into_iter()
        .map(|origin| origin.parse().map_err(|_| format!("invalid frontend origin: {origin}")))
        .collect::<Result<_, _>>()?;
    let cors = CorsLayer::new()
        .allow_origin(frontend_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::COOKIE,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);

    // Auth-protected routes
    let protected = Router::new()
        // Users
        .route("/users/me", get(routes::users::get_me))
        .route("/users/me", patch(routes::users::update_me))
        .route("/users/search", get(routes::users::search_users))
        // Event sync
        .route("/events/sync", post(routes::events::sync_events))
        // Friends
        .route("/friends", get(routes::friends::list_friends))
        .route("/friends/requests", get(routes::friends::incoming_requests))
        .route("/friends/request", post(routes::friends::send_request))
        .route(
            "/friends/{id}/accept",
            post(routes::friends::accept_request),
        )
        .route(
            "/friends/{id}/decline",
            post(routes::friends::decline_request),
        )
        .route("/friends/{id}", delete(routes::friends::remove_friend))
        // Bets
        .route("/bets", post(routes::bets::create_bet))
        .route("/bets", get(routes::bets::list_bets))
        .route("/bets/{id}", get(routes::bets::get_bet))
        .route("/bets/{id}/accept", post(routes::bets::accept_bet))
        .route("/bets/{id}/decline", post(routes::bets::decline_bet))
        .route("/bets/{id}/cancel", post(routes::bets::cancel_bet))
        .route("/bets/{id}/settle", post(routes::bets::settle_bet))
        .route("/bets/{id}/take", post(routes::bets::take_open_bet))
        // Wallet
        .route("/wallet/balance", get(routes::wallet::get_balance))
        .route(
            "/wallet/transactions",
            get(routes::wallet::get_transactions),
        )
        .route(
            "/wallet/daily-bonus",
            post(routes::wallet::claim_daily_bonus),
        )
        // Notifications
        .route(
            "/notifications",
            get(routes::notifications::get_notifications),
        )
        .route(
            "/notifications/read",
            post(routes::notifications::mark_read),
        )
        // Feed
        .route("/feed", get(routes::feed::get_feed))
        .route("/feed/open", get(routes::feed::get_open_bets))
        // WebSocket
        .route("/ws", get(ws::ws_handler))
        .layer(middleware::from_fn(db::auth_middleware));

    // Public routes
    let public = Router::new()
        .route(
            "/users/{username}",
            get(routes::users::get_user_by_username),
        )
        .route("/events", get(routes::events::list_events))
        .route("/events/{id}", get(routes::events::get_event))
        .route("/events/{id}/odds", get(routes::events::get_event_odds));

    let app = Router::new()
        .route("/health", get(routes::health::health))
        .nest("/api", protected)
        .nest("/api", public)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|_: BoxError| async {
                    axum::http::StatusCode::REQUEST_TIMEOUT
                }))
                .layer(ConcurrencyLimitLayer::new(1024))
                .layer(TimeoutLayer::new(Duration::from_secs(cfg.request_timeout_secs))),
        )
        .layer(DefaultBodyLimit::max(cfg.max_request_body_bytes))
        .layer(Extension(pool))
        .layer(Extension(cfg.clone()))
        .layer(Extension(ws_broadcast))
        .layer(Extension(sync_throttle))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", cfg.host, cfg.port);
    tracing::info!("Starting server on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|error| format!("failed to bind listener: {error}"))?;
    axum::serve(listener, app)
        .await
        .map_err(|error| format!("server error: {error}"))?;

    Ok(())
}

async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), String> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|error| format!("failed to run SQL migrations: {error}"))?;

    Ok(())
}
