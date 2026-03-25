use serde::Serialize;
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Vote {
    pub id: String,
    pub matchup_id: String,
    pub voter_id: String,
    pub choice: String,
    pub comment: Option<String>,
    pub created_at: String,
}

pub async fn cast_vote(
    pool: &SqlitePool,
    matchup_id: &str,
    voter_id: &str,
    choice: &str,
    comment: Option<&str>,
) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO votes (id, matchup_id, voter_id, choice, comment) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(matchup_id)
    .bind(voter_id)
    .bind(choice)
    .bind(comment)
    .execute(pool)
    .await?;
    Ok(id)
}

#[derive(Debug, Serialize)]
pub struct VoteTally {
    pub votes_a: i64,
    pub votes_b: i64,
}

pub async fn get_tally(pool: &SqlitePool, matchup_id: &str) -> Result<VoteTally, sqlx::Error> {
    let a: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM votes WHERE matchup_id = ? AND choice = 'a'")
            .bind(matchup_id)
            .fetch_one(pool)
            .await?;

    let b: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM votes WHERE matchup_id = ? AND choice = 'b'")
            .bind(matchup_id)
            .fetch_one(pool)
            .await?;

    Ok(VoteTally {
        votes_a: a.0,
        votes_b: b.0,
    })
}

pub async fn get_comments_for_matchup(
    pool: &SqlitePool,
    matchup_id: &str,
) -> Result<Vec<Vote>, sqlx::Error> {
    sqlx::query_as::<_, Vote>(
        "SELECT * FROM votes WHERE matchup_id = ? AND comment IS NOT NULL ORDER BY created_at DESC",
    )
    .bind(matchup_id)
    .fetch_all(pool)
    .await
}

pub async fn total_votes(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM votes")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}
