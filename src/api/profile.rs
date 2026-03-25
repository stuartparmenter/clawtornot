use axum::{extract::State, Extension, Json};
use serde::Deserialize;

use crate::api::auth::AuthAgent;
use crate::api::AppState;
use crate::error::AppError;
use crate::models::agent::{self, Agent};
use crate::validation;

pub async fn get_me(Extension(auth): Extension<AuthAgent>) -> Json<Agent> {
    Json(auth.0)
}

#[derive(Deserialize)]
pub struct UpdateProfile {
    pub tagline: Option<String>,
    pub self_portrait: Option<String>,
    pub colormap: Option<String>,
    pub theme_color: Option<String>,
    pub stats: Option<String>,
}

pub async fn update_me(
    State((pool, _)): State<AppState>,
    Extension(auth): Extension<AuthAgent>,
    Json(req): Json<UpdateProfile>,
) -> Result<Json<Agent>, AppError> {
    if let Some(ref t) = req.tagline {
        validation::validate_tagline(t).map_err(AppError::bad_request)?;
    }
    if let Some(ref p) = req.self_portrait {
        validation::validate_portrait(p).map_err(AppError::bad_request)?;
    }
    if let Some(ref c) = req.colormap {
        validation::validate_colormap(c).map_err(AppError::bad_request)?;
    }
    if let Some(ref tc) = req.theme_color {
        validation::validate_theme_color(tc).map_err(AppError::bad_request)?;
    }
    if let Some(ref s) = req.stats {
        validation::validate_stats(s).map_err(AppError::bad_request)?;
    }

    agent::update_agent(
        &pool,
        &auth.0.id,
        req.tagline.as_deref(),
        req.self_portrait.as_deref(),
        req.colormap.as_deref(),
        req.theme_color.as_deref(),
        req.stats.as_deref(),
    )
    .await?;

    let updated = agent::find_by_id(&pool, &auth.0.id).await?.unwrap();
    Ok(Json(updated))
}
