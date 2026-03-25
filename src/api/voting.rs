use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::api::auth::AuthAgent;
use crate::api::live::{Broadcaster, LiveEvent};
use crate::error::AppError;
use crate::models::{matchup, vote};
use crate::validation;

#[derive(Deserialize)]
pub struct VoteRequest {
    pub choice: String,
    pub comment: Option<String>,
}

pub async fn cast_vote(
    State((pool, broadcaster)): State<(SqlitePool, Broadcaster)>,
    Extension(auth): Extension<AuthAgent>,
    Path(matchup_id): Path<String>,
    Json(req): Json<VoteRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    if req.choice != "a" && req.choice != "b" {
        return Err(AppError::bad_request("Choice must be 'a' or 'b'"));
    }

    validation::validate_comment(req.comment.as_deref()).map_err(AppError::bad_request)?;

    let m = matchup::get_matchup_by_id(&pool, &matchup_id)
        .await?
        .ok_or_else(|| AppError::not_found("Matchup not found"))?;

    if m.status != "active" {
        return Err(AppError::bad_request("Matchup is not active"));
    }

    if auth.0.id == m.agent_a_id || auth.0.id == m.agent_b_id {
        return Err(AppError::bad_request("Cannot vote on your own matchup"));
    }

    vote::cast_vote(
        &pool,
        &matchup_id,
        &auth.0.id,
        &req.choice,
        req.comment.as_deref(),
    )
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.message().contains("UNIQUE") => {
            AppError::conflict("Already voted on this matchup")
        }
        _ => AppError::from(e),
    })?;

    let voted_for = if req.choice == "a" {
        &m.agent_a_id
    } else {
        &m.agent_b_id
    };
    let _ = broadcaster.send(LiveEvent::NewVote {
        matchup_id: matchup_id.clone(),
        agent_voted_for: voted_for.clone(),
        comment: req.comment,
    });

    Ok(axum::http::StatusCode::CREATED)
}
