use crate::*;
use crate::{admin::Settings, auth::User};
use axum::extract::State;
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Destination {
    pub address: String,
    pub lat: f64,
    pub lng: f64,
}
pub fn fee(base: i64, per_km: i64, meters: i64) -> i64 {
    base + (per_km * meters + 500) / 1000
}
pub fn coverage(v: &Settings, lat: f64, lng: f64) -> Result<()> {
    coordinates(lat, lng)?;
    if lat < v.south || lat > v.north || lng < v.west || lng > v.east {
        return Err(ApiError::bad(
            "Address is outside the configured service area",
        ));
    }
    Ok(())
}
pub async fn quote(
    State(s): State<App>,
    u: User,
    Json(v): Json<Destination>,
) -> Result<Json<Value>> {
    u.require(&["customer"])?;
    let address = text(&v.address, 10, 500)?;
    let settings = crate::admin::load(&s.db).await?;
    coverage(&settings, v.lat, v.lng)?;
    if s.maps_key.is_empty() {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "Google Maps routing is not configured; checkout is unavailable".into(),
        ));
    }
    let recent: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM delivery_quotes WHERE customer_id=? AND expires_at>?",
    )
    .bind(&u.id)
    .bind(now())
    .fetch_one(&s.db)
    .await?;
    if recent >= 15 {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many route requests; retry later".into(),
        ));
    }
    let response=s.client.post(&s.maps_url).header("X-Goog-Api-Key",&s.maps_key).header("X-Goog-FieldMask","routes.distanceMeters").json(&json!({"origin":{"location":{"latLng":{"latitude":settings.pickup_lat,"longitude":settings.pickup_lng}}},"destination":{"location":{"latLng":{"latitude":v.lat,"longitude":v.lng}}},"travelMode":"DRIVE","routingPreference":"TRAFFIC_UNAWARE","computeAlternativeRoutes":false})).send().await.map_err(|_|ApiError(StatusCode::BAD_GATEWAY,"Route service unavailable; please retry".into()))?;
    if !response.status().is_success() {
        return Err(ApiError(
            StatusCode::BAD_GATEWAY,
            "Road distance could not be verified; please retry".into(),
        ));
    }
    let body: Value = response
        .json()
        .await
        .map_err(|_| ApiError::bad("Invalid routing response"))?;
    let meters = body
        .pointer("/routes/0/distanceMeters")
        .and_then(Value::as_i64)
        .filter(|m| *m > 0 && *m <= settings.max_distance_meters)
        .ok_or(ApiError::bad(
            "No eligible driving route within the service area",
        ))?;
    let fee = fee(settings.base_fee_centavos, settings.per_km_centavos, meters);
    let qid = id();
    let expires_at = now() + 600;
    sqlx::query("INSERT INTO delivery_quotes(id,customer_id,address,lat,lng,distance_meters,fee_centavos,base_fee_centavos,per_km_centavos,expires_at) VALUES (?,?,?,?,?,?,?,?,?,?)").bind(&qid).bind(&u.id).bind(&address).bind(v.lat).bind(v.lng).bind(meters).bind(fee).bind(settings.base_fee_centavos).bind(settings.per_km_centavos).bind(expires_at).execute(&s.db).await?;
    Ok(Json(
        json!({"id":qid,"address":address,"lat":v.lat,"lng":v.lng,"distance_meters":meters,"delivery_fee_centavos":fee,"base_fee_centavos":settings.base_fee_centavos,"per_km_centavos":settings.per_km_centavos,"expires_at":expires_at}),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_final_delivery_fee() {
        assert_eq!(fee(6000, 350, 5000), 7750);
        assert_eq!(fee(6000, 350, 1234), 6432);
        assert_eq!(fee(6000, 350, 10), 6004);
    }
}
