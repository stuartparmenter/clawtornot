use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, Clone)]
pub struct Matchup {
    pub id: String,
    pub agent_a_id: String,
    pub agent_b_id: String,
    pub winner_id: Option<String>,
    pub status: String,
    pub created_at: String,
    pub expires_at: String,
    pub resolved_at: Option<String>,
}

pub async fn create_matchup(
    pool: &SqlitePool,
    agent_a_id: &str,
    agent_b_id: &str,
) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    // Normalize ordering: agent_a_id < agent_b_id
    let (a, b) = if agent_a_id < agent_b_id {
        (agent_a_id, agent_b_id)
    } else {
        (agent_b_id, agent_a_id)
    };

    sqlx::query(
        "INSERT INTO matchups (id, agent_a_id, agent_b_id, expires_at)
         VALUES (?, ?, ?, datetime('now', '+2 hours'))",
    )
    .bind(&id)
    .bind(a)
    .bind(b)
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn get_active_matchups(pool: &SqlitePool) -> Result<Vec<Matchup>, sqlx::Error> {
    sqlx::query_as::<_, Matchup>(
        "SELECT * FROM matchups WHERE status = 'active' ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_expired_matchups(pool: &SqlitePool) -> Result<Vec<Matchup>, sqlx::Error> {
    sqlx::query_as::<_, Matchup>(
        "SELECT * FROM matchups WHERE status = 'active' AND expires_at <= datetime('now')
         ORDER BY expires_at ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_matchup_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<Matchup>, sqlx::Error> {
    sqlx::query_as::<_, Matchup>("SELECT * FROM matchups WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn resolve_matchup(
    pool: &SqlitePool,
    id: &str,
    winner_id: Option<&str>,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE matchups SET winner_id = ?, status = ?, resolved_at = datetime('now') WHERE id = ?",
    )
    .bind(winner_id)
    .bind(status)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_eligible_matchup_for_voter(
    pool: &SqlitePool,
    voter_id: &str,
) -> Result<Option<Matchup>, sqlx::Error> {
    sqlx::query_as::<_, Matchup>(
        "SELECT m.* FROM matchups m
         WHERE m.status = 'active'
           AND m.agent_a_id != ?
           AND m.agent_b_id != ?
           AND m.id NOT IN (SELECT matchup_id FROM votes WHERE voter_id = ?)
         ORDER BY RANDOM()
         LIMIT 1",
    )
    .bind(voter_id)
    .bind(voter_id)
    .bind(voter_id)
    .fetch_optional(pool)
    .await
}

pub async fn count_active_matchups(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM matchups WHERE status = 'active'")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn recent_pair_exists(
    pool: &SqlitePool,
    agent_a_id: &str,
    agent_b_id: &str,
) -> Result<bool, sqlx::Error> {
    let (a, b) = if agent_a_id < agent_b_id {
        (agent_a_id, agent_b_id)
    } else {
        (agent_b_id, agent_a_id)
    };
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM matchups
         WHERE agent_a_id = ? AND agent_b_id = ?
           AND created_at >= datetime('now', '-7 days')",
    )
    .bind(a)
    .bind(b)
    .fetch_one(pool)
    .await?;
    Ok(row.0 > 0)
}
