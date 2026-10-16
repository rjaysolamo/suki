use crate::{auth::User, *};
use axum::extract::State;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Address {
    pub id: String,
    pub label: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AddressInput {
    pub label: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
}
pub async fn list(State(s): State<App>, u: User) -> Result<Json<Vec<Address>>> {
    u.require(&["customer"])?;
    Ok(Json(
        sqlx::query_as(
            "SELECT * FROM customer_addresses WHERE customer_id=? ORDER BY label LIMIT 20",
        )
        .bind(u.id)
        .fetch_all(&s.db)
        .await?,
    ))
}
pub async fn create(
    State(s): State<App>,
    u: User,
    Json(v): Json<AddressInput>,
) -> Result<Json<Value>> {
    u.require(&["customer"])?;
    coordinates(v.lat, v.lng)?;
    let label = text(&v.label, 1, 80)?;
    let address = text(&v.address, 10, 500)?;
    let mut tx = s.db.begin().await?;
    sqlx::query("SELECT id FROM users WHERE id=? FOR UPDATE")
        .bind(&u.id)
        .fetch_one(&mut *tx)
        .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM customer_addresses WHERE customer_id=?")
            .bind(&u.id)
            .fetch_one(&mut *tx)
            .await?;
    if count >= 20 {
        return Err(ApiError::bad("Maximum of 20 saved addresses"));
    }
    let aid = id();
    sqlx::query("INSERT INTO customer_addresses VALUES(?,?,?,?,?,?)")
        .bind(&aid)
        .bind(&u.id)
        .bind(label)
        .bind(address)
        .bind(v.lat)
        .bind(v.lng)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({"id":aid})))
}
