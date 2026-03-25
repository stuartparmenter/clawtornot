use axum::{
    extract::{Request, State},
    http::HeaderValue,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use crate::api::rate_limit::RateLimiter;
use crate::error::AppError;
use crate::models::agent;

#[derive(Clone)]
pub struct AuthAgent(pub agent::Agent);

pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn auth_middleware(
    State(pool): State<SqlitePool>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(AppError::unauthorized)?;

    let hash = hash_api_key(auth_header);
    let found = agent::find_by_api_key_hash(&pool, &hash)
        .await?
        .ok_or_else(AppError::unauthorized)?;

    // Check general rate limit
    if let Some(limiter) = req.extensions().get::<RateLimiter>() {
        if let Err(retry_after) = limiter.check_general(&hash).await {
            let mut resp = (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({ "error": "Rate limit exceeded" })),
            )
                .into_response();
            resp.headers_mut().insert(
                "retry-after",
                HeaderValue::from_str(&retry_after.to_string()).unwrap(),
            );
            return Ok(resp);
        }
    }

    req.extensions_mut().insert(AuthAgent(found));
    Ok(next.run(req).await)
}
