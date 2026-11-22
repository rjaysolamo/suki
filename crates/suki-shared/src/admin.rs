use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub base_fee_centavos: i64,
    pub per_km_centavos: i64,
    pub minimum_centavos: i64,
    pub pickup_lat: f64,
    pub pickup_lng: f64,
    pub south: f64,
    pub north: f64,
    pub west: f64,
    pub east: f64,
    pub max_distance_meters: i64,
    pub gcash_qr_url: String,
    pub chat_retention_days: i32,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UserUpdate {
    pub role: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Expense {
    pub amount_centavos: i64,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderReport {
    pub order_id: String,
    pub estimated_total_centavos: i64,
    pub received_centavos: i64,
    pub delivery_fee_centavos: i64,
    pub purchase_cost_centavos: Option<i64>,
    pub refunds_centavos: i64,
    pub expenses_centavos: i64,
    pub margin_centavos: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Reports {
    pub counts: Vec<(String, i64)>,
    pub orders: Vec<OrderReport>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct AuditEntry {
    pub id: i64,
    pub actor_id: String,
    pub action: String,
    pub entity_id: String,
    pub details: String,
    pub created_at: i64,
}

/// Registration details shown only to authorized onboarding administrators.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct PendingApplicant {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub role: String,
    pub stall_name: Option<String>,
    pub created_at: i64,
}
