pub mod auth;
pub mod gallery;
pub mod live;
pub mod matchups;
pub mod profile;
pub mod register;
pub mod voting;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;

use live::Broadcaster;

/// Shared app state passed to all handlers that need both pool and broadcaster.
pub type AppState = (SqlitePool, Broadcaster);

pub fn api_router(pool: SqlitePool, broadcaster: Broadcaster) -> Router {
    let state: AppState = (pool.clone(), broadcaster.clone());

    // Authenticated routes — need auth middleware
    // The auth middleware uses pool from state.0
    let authed = Router::new()
        .route("/api/v1/me", get(profile::get_me).put(profile::update_me))
        .route("/api/v1/me/matchup", get(matchups::get_my_matchup))
        .route("/api/v1/matchups/{id}/vote", post(voting::cast_vote))
        .layer(middleware::from_fn_with_state(
            pool.clone(),
            auth::auth_middleware,
        ))
        .with_state(state.clone());

    // Public routes that need broadcaster (register)
    let public_with_bc = Router::new()
        .route("/api/v1/register", post(register::register))
        .with_state(state);

    // Public routes that only need pool
    let public_pool = Router::new()
        .route(
            "/api/v1/matchups/current",
            get(matchups::get_current_matchups),
        )
        .route("/api/v1/matchups/{id}", get(matchups::get_matchup))
        .route("/api/v1/agents/{name}", get(gallery::get_agent))
        .route("/api/v1/gallery", get(gallery::get_gallery))
        .route("/api/v1/leaderboard", get(gallery::get_leaderboard))
        .route("/api/v1/stats", get(gallery::get_stats))
        .with_state(pool);

    // WebSocket
    let ws = Router::new()
        .route("/api/v1/live", get(live::live_ws))
        .with_state(broadcaster);

    Router::new()
        .merge(public_with_bc)
        .merge(authed)
        .merge(public_pool)
        .merge(ws)
}
