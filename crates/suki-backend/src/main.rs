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
    let integrity_key = suki_shared::crypto::IntegrityKey::from_hex(
        &std::env::var("SUKI_INTEGRITY_KEY")
            .map_err(|_| "Set SUKI_INTEGRITY_KEY before starting the API")?,
    )
    .map_err(|_| "SUKI_INTEGRITY_KEY must be 32 bytes encoded as 64 hex characters")?;
    let field_cipher = Arc::new(suki_api::protected::FieldCipher::from_hex(
        &std::env::var("DATA_ENCRYPTION_KEY")
            .map_err(|_| "Set DATA_ENCRYPTION_KEY before starting the API")?,
    )?);
    let session_key = Arc::new(suki_api::sessions::SessionKey::from_hex(
        &std::env::var("SESSION_SIGNING_KEY")
            .map_err(|_| "Set SESSION_SIGNING_KEY before starting the API")?,
    )?);
    let admin_emails = match std::env::var("ADMIN_EMAILS") {
        Ok(value) => suki_api::auth::AdminEmails::parse(&value)?,
        Err(std::env::VarError::NotPresent) => suki_api::auth::AdminEmails::default(),
        Err(error) => return Err(error.into()),
    };
    let db = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(15)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    suki_api::protected::backfill(&db, &field_cipher)
        .await
        .map_err(|_| "Protected data migration or key validation failed")?;
    suki_api::auth::bootstrap(&db, &admin_emails).await?;
    let origin = std::env::var("APP_ORIGIN").unwrap_or_else(|_| "http://localhost:8080".into());
    let secure_cookies = std::env::var("COOKIE_SECURE")
        .map(|s| s != "false")
        .unwrap_or(true);
    if !secure_cookies && !origin.starts_with("http://localhost:") {
        return Err("Insecure cookies are allowed only for localhost development".into());
    }
    let s = Arc::new(AppState {
        db,
        rate_limits: Default::default(),
        session_key,
        admin_emails,
        integrity_key,
        field_cipher,
        client: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(12))
            .build()?,
        secure_cookies,
        origin,
        maps_key: std::env::var("GOOGLE_MAPS_API_KEY").unwrap_or_default(),
        maps_url: std::env::var("GOOGLE_ROUTES_URL")
            .unwrap_or_else(|_| "https://routes.googleapis.com/directions/v2:computeRoutes".into()),
        r2: suki_api::storage::R2::from_env()?,
        legacy_quotes: std::env::var("SUKI_ALLOW_LEGACY_QUOTES").as_deref() == Ok("true"),
        media_dir: std::env::var_os("SUKI_MEDIA_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "target/suki-media".into()),
        events: tokio::sync::broadcast::channel(256).0,
    });

    let listener = tokio::net::TcpListener::bind(
        std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
    )
    .await?;

    tracing::info!(address=%listener.local_addr()?,"Suki API ready");
    let images = std::env::var_os("SUKI_IMAGE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/images"));
    let mut app = router(s).nest_service("/images", tower_http::services::ServeDir::new(images));
    if let Some(preview) = std::env::var_os("SUKI_PREVIEW_DIR") {
        app = app.fallback_service(tower_http::services::ServeDir::new(preview));
    }
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.ok();
    })
    .await?;
    Ok(())
}
