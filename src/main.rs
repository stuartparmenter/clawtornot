use clawtornot::api;
use clawtornot::config::Config;
use clawtornot::db;
use clawtornot::engine;
use clawtornot::web;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let config = Config::from_env();
    let pool = db::create_pool(&config.database_url).await;
    db::run_migrations(&pool).await;

    let broadcaster = api::live::create_broadcaster();
    let app = api::api_router(pool.clone(), broadcaster.clone())
        .merge(web::web_router(pool.clone()))
        .layer(axum::extract::DefaultBodyLimit::max(65_536));

    engine::spawn_background_tasks(pool, broadcaster);

    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!("Listening on {addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
