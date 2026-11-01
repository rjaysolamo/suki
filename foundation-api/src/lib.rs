pub mod identity;

use axum::{Router, extract::State, http::StatusCode, routing::get};
use sqlx::PgPool;

pub fn router(pool: PgPool) -> Router {
    Router::new()
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .with_state(pool)
}

async fn live() -> StatusCode {
    StatusCode::OK
}

async fn ready(State(pool): State<PgPool>) -> StatusCode {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
    {
        Ok(1) => StatusCode::OK,
        _ => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub mod pricing {
    pub const MINIMUM_MERCHANDISE_CENTAVOS: i64 = 50_000;
    pub const SERVICE_FEE_CENTAVOS: i64 = 8_700;
    pub const DELIVERY_RATE_CENTAVOS_PER_KM: i64 = 450;

    /// The caller supplies an independently verified one-way road distance in meters.
    pub fn quote(subtotal: i64, distance_meters: i64) -> Option<(i64, i64)> {
        if subtotal < MINIMUM_MERCHANDISE_CENTAVOS || distance_meters < 0 {
            return None;
        }
        let delivery = distance_meters
            .checked_mul(DELIVERY_RATE_CENTAVOS_PER_KM)?
            .checked_add(500)?
            / 1_000;
        let total = subtotal
            .checked_add(SERVICE_FEE_CENTAVOS)?
            .checked_add(delivery)?;
        Some((delivery, total))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn business_example_and_minimum() {
            assert_eq!(quote(60_000, 5_000), Some((2_250, 70_950)));
            assert_eq!(quote(49_999, 5_000), None);
            assert_eq!(quote(50_000, 0), Some((0, 58_700)));
        }
        #[test]
        fn rejects_invalid_and_overflow_inputs_and_rounds_centavos() {
            assert_eq!(quote(50_000, -1), None);
            assert_eq!(quote(i64::MAX, 1), None);
            assert_eq!(quote(50_000, i64::MAX), None);
            assert_eq!(quote(50_000, 1), Some((0, 58_700)));
            assert_eq!(quote(50_000, 2), Some((1, 58_701)));
        }
    }
}
