pub mod addresses;
pub mod admin;
pub mod auth;
pub mod catalog;
pub mod delivery;
pub mod orders;
pub mod payments;
pub mod realtime;
pub mod vendors;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, patch, post, put},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: MySqlPool,
    pub client: reqwest::Client,
    pub secure_cookies: bool,
    pub origin: String,
    pub maps_key: String,
    pub maps_url: String,
    pub events: broadcast::Sender<(String, Value)>,
}
pub type App = Arc<AppState>;
pub type Result<T> = std::result::Result<T, ApiError>;
#[derive(Debug)]
pub struct ApiError(pub StatusCode, pub String);
impl ApiError {
    pub fn bad(s: &str) -> Self {
        Self(StatusCode::BAD_REQUEST, s.into())
    }
    pub fn forbidden() -> Self {
        Self(StatusCode::FORBIDDEN, "Access denied".into())
    }
    pub fn conflict(s: &str) -> Self {
        Self(StatusCode::CONFLICT, s.into())
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref d) = e
            && d.is_unique_violation()
        {
            return Self::conflict("A record with these details already exists");
        }
        tracing::error!(error=%e,"database operation failed");
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database operation failed".into(),
        )
    }
}
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
pub fn id() -> String {
    uuid::Uuid::new_v4().to_string()
}
pub fn hash(s: &str) -> String {
    use std::fmt::Write;
    Sha256::digest(s.as_bytes())
        .iter()
        .fold(String::with_capacity(64), |mut out, b| {
            write!(out, "{b:02x}").expect("Writing to String cannot fail");
            out
        })
}
pub fn text(s: &str, min: usize, max: usize) -> Result<String> {
    let s = s.trim();
    if s.len() < min || s.len() > max || s.chars().any(|c| c == '\0') {
        return Err(ApiError::bad("Invalid text length or content"));
    }
    Ok(s.into())
}
pub fn cost(price: i64, quantity: i64) -> Result<i64> {
    if !(0..=100_000_000).contains(&price) || !(0..=1_000_000).contains(&quantity) {
        return Err(ApiError::bad("Invalid price or quantity"));
    }
    Ok((price * quantity + 500) / 1000)
}
pub fn coordinates(lat: f64, lng: f64) -> Result<()> {
    if !lat.is_finite()
        || !lng.is_finite()
        || !(-90.0..=90.0).contains(&lat)
        || !(-180.0..=180.0).contains(&lng)
    {
        return Err(ApiError::bad("Invalid coordinates"));
    }
    Ok(())
}
pub async fn audit(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    actor: &str,
    action: &str,
    entity: &str,
    details: Value,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO audit_logs(actor_id,action,entity_id,details,created_at) VALUES (?,?,?,?,?)",
    )
    .bind(actor)
    .bind(action)
    .bind(entity)
    .bind(details.to_string())
    .bind(now())
    .execute(&mut **tx)
    .await?;
    Ok(())
}
async fn safety(
    State(s): State<App>,
    headers: HeaderMap,
    req: axum::extract::Request,
    next: Next,
) -> Response {
    if !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        if headers.get("x-suki-request").and_then(|v| v.to_str().ok()) != Some("1") {
            return ApiError::forbidden().into_response();
        }
        if let Some(origin) = headers.get("origin")
            && origin.to_str().ok() != Some(&s.origin)
        {
            return ApiError::forbidden().into_response();
        }
    }
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
        .headers_mut()
        .insert("x-content-type-options", "nosniff".parse().unwrap());
    response
}
pub fn router(s: App) -> Router {
    Router::new()
        .route(
            "/api/addresses",
            get(addresses::list).post(addresses::create),
        )
        .route(
            "/api/categories",
            get(vendors::categories).post(vendors::add_category),
        )
        .route("/api/vendor/items", get(vendors::items))
        .route("/api/vendor/items/{id}", put(vendors::confirm))
        .route("/api/health", get(health))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/products", get(catalog::list).post(catalog::create))
        .route("/api/products/{id}", put(catalog::update))
        .route(
            "/api/settings",
            get(admin::settings).put(admin::update_settings),
        )
        .route("/api/delivery/quote", post(delivery::quote))
        .route("/api/orders", get(orders::list).post(orders::create))
        .route("/api/orders/{id}", get(orders::detail))
        .route("/api/orders/{id}/status", patch(orders::transition))
        .route("/api/orders/{id}/assign", put(orders::assign))
        .route(
            "/api/orders/{id}/items/{item}/purchase",
            put(orders::purchase),
        )
        .route("/api/orders/{id}/reconcile", post(orders::reconcile))
        .route(
            "/api/orders/{id}/adjustments",
            post(orders::propose_adjustment),
        )
        .route(
            "/api/orders/{id}/adjustments/{adjustment}",
            post(orders::decide_adjustment),
        )
        .route(
            "/api/orders/{id}/substitutions",
            post(orders::propose_substitution),
        )
        .route(
            "/api/orders/{id}/substitutions/{substitution}",
            post(orders::decide_substitution),
        )
        .route("/api/orders/{id}/payments", post(payments::submit))
        .route("/api/payments/{id}/verify", post(payments::verify))
        .route("/api/refunds/{id}/process", post(payments::refund))
        .route(
            "/api/orders/{id}/messages",
            get(realtime::messages).post(realtime::send_message),
        )
        .route(
            "/api/orders/{id}/location",
            get(realtime::location).put(realtime::update_location),
        )
        .route("/api/orders/{id}/events", get(realtime::websocket))
        .route("/api/admin/users", get(admin::users))
        .route("/api/admin/users/{id}", patch(admin::update_user))
        .route("/api/admin/reports", get(admin::reports))
        .route("/api/admin/audit", get(admin::logs))
        .route("/api/orders/{id}/expenses", post(admin::expense))
        .layer(DefaultBodyLimit::max(32 * 1024))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(middleware::from_fn_with_state(s.clone(), safety))
        .with_state(s)
}

async fn health(State(s): State<App>) -> Result<Json<Value>> {
    sqlx::query("SELECT 1").execute(&s.db).await?;
    Ok(Json(json!({"status":"ok"})))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_currency() {
        assert_eq!(cost(350, 1234).unwrap(), 432);
        assert_eq!(cost(8_000, 3000).unwrap(), 24_000);
        assert!(cost(-1, 1000).is_err());
        assert!(cost(100, i64::MAX).is_err())
    }
    #[test]
    fn valid_coordinates() {
        assert!(coordinates(f64::NAN, 125.0).is_err());
        assert!(coordinates(7.1, 181.0).is_err());
        assert!(coordinates(7.1, 125.6).is_ok())
    }
}
