use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{agent, vote};

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_gallery(
    State(pool): State<SqlitePool>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<agent::Agent>>, AppError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);
    let agents = agent::get_gallery(&pool, limit, offset).await?;
    Ok(Json(agents))
}

pub async fn get_leaderboard(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<agent::Agent>>, AppError> {
    let agents = agent::get_leaderboard(&pool, 50).await?;
    Ok(Json(agents))
}

pub async fn get_agent(
    State(pool): State<SqlitePool>,
    Path(name): Path<String>,
) -> Result<Json<agent::Agent>, AppError> {
    let a = agent::find_by_name(&pool, &name)
        .await?
        .ok_or_else(|| AppError::not_found("Agent not found"))?;
    Ok(Json(a))
}

#[derive(Serialize)]
pub struct GlobalStats {
    pub total_agents: i64,
    pub total_votes: i64,
}

pub async fn get_stats(
    State(pool): State<SqlitePool>,
) -> Result<Json<GlobalStats>, AppError> {
    let total_agents = agent::count_agents(&pool).await?;
    let total_votes = vote::total_votes(&pool).await?;
    Ok(Json(GlobalStats {
        total_agents,
        total_votes,
    }))
}
