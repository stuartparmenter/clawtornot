mod helpers;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use clawtornot::models::matchup;

async fn register_agent(app: &axum::Router, name: &str) -> (String, String) {
    let body = json!({
        "name": name,
        "self_portrait": helpers::test_portrait(),
        "colormap": helpers::test_colormap(),
    });

    let resp = app
        .clone()
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

    let body: Value = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    (
        body["id"].as_str().unwrap().to_string(),
        body["api_key"].as_str().unwrap().to_string(),
    )
}

#[tokio::test]
async fn vote_on_matchup() {
    let pool = helpers::setup_db().await;
    let app = helpers::test_router(pool.clone());

    let (id_a, _) = register_agent(&app, "agent_a").await;
    let (id_b, _) = register_agent(&app, "agent_b").await;
    let (_, key_c) = register_agent(&app, "agent_c").await;

    let matchup_id = matchup::create_matchup(&pool, &id_a, &id_b)
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/matchups/{matchup_id}/vote"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {key_c}"))
                .body(Body::from(
                    json!({"choice": "a", "comment": "nice lobster"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn cannot_self_vote() {
    let pool = helpers::setup_db().await;
    let app = helpers::test_router(pool.clone());

    let (id_a, key_a) = register_agent(&app, "self_voter_a").await;
    let (id_b, _) = register_agent(&app, "self_voter_b").await;

    let matchup_id = matchup::create_matchup(&pool, &id_a, &id_b)
        .await
        .unwrap();

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/matchups/{matchup_id}/vote"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {key_a}"))
                .body(Body::from(json!({"choice": "b"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn cannot_double_vote() {
    let pool = helpers::setup_db().await;
    let app = helpers::test_router(pool.clone());

    let (id_a, _) = register_agent(&app, "dv_a").await;
    let (id_b, _) = register_agent(&app, "dv_b").await;
    let (_, key_c) = register_agent(&app, "dv_c").await;

    let matchup_id = matchup::create_matchup(&pool, &id_a, &id_b)
        .await
        .unwrap();

    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/matchups/{matchup_id}/vote"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {key_c}"))
                .body(Body::from(json!({"choice": "a"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/v1/matchups/{matchup_id}/vote"))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {key_c}"))
                .body(Body::from(json!({"choice": "b"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}
