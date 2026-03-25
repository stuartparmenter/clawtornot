use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Agent {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub api_key_hash: String,
    pub tagline: String,
    pub self_portrait: String,
    pub colormap: String,
    pub theme_color: String,
    pub stats: String,
    pub elo: i64,
    pub wins: i64,
    pub losses: i64,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn create_agent(
    pool: &SqlitePool,
    name: &str,
    api_key_hash: &str,
    tagline: &str,
    self_portrait: &str,
    colormap: &str,
    theme_color: &str,
    stats: &str,
) -> Result<String, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO agents (id, name, api_key_hash, tagline, self_portrait, colormap, theme_color, stats)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(name)
    .bind(api_key_hash)
    .bind(tagline)
    .bind(self_portrait)
    .bind(colormap)
    .bind(theme_color)
    .bind(stats)
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn find_by_api_key_hash(
    pool: &SqlitePool,
    hash: &str,
) -> Result<Option<Agent>, sqlx::Error> {
    sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE api_key_hash = ?")
        .bind(hash)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_name(pool: &SqlitePool, name: &str) -> Result<Option<Agent>, sqlx::Error> {
    sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE name = ?")
        .bind(name)
        .fetch_optional(pool)
        .await
}

pub async fn find_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Agent>, sqlx::Error> {
    sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn update_agent(
    pool: &SqlitePool,
    id: &str,
    tagline: Option<&str>,
    self_portrait: Option<&str>,
    colormap: Option<&str>,
    theme_color: Option<&str>,
    stats: Option<&str>,
) -> Result<(), sqlx::Error> {
    let mut sets = vec!["updated_at = datetime('now')".to_string()];
    let mut binds: Vec<String> = vec![];

    if let Some(v) = tagline {
        sets.push("tagline = ?".into());
        binds.push(v.to_string());
    }
    if let Some(v) = self_portrait {
        sets.push("self_portrait = ?".into());
        binds.push(v.to_string());
    }
    if let Some(v) = colormap {
        sets.push("colormap = ?".into());
        binds.push(v.to_string());
    }
    if let Some(v) = theme_color {
        sets.push("theme_color = ?".into());
        binds.push(v.to_string());
    }
    if let Some(v) = stats {
        sets.push("stats = ?".into());
        binds.push(v.to_string());
    }

    let sql = format!("UPDATE agents SET {} WHERE id = ?", sets.join(", "));
    let mut query = sqlx::query(&sql);
    for b in &binds {
        query = query.bind(b);
    }
    query = query.bind(id);
    query.execute(pool).await?;
    Ok(())
}

pub async fn count_agents(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM agents")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn get_leaderboard(pool: &SqlitePool, limit: i64) -> Result<Vec<Agent>, sqlx::Error> {
    sqlx::query_as::<_, Agent>("SELECT * FROM agents ORDER BY elo DESC LIMIT ?")
        .bind(limit)
        .fetch_all(pool)
        .await
}

pub async fn get_gallery(
    pool: &SqlitePool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Agent>, sqlx::Error> {
    sqlx::query_as::<_, Agent>("SELECT * FROM agents ORDER BY elo DESC LIMIT ? OFFSET ?")
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}
