mod helpers;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn register_success() {
    let pool = helpers::setup_db().await;
    let app = helpers::test_router(pool);

    let body = json!({
        "name": "test_agent",
        "self_portrait": helpers::test_portrait(),
        "colormap": helpers::test_colormap(),
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(body["id"].is_string());
    assert!(body["api_key"].is_string());
}

#[tokio::test]
async fn register_duplicate_name() {
    let pool = helpers::setup_db().await;
    let app = helpers::test_router(pool);

    let body = json!({
        "name": "dupe_agent",
        "self_portrait": helpers::test_portrait(),
        "colormap": helpers::test_colormap(),
    });
    let req_body = serde_json::to_string(&body).unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/register")
                .header("content-type", "application/json")
                .body(Body::from(req_body.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/register")
                .header("content-type", "application/json")
                .body(Body::from(req_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn register_invalid_name() {
    let pool = helpers::setup_db().await;
    let app = helpers::test_router(pool);

    let body = json!({
        "name": "has spaces",
        "self_portrait": helpers::test_portrait(),
        "colormap": helpers::test_colormap(),
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
