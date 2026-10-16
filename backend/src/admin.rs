use crate::auth::User;
use crate::*;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, sqlx::FromRow, Clone)]
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
pub async fn load(db: &MySqlPool) -> Result<Settings> {
    Ok(sqlx::query_as("SELECT * FROM settings WHERE id=1")
        .fetch_one(db)
        .await?)
}
pub async fn settings(State(s): State<App>) -> Result<Json<Settings>> {
    Ok(Json(load(&s.db).await?))
}
pub async fn update_settings(
    State(s): State<App>,
    u: User,
    Json(v): Json<Settings>,
) -> Result<Json<Value>> {
    u.require(&["admin"])?;
    coordinates(v.pickup_lat, v.pickup_lng)?;
    coordinates(v.south, v.west)?;
    coordinates(v.north, v.east)?;
    if v.south >= v.north
        || v.west >= v.east
        || v.base_fee_centavos < 0
        || v.base_fee_centavos > 100_000
        || v.per_km_centavos < 0
        || v.per_km_centavos > 10_000
        || !(1..=1_000_000).contains(&v.minimum_centavos)
        || !(100..=100_000).contains(&v.max_distance_meters)
        || !(1..=365).contains(&v.chat_retention_days)
    {
        return Err(ApiError::bad("Invalid delivery settings"));
    }
    if !v.gcash_qr_url.is_empty()
        && (!v.gcash_qr_url.starts_with("/images/")
            || v.gcash_qr_url.contains("..")
            || v.gcash_qr_url.len() > 500)
    {
        return Err(ApiError::bad("GCash QR must be a local /images/ path"));
    }
    let mut tx = s.db.begin().await?;
    sqlx::query("UPDATE settings SET base_fee_centavos=?,per_km_centavos=?,minimum_centavos=?,pickup_lat=?,pickup_lng=?,south=?,north=?,west=?,east=?,max_distance_meters=?,gcash_qr_url=?,chat_retention_days=? WHERE id=1").bind(v.base_fee_centavos).bind(v.per_km_centavos).bind(v.minimum_centavos).bind(v.pickup_lat).bind(v.pickup_lng).bind(v.south).bind(v.north).bind(v.west).bind(v.east).bind(v.max_distance_meters).bind(&v.gcash_qr_url).bind(v.chat_retention_days).execute(&mut *tx).await?;
    audit(&mut tx, &u.id, "settings.update", "1", json!(v)).await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
pub async fn users(State(s): State<App>, u: User) -> Result<Json<Vec<User>>> {
    u.require(&["admin"])?;
    Ok(Json(
        sqlx::query_as(
            "SELECT id,name,email,phone,role,status FROM users ORDER BY created_at DESC LIMIT 500",
        )
        .fetch_all(&s.db)
        .await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserUpdate {
    role: String,
    status: String,
}
pub async fn update_user(
    State(s): State<App>,
    u: User,
    Path(uid): Path<String>,
    Json(v): Json<UserUpdate>,
) -> Result<Json<Value>> {
    u.require(&["admin"])?;
    if !["customer", "vendor", "courier", "admin"].contains(&v.role.as_str())
        || !["pending", "active", "suspended", "deactivated"].contains(&v.status.as_str())
        || u.id == uid
    {
        return Err(ApiError::bad(
            "Invalid account update; administrators cannot change their own access",
        ));
    }
    let mut tx = s.db.begin().await?;
    let exists: Option<String> = sqlx::query_scalar("SELECT role FROM users WHERE id=? FOR UPDATE")
        .bind(&uid)
        .fetch_optional(&mut *tx)
        .await?;
    if exists.is_none() {
        return Err(ApiError::bad("Account not found"));
    }
    if v.role == "vendor" {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vendor_profiles WHERE user_id=?")
            .bind(&uid)
            .fetch_one(&mut *tx)
            .await?;
        if n == 0 {
            return Err(ApiError::bad(
                "Vendor profile required; register as vendor first",
            ));
        }
    }
    sqlx::query("UPDATE users SET role=?,status=? WHERE id=?")
        .bind(&v.role)
        .bind(&v.status)
        .bind(&uid)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE user_id=?")
        .bind(&uid)
        .execute(&mut *tx)
        .await?;
    audit(
        &mut tx,
        &u.id,
        "user.update",
        &uid,
        json!({"role":v.role,"status":v.status}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
pub async fn reports(State(s): State<App>, u: User) -> Result<Json<Value>> {
    u.require(&["admin"])?;
    let counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT status,COUNT(*) FROM orders GROUP BY status")
            .fetch_all(&s.db)
            .await?;
    let mut reports = Vec::new();
    let ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM orders ORDER BY created_at DESC LIMIT 500")
            .fetch_all(&s.db)
            .await?;
    for oid in ids {
        let (estimated,fee,final_goods):(i64,i64,Option<i64>)=sqlx::query_as("SELECT estimated_total_centavos,delivery_fee_centavos,final_goods_centavos FROM orders WHERE id=?").bind(&oid).fetch_one(&s.db).await?;
        let received:i64=sqlx::query_scalar("SELECT CAST(COALESCE(SUM(received_amount_centavos),0) AS SIGNED) FROM payments WHERE order_id=? AND status='paid'").bind(&oid).fetch_one(&s.db).await?;
        let refunds:i64=sqlx::query_scalar("SELECT CAST(COALESCE(SUM(amount_centavos),0) AS SIGNED) FROM refunds WHERE order_id=? AND status='processed'").bind(&oid).fetch_one(&s.db).await?;
        let expenses:i64=sqlx::query_scalar("SELECT CAST(COALESCE(SUM(amount_centavos),0) AS SIGNED) FROM expenses WHERE order_id=?").bind(&oid).fetch_one(&s.db).await?;
        reports.push(json!({"order_id":oid,"estimated_total_centavos":estimated,"received_centavos":received,"delivery_fee_centavos":fee,"purchase_cost_centavos":final_goods,"refunds_centavos":refunds,"expenses_centavos":expenses,"margin_centavos":final_goods.map(|g|received-refunds-g-expenses)}))
    }
    Ok(Json(json!({"counts":counts,"orders":reports})))
}
pub async fn logs(State(s): State<App>, u: User) -> Result<Json<Vec<Value>>> {
    u.require(&["admin"])?;
    let rows:Vec<(i64,String,String,String,String,i64)>=sqlx::query_as("SELECT id,actor_id,action,entity_id,details,created_at FROM audit_logs ORDER BY id DESC LIMIT 500").fetch_all(&s.db).await?;
    Ok(Json(rows.into_iter().map(|r|json!({"id":r.0,"actor_id":r.1,"action":r.2,"entity_id":r.3,"details":r.4,"created_at":r.5})).collect()))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expense {
    amount_centavos: i64,
    description: String,
}
pub async fn expense(
    State(s): State<App>,
    u: User,
    Path(oid): Path<String>,
    Json(v): Json<Expense>,
) -> Result<Json<Value>> {
    u.require(&["admin", "courier"])?;
    let o = crate::orders::access(&s, &u, &oid).await?;
    if u.role == "courier" && o.courier_id.as_deref() != Some(&u.id) {
        return Err(ApiError::forbidden());
    }
    if !(1..=100_000_000).contains(&v.amount_centavos) {
        return Err(ApiError::bad("Invalid expense amount"));
    }
    let desc = text(&v.description, 2, 500)?;
    let mut tx = s.db.begin().await?;
    let eid = id();
    sqlx::query("INSERT INTO expenses VALUES (?,?,?,?,?,?)")
        .bind(&eid)
        .bind(&oid)
        .bind(v.amount_centavos)
        .bind(desc)
        .bind(&u.id)
        .bind(now())
        .execute(&mut *tx)
        .await?;
    audit(
        &mut tx,
        &u.id,
        "expense.create",
        &eid,
        json!({"order_id":oid,"amount_centavos":v.amount_centavos}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"id":eid})))
}
