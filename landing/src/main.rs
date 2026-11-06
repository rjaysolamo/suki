mod data;
mod style;
mod templates;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct QuoteRequest {
    pub district: String,
    pub items: Vec<QuoteItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QuoteItem {
    pub name: String,
    pub qty: f64,
}

#[derive(Debug, Serialize)]
pub struct QuoteResponse {
    pub district: String,
    pub distance_km: f64,
    pub est_minutes: u32,
    pub items_subtotal_cents: u64,
    pub delivery_fee_cents: u64,
    pub total_cents: u64,
    pub items_subtotal_peso: f64,
    pub delivery_fee_peso: f64,
    pub total_peso: f64,
    pub calculated_in_microseconds: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,suki_landing=debug".into()),
        )
        .init();

    // Serve shared root assets (images, css, js)
    let assets_dir = if PathBuf::from("assets").exists() {
        PathBuf::from("assets")
    } else if PathBuf::from("../assets").exists() {
        PathBuf::from("../assets")
    } else {
        PathBuf::from("assets")
    };

    info!("Serving static assets from: {:?}", assets_dir);

    let app = Router::new()
        .route("/", get(handle_index))
        .route("/health", get(handle_health))
        .route("/api/health", get(handle_health))
        .route("/api/hubs", get(handle_get_hubs))
        .route("/api/prices", get(handle_get_prices))
        .route("/api/districts", get(handle_get_districts))
        .route("/api/quote", post(handle_calculate_quote))
        .nest_service("/assets", ServeDir::new(assets_dir))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Suki Davao Pure Rust Landing Page starting on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("Failed to bind to {}: {}", addr, e));

    axum::serve(listener, app).await.unwrap();
}

async fn handle_index() -> impl IntoResponse {
    Html(templates::render_landing_page())
}

async fn handle_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "suki-landing",
        "platform": "Davao City Market Platform",
        "language": "100% Pure Rust",
        "rust_edition": "2024",
        "framework": "Axum 0.8"
    }))
}

async fn handle_get_hubs() -> impl IntoResponse {
    Json(data::get_market_hubs())
}

async fn handle_get_prices() -> impl IntoResponse {
    Json(data::get_price_index())
}

async fn handle_get_districts() -> impl IntoResponse {
    Json(data::get_davao_districts())
}

async fn handle_calculate_quote(
    Json(payload): Json<QuoteRequest>,
) -> Result<Json<QuoteResponse>, StatusCode> {
    let start_time = std::time::Instant::now();

    let districts = data::get_davao_districts();
    let matched_district = districts
        .iter()
        .find(|d| d.name.to_lowercase().contains(&payload.district.to_lowercase()))
        .unwrap_or(&districts[0]);

    let price_items = data::get_price_index();
    let mut subtotal_cents: u64 = 0;

    for item in &payload.items {
        if let Some(catalog_item) = price_items.iter().find(|p| p.name.eq_ignore_ascii_case(&item.name)) {
            let item_cents = (catalog_item.palengke_price * 100.0) as u64;
            let qty_millikg = (item.qty * 1000.0) as u64;
            subtotal_cents += (item_cents * qty_millikg) / 1000;
        }
    }

    let delivery_fee_cents = (matched_district.delivery_fee * 100.0) as u64;
    let total_cents = subtotal_cents + delivery_fee_cents;
    let elapsed_micros = start_time.elapsed().as_micros() as u64;

    Ok(Json(QuoteResponse {
        district: matched_district.name.to_string(),
        distance_km: matched_district.distance_km,
        est_minutes: matched_district.est_minutes,
        items_subtotal_cents: subtotal_cents,
        delivery_fee_cents: delivery_fee_cents,
        total_cents: total_cents,
        items_subtotal_peso: subtotal_cents as f64 / 100.0,
        delivery_fee_peso: delivery_fee_cents as f64 / 100.0,
        total_peso: total_cents as f64 / 100.0,
        calculated_in_microseconds: elapsed_micros,
    }))
}
