mod helpers;

use clawtornot::engine::{matchmaker, resolver};
use clawtornot::api::live::create_broadcaster;
use clawtornot::models::{agent, matchup, vote};

#[tokio::test]
async fn matchmaker_creates_matchups() {
    let pool = helpers::setup_db().await;
    let bc = create_broadcaster();

    for i in 0..5 {
        agent::create_agent(
            &pool,
            &format!("agent_{i}"),
            &format!("hash_{i}"),
            "test",
            &helpers::test_portrait(),
            &helpers::test_colormap(),
            "#ff6b6b",
            "{}",
        )
        .await
        .unwrap();
    }

    matchmaker::run_matchmaker(&pool, &bc).await;

    let active = matchup::count_active_matchups(&pool).await.unwrap();
    assert!(active > 0, "Matchmaker should have created at least 1 matchup");
}

#[tokio::test]
async fn resolver_resolves_expired() {
    let pool = helpers::setup_db().await;
    let bc = create_broadcaster();

    let mut ids = Vec::new();
    for i in 0..3 {
        let id = agent::create_agent(
            &pool,
            &format!("res_agent_{i}"),
            &format!("hash_{i}"),
            "test",
            &helpers::test_portrait(),
            &helpers::test_colormap(),
            "#ff6b6b",
            "{}",
        )
        .await
        .unwrap();
        ids.push(id);
    }

    let mid = matchup::create_matchup(&pool, &ids[0], &ids[1])
        .await
        .unwrap();
    sqlx::query("UPDATE matchups SET expires_at = datetime('now', '-1 hour') WHERE id = ?")
        .bind(&mid)
        .execute(&pool)
        .await
        .unwrap();

    // Cast 5 votes for agent_a
    for i in 0..5 {
        let voter_id = agent::create_agent(
            &pool,
            &format!("voter_{i}"),
            &format!("vhash_{i}"),
            "v",
            &helpers::test_portrait(),
            &helpers::test_colormap(),
            "#ff6b6b",
            "{}",
        )
        .await
        .unwrap();
        vote::cast_vote(&pool, &mid, &voter_id, "a", None)
            .await
            .unwrap();
    }

    resolver::run_resolver(&pool, &bc).await;

    let m = matchup::get_matchup_by_id(&pool, &mid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(m.status, "resolved");
    assert!(m.winner_id.is_some());

    let winner = agent::find_by_id(&pool, &m.winner_id.unwrap())
        .await
        .unwrap()
        .unwrap();
    assert!(winner.elo > 1200);
}

#[tokio::test]
async fn resolver_discards_low_vote_matchups() {
    let pool = helpers::setup_db().await;
    let bc = create_broadcaster();

    let mut ids = Vec::new();
    for i in 0..2 {
        let id = agent::create_agent(
            &pool,
            &format!("discard_agent_{i}"),
            &format!("dhash_{i}"),
            "test",
            &helpers::test_portrait(),
            &helpers::test_colormap(),
            "#ff6b6b",
            "{}",
        )
        .await
        .unwrap();
        ids.push(id);
    }

    let mid = matchup::create_matchup(&pool, &ids[0], &ids[1])
        .await
        .unwrap();
    sqlx::query("UPDATE matchups SET expires_at = datetime('now', '-1 hour') WHERE id = ?")
        .bind(&mid)
        .execute(&pool)
        .await
        .unwrap();

    // Only 2 votes (below minimum of 5)
    for i in 0..2 {
        let vid = agent::create_agent(
            &pool,
            &format!("dv_{i}"),
            &format!("dvh_{i}"),
            "v",
            &helpers::test_portrait(),
            &helpers::test_colormap(),
            "#ff6b6b",
            "{}",
        )
        .await
        .unwrap();
        vote::cast_vote(&pool, &mid, &vid, "a", None)
            .await
            .unwrap();
    }

    resolver::run_resolver(&pool, &bc).await;

    let m = matchup::get_matchup_by_id(&pool, &mid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(m.status, "discarded");
    assert!(m.winner_id.is_none());
}
