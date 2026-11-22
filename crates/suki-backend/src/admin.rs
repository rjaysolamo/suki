use crate::auth::User;
use crate::*;
use axum::extract::{Path, State};
pub use suki_shared::admin::{
    AuditEntry, Expense, OrderReport, PendingApplicant, Reports, Settings, UserUpdate,
};
use suki_shared::auth::AccountStatus;
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
) -> Result<Json<Acknowledged>> {
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
    Ok(Json(Acknowledged { ok: true }))
}
pub async fn pending_approvals(
    State(s): State<App>,
    u: User,
) -> Result<Json<Vec<PendingApplicant>>> {
    u.require(&["admin"])?;
    Ok(Json(
        sqlx::query_as(
            "SELECT u.id,u.name,u.email,u.phone,u.role,v.shop_name AS stall_name,u.created_at
         FROM users u LEFT JOIN vendor_profiles v ON v.user_id=u.id AND u.role='vendor'
         WHERE u.status='pending_approval' AND u.role IN ('vendor','courier')
         ORDER BY u.created_at,u.id",
        )
        .fetch_all(&s.db)
        .await?,
    ))
}

pub async fn users(State(s): State<App>, u: User) -> Result<Json<Vec<crate::auth::Account>>> {
    u.require(&["admin"])?;
    Ok(Json(
        sqlx::query_as(
            "SELECT id,name,email,phone,role,status FROM users ORDER BY created_at DESC LIMIT 500",
        )
        .fetch_all(&s.db)
        .await?,
    ))
}
pub async fn update_user(
    State(s): State<App>,
    u: User,
    Path(uid): Path<String>,
    Json(v): Json<UserUpdate>,
) -> Result<Json<Acknowledged>> {
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
    let current: Option<String> =
        sqlx::query_scalar("SELECT status FROM users WHERE id=? FOR UPDATE")
            .bind(&uid)
            .fetch_optional(&mut *tx)
            .await?;
    let current = current.ok_or_else(|| ApiError::bad("Account not found"))?;
    if matches!(current.as_str(), "pending_approval" | "rejected") {
        return Err(ApiError::conflict(
            "Onboarding accounts require an approval decision",
        ));
    }
    if v.status == "pending" && matches!(v.role.as_str(), "vendor" | "courier") {
        return Err(ApiError::bad(
            "Use onboarding to review vendor and rider applications",
        ));
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
    sqlx::query("DELETE FROM user_sessions WHERE user_id=?")
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
    Ok(Json(Acknowledged { ok: true }))
}
pub async fn approve(
    State(s): State<App>,
    u: User,
    Path(uid): Path<String>,
) -> Result<Json<Acknowledged>> {
    decide(&s, &u, &uid, AccountStatus::Active, "user.approve").await
}

pub async fn reject(
    State(s): State<App>,
    u: User,
    Path(uid): Path<String>,
) -> Result<Json<Acknowledged>> {
    decide(&s, &u, &uid, AccountStatus::Rejected, "user.reject").await
}

async fn decide(
    s: &App,
    admin: &User,
    uid: &str,
    decision: AccountStatus,
    action: &str,
) -> Result<Json<Acknowledged>> {
    admin.require(&["admin"])?;
    let mut tx = s.db.begin().await?;
    let account: Option<(String, String, String)> =
        sqlx::query_as("SELECT id,role,status FROM users WHERE id=? FOR UPDATE")
            .bind(uid)
            .fetch_optional(&mut *tx)
            .await?;
    let (uid, role, status) =
        account.ok_or(ApiError(StatusCode::NOT_FOUND, "Account not found".into()))?;
    if !matches!(role.as_str(), "vendor" | "courier")
        || status != AccountStatus::PendingApproval.as_str()
    {
        return Err(ApiError::conflict("Account is not pending onboarding"));
    }
    let decision = decision.as_str();
    sqlx::query("UPDATE users SET status=? WHERE id=?")
        .bind(decision)
        .bind(&uid)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM user_sessions WHERE user_id=?")
        .bind(&uid)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM sessions WHERE user_id=?")
        .bind(&uid)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO onboarding_decisions(user_id,actor_id,decision,decided_at) VALUES (?,?,?,?)",
    )
    .bind(&uid)
    .bind(&admin.id)
    .bind(decision)
    .bind(now())
    .execute(&mut *tx)
    .await?;
    audit(
        &mut tx,
        &admin.id,
        action,
        &uid,
        json!({"role":role,"status":decision}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(Acknowledged { ok: true }))
}

pub async fn reports(State(s): State<App>, u: User) -> Result<Json<Reports>> {
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
        let (encrypted,fee,final_goods):(Vec<u8>,i64,Option<i64>)=sqlx::query_as("SELECT total_amount_centavos_encrypted,delivery_fee_centavos,final_goods_centavos FROM orders WHERE id=?").bind(&oid).fetch_one(&s.db).await?;
        let received:i64=sqlx::query_scalar("SELECT CAST(COALESCE(SUM(received_amount_centavos),0) AS SIGNED) FROM payments WHERE order_id=? AND status='paid'").bind(&oid).fetch_one(&s.db).await?;
        let refunds:i64=sqlx::query_scalar("SELECT CAST(COALESCE(SUM(amount_centavos),0) AS SIGNED) FROM refunds WHERE order_id=? AND status='processed'").bind(&oid).fetch_one(&s.db).await?;
        let expenses:i64=sqlx::query_scalar("SELECT CAST(COALESCE(SUM(amount_centavos),0) AS SIGNED) FROM expenses WHERE order_id=?").bind(&oid).fetch_one(&s.db).await?;
        reports.push(OrderReport {
            estimated_total_centavos: s.field_cipher.decode("orders.total", &oid, &encrypted)?,
            order_id: oid,
            received_centavos: received,
            delivery_fee_centavos: fee,
            purchase_cost_centavos: final_goods,
            refunds_centavos: refunds,
            expenses_centavos: expenses,
            margin_centavos: final_goods.map(|g| received - refunds - g - expenses),
        })
    }
    Ok(Json(Reports {
        counts,
        orders: reports,
    }))
}
pub async fn logs(State(s): State<App>, u: User) -> Result<Json<Vec<AuditEntry>>> {
    u.require(&["admin"])?;
    Ok(Json(sqlx::query_as("SELECT id,actor_id,action,entity_id,details,created_at FROM audit_logs ORDER BY id DESC LIMIT 500").fetch_all(&s.db).await?))
}
pub async fn expense(
    State(s): State<App>,
    u: User,
    Path(oid): Path<String>,
    Json(v): Json<Expense>,
) -> Result<Json<Created<String>>> {
    u.require(&["admin", "courier"])?;
    let mut tx = s.db.begin().await?;
    let o = crate::orders::locked(&mut tx, &u, &oid).await?;
    if u.role == "courier" && o.courier_id.as_deref() != Some(&u.id) {
        return Err(ApiError::forbidden());
    }
    if !(1..=100_000_000).contains(&v.amount_centavos) {
        return Err(ApiError::bad("Invalid expense amount"));
    }
    let desc = text(&v.description, 2, 500)?;
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
    integrity::commit(tx, &s.integrity_key, &o.id, &u.id).await?;
    Ok(Json(Created { id: eid }))
}
