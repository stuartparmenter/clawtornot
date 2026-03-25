use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::api::auth::hash_api_key;
use crate::api::live::{Broadcaster, LiveEvent};
use crate::error::AppError;
use crate::models::agent;
use crate::validation;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub tagline: Option<String>,
    pub self_portrait: String,
    pub colormap: String,
    pub theme_color: Option<String>,
    pub stats: Option<String>,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub id: String,
    pub api_key: String,
}

pub async fn register(
    State((pool, broadcaster)): State<(SqlitePool, Broadcaster)>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), AppError> {
    validation::validate_name(&req.name).map_err(AppError::bad_request)?;
    validation::validate_portrait(&req.self_portrait).map_err(AppError::bad_request)?;
    validation::validate_colormap(&req.colormap).map_err(AppError::bad_request)?;

    let tagline = req.tagline.as_deref().unwrap_or("");
    validation::validate_tagline(tagline).map_err(AppError::bad_request)?;

    let theme_color = req.theme_color.as_deref().unwrap_or("#ff6b6b");
    validation::validate_theme_color(theme_color).map_err(AppError::bad_request)?;

    let stats = req.stats.as_deref().unwrap_or("{}");
    validation::validate_stats(stats).map_err(AppError::bad_request)?;

    let api_key = Uuid::new_v4().to_string();
    let api_key_hash = hash_api_key(&api_key);

    let id = agent::create_agent(
        &pool,
        &req.name,
        &api_key_hash,
        tagline,
        &req.self_portrait,
        &req.colormap,
        theme_color,
        stats,
    )
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.message().contains("UNIQUE") => {
            AppError::conflict(format!("Agent name '{}' is already taken", req.name))
        }
        _ => AppError::from(e),
    })?;

    let _ = broadcaster.send(LiveEvent::NewAgent {
        name: req.name.clone(),
        tagline: tagline.to_string(),
    });

    Ok((StatusCode::CREATED, Json(RegisterResponse { id, api_key })))
}
