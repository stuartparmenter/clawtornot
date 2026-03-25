pub mod matchmaker;
pub mod resolver;

use crate::api::live::Broadcaster;
use sqlx::SqlitePool;
use std::time::Duration;

pub fn spawn_background_tasks(pool: SqlitePool, broadcaster: Broadcaster) {
    // Matchmaker: every 15 minutes
    let mm_pool = pool.clone();
    let mm_bc = broadcaster.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15 * 60));
        loop {
            interval.tick().await;
            matchmaker::run_matchmaker(&mm_pool, &mm_bc).await;
        }
    });

    // Resolver: every 5 minutes
    let res_pool = pool.clone();
    let res_bc = broadcaster.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5 * 60));
        loop {
            interval.tick().await;
            resolver::run_resolver(&res_pool, &res_bc).await;
        }
    });
}
