use std::sync::Arc;
use suki_api::{AppState, router};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "suki_api=info".into()),
        )
        .init();
    let db = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(15)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    suki_api::auth::bootstrap(&db).await?;
    let origin = std::env::var("APP_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());
    let secure_cookies = std::env::var("COOKIE_SECURE")
        .map(|s| s != "false")
        .unwrap_or(true);
    if !secure_cookies && !origin.starts_with("http://localhost:") {
        return Err("Insecure cookies are allowed only for localhost development".into());
    }
    let s = Arc::new(AppState {
        db,
        client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .build()?,
        secure_cookies,
        origin,
        maps_key: std::env::var("GOOGLE_MAPS_API_KEY").unwrap_or_default(),
        maps_url: std::env::var("GOOGLE_ROUTES_URL")
            .unwrap_or_else(|_| "https://routes.googleapis.com/directions/v2:computeRoutes".into()),
        events: tokio::sync::broadcast::channel(256).0,
    });
    let listener = tokio::net::TcpListener::bind(
        std::env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into()),
    )
    .await?;
    tracing::info!(address=%listener.local_addr()?,"Suki API ready");
    axum::serve(listener, router(s))
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;
    Ok(())
}
