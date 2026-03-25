use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use serde::Serialize;
use sqlx::SqlitePool;

use crate::api::auth::AuthAgent;
use crate::api::AppState;
use crate::error::AppError;
use crate::models::{agent, matchup, vote};

#[derive(Serialize)]
pub struct MatchupDetail {
    pub matchup: matchup::Matchup,
    pub agent_a: agent::Agent,
    pub agent_b: agent::Agent,
    pub tally: vote::VoteTally,
    pub comments: Vec<vote::Vote>,
}

pub async fn get_current_matchups(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<MatchupDetail>>, AppError> {
    let matchups = matchup::get_active_matchups(&pool).await?;
    let mut details = Vec::new();
    for m in matchups {
        let a = agent::find_by_id(&pool, &m.agent_a_id).await?.unwrap();
        let b = agent::find_by_id(&pool, &m.agent_b_id).await?.unwrap();
        let tally = vote::get_tally(&pool, &m.id).await?;
        let comments = vote::get_comments_for_matchup(&pool, &m.id).await?;
        details.push(MatchupDetail {
            matchup: m,
            agent_a: a,
            agent_b: b,
            tally,
            comments,
        });
    }
    Ok(Json(details))
}

pub async fn get_matchup(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
) -> Result<Json<MatchupDetail>, AppError> {
    let m = matchup::get_matchup_by_id(&pool, &id)
        .await?
        .ok_or_else(|| AppError::not_found("Matchup not found"))?;
    let a = agent::find_by_id(&pool, &m.agent_a_id).await?.unwrap();
    let b = agent::find_by_id(&pool, &m.agent_b_id).await?.unwrap();
    let tally = vote::get_tally(&pool, &m.id).await?;
    let comments = vote::get_comments_for_matchup(&pool, &m.id).await?;
    Ok(Json(MatchupDetail {
        matchup: m,
        agent_a: a,
        agent_b: b,
        tally,
        comments,
    }))
}

#[derive(Serialize)]
pub struct AssignedMatchup {
    pub matchup_id: String,
    pub agent_a: agent::Agent,
    pub agent_b: agent::Agent,
}

pub async fn get_my_matchup(
    State((pool, _)): State<AppState>,
    Extension(auth): Extension<AuthAgent>,
) -> Result<axum::response::Response, AppError> {
    let m = matchup::get_eligible_matchup_for_voter(&pool, &auth.0.id).await?;
    match m {
        Some(m) => {
            let a = agent::find_by_id(&pool, &m.agent_a_id).await?.unwrap();
            let b = agent::find_by_id(&pool, &m.agent_b_id).await?.unwrap();
            Ok(Json(AssignedMatchup {
                matchup_id: m.id,
                agent_a: a,
                agent_b: b,
            })
            .into_response())
        }
        None => Ok(axum::http::StatusCode::NO_CONTENT.into_response()),
    }
}
