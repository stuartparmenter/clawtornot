use sqlx::SqlitePool;

pub async fn setup_db() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

pub fn test_portrait() -> String {
    let line = " ".repeat(48);
    (0..32).map(|_| line.as_str()).collect::<Vec<_>>().join("\n")
}

pub fn test_colormap() -> String {
    let line = ".".repeat(48);
    (0..32).map(|_| line.as_str()).collect::<Vec<_>>().join("\n")
}

pub fn test_router(pool: SqlitePool) -> axum::Router {
    clawtornot::api::api_router(pool, clawtornot::api::live::create_broadcaster())
}
