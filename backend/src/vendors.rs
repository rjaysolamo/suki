use crate::{auth::User, *};
use axum::extract::{Path, State};
use serde::Deserialize;
type VendorItem = (String, String, String, i64, String, String, Option<i64>);
pub async fn items(State(s): State<App>, u: User) -> Result<Json<Vec<Value>>> {
    u.require(&["vendor"])?;
    let rows:Vec<VendorItem>=sqlx::query_as("SELECT i.id,i.order_id,i.name,i.quantity_milli,i.unit,o.status,i.available_quantity_milli FROM order_items i JOIN orders o ON o.id=i.order_id WHERE i.vendor_id=? AND o.status IN ('paid','shopping','price_adjustment') ORDER BY o.created_at LIMIT 500").bind(&u.id).fetch_all(&s.db).await?;
    Ok(Json(rows.into_iter().map(|r|json!({"item_id":r.0,"order_id":r.1,"name":r.2,"quantity_milli":r.3,"unit":r.4,"order_status":r.5,"available_quantity_milli":r.6})).collect()))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Availability {
    available_quantity_milli: i64,
}
pub async fn confirm(
    State(s): State<App>,
    u: User,
    Path(iid): Path<String>,
    Json(v): Json<Availability>,
) -> Result<Json<Value>> {
    u.require(&["vendor"])?;
    let mut tx = s.db.begin().await?;
    let row:Option<(String,i64,String)>=sqlx::query_as("SELECT i.vendor_id,i.quantity_milli,o.status FROM order_items i JOIN orders o ON o.id=i.order_id WHERE i.id=? FOR UPDATE").bind(&iid).fetch_optional(&mut *tx).await?;
    let (vendor, quantity, status) = row.ok_or(ApiError::forbidden())?;
    if vendor != u.id {
        return Err(ApiError::forbidden());
    }
    if !["paid", "shopping", "price_adjustment"].contains(&status.as_str())
        || v.available_quantity_milli < 0
        || v.available_quantity_milli > quantity
    {
        return Err(ApiError::bad("Invalid availability confirmation"));
    }
    sqlx::query("UPDATE order_items SET available_quantity_milli=? WHERE id=?")
        .bind(v.available_quantity_milli)
        .bind(&iid)
        .execute(&mut *tx)
        .await?;
    audit(
        &mut tx,
        &u.id,
        "vendor.confirm",
        &iid,
        json!({"available_quantity_milli":v.available_quantity_milli}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
pub async fn categories(State(s): State<App>) -> Result<Json<Vec<String>>> {
    Ok(Json(
        sqlx::query_scalar("SELECT name FROM categories ORDER BY name")
            .fetch_all(&s.db)
            .await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Category {
    name: String,
}
pub async fn add_category(
    State(s): State<App>,
    u: User,
    Json(v): Json<Category>,
) -> Result<Json<Value>> {
    u.require(&["admin"])?;
    let name = text(&v.name, 2, 80)?;
    let mut tx = s.db.begin().await?;
    sqlx::query("INSERT INTO categories VALUES(?)")
        .bind(&name)
        .execute(&mut *tx)
        .await?;
    audit(
        &mut tx,
        &u.id,
        "category.create",
        &name,
        json!({"name":name}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
