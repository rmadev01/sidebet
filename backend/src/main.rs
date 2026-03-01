mod config;
mod db;
mod models;
mod routes;
mod services;
mod ws;

use axum::http::HeaderValue;
use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Extension, Router,
};
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static MIGRATOR: Migrator = sqlx::migrate!("./src/db/migrations");

#[tokio::main]
async fn main() {
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
        .max_connections(20)
        .connect(&cfg.database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations at startup
    run_migrations(&pool).await;

    tracing::info!("Migrations applied successfully");

    // WebSocket broadcast
    let ws_broadcast = ws::create_broadcast();

    // CORS — must allow credentials for cookie-based auth
    let frontend_origin: HeaderValue = cfg
        .frontend_url
        .parse()
        .expect("FRONTEND_URL must be a valid origin");
    let cors = CorsLayer::new()
        .allow_origin(frontend_origin)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_credentials(true);

    // Auth-protected routes
    let protected = Router::new()
        // Users
        .route("/users/me", get(routes::users::get_me))
        .route("/users/me", patch(routes::users::update_me))
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
        // Feed
        .route("/feed", get(routes::feed::get_feed))
        // WebSocket
        .route("/ws", get(ws::ws_handler))
        .layer(middleware::from_fn(db::auth_middleware));

    // Public routes
    let public = Router::new()
        .route("/users/search", get(routes::users::search_users))
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
        .layer(Extension(pool))
        .layer(Extension(cfg.clone()))
        .layer(Extension(ws_broadcast))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", cfg.host, cfg.port);
    tracing::info!("Starting server on {addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn run_migrations(pool: &sqlx::PgPool) {
    MIGRATOR
        .run(pool)
        .await
        .expect("Failed to run SQL migrations");
}
