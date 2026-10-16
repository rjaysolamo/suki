use crate::auth::User;
use crate::*;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
#[derive(Serialize, sqlx::FromRow, Clone)]
pub struct Order {
    pub id: String,
    pub customer_id: String,
    pub courier_id: Option<String>,
    pub status: String,
    pub payment_status: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
    pub distance_meters: i64,
    pub delivery_fee_centavos: i64,
    pub goods_subtotal_centavos: i64,
    pub estimated_total_centavos: i64,
    pub final_goods_centavos: Option<i64>,
    pub authorized_goods_centavos: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub delivered_at: Option<i64>,
}
#[derive(Serialize, sqlx::FromRow)]
pub struct Item {
    pub id: String,
    pub product_id: i64,
    pub vendor_id: String,
    pub name: String,
    pub unit: String,
    pub unit_price_centavos: i64,
    pub quantity_milli: i64,
    pub estimated_cost_centavos: i64,
    pub actual_quantity_milli: Option<i64>,
    pub actual_unit_price_centavos: Option<i64>,
    pub actual_cost_centavos: Option<i64>,
    pub unavailable: bool,
}
pub async fn access(s: &App, u: &User, oid: &str) -> Result<Order> {
    let o: Order = sqlx::query_as("SELECT * FROM orders WHERE id=?")
        .bind(oid)
        .fetch_optional(&s.db)
        .await?
        .ok_or(ApiError(StatusCode::NOT_FOUND, "Order not found".into()))?;
    authorize(u, &o)?;
    Ok(o)
}
pub fn authorize(u: &User, o: &Order) -> Result<()> {
    if u.role == "admin"
        || (u.role == "customer" && o.customer_id == u.id)
        || (u.role == "courier" && o.courier_id.as_deref() == Some(&u.id))
    {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}
pub async fn locked(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    u: &User,
    oid: &str,
) -> Result<Order> {
    let o: Order = sqlx::query_as("SELECT * FROM orders WHERE id=? FOR UPDATE")
        .bind(oid)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(ApiError(StatusCode::NOT_FOUND, "Order not found".into()))?;
    authorize(u, &o)?;
    Ok(o)
}
pub async fn history(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    u: &User,
    oid: &str,
    status: &str,
) -> Result<()> {
    sqlx::query("UPDATE orders SET status=?,updated_at=? WHERE id=?")
        .bind(status)
        .bind(now())
        .bind(oid)
        .execute(&mut **tx)
        .await?;
    sqlx::query(
        "INSERT INTO order_status_history(order_id,status,actor_id,created_at) VALUES (?,?,?,?)",
    )
    .bind(oid)
    .bind(status)
    .bind(&u.id)
    .bind(now())
    .execute(&mut **tx)
    .await?;
    audit(tx, &u.id, "order.status", oid, json!({"status":status})).await?;
    Ok(())
}
pub async fn list(State(s): State<App>, u: User) -> Result<Json<Vec<Order>>> {
    u.require(&["customer", "courier", "admin"])?;
    Ok(Json(sqlx::query_as("SELECT * FROM orders WHERE (?='admin') OR (customer_id=? AND ?='customer') OR (courier_id=? AND ?='courier') ORDER BY created_at DESC LIMIT 100").bind(&u.role).bind(&u.id).bind(&u.role).bind(&u.id).bind(&u.role).fetch_all(&s.db).await?))
}
pub async fn detail(State(s): State<App>, u: User, Path(oid): Path<String>) -> Result<Json<Value>> {
    let o = access(&s, &u, &oid).await?;
    let items: Vec<Item> = sqlx::query_as("SELECT * FROM order_items WHERE order_id=? ORDER BY id")
        .bind(&oid)
        .fetch_all(&s.db)
        .await?;
    let payments: Vec<crate::payments::Payment> =
        sqlx::query_as("SELECT * FROM payments WHERE order_id=? ORDER BY created_at")
            .bind(&oid)
            .fetch_all(&s.db)
            .await?;
    let refunds: Vec<(String, i64, String)> =
        sqlx::query_as("SELECT id,amount_centavos,status FROM refunds WHERE order_id=?")
            .bind(&oid)
            .fetch_all(&s.db)
            .await?;
    let adjustments: Vec<(String, i64, String, String)> = sqlx::query_as(
        "SELECT id,proposed_goods_centavos,reason,status FROM adjustments WHERE order_id=?",
    )
    .bind(&oid)
    .fetch_all(&s.db)
    .await?;
    let substitutions:Vec<(String,String,String,i64,i64,String)>=sqlx::query_as("SELECT id,item_id,name,quantity_milli,unit_price_centavos,status FROM substitutions WHERE order_id=?").bind(&oid).fetch_all(&s.db).await?;
    let history: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT status,actor_id,created_at FROM order_status_history WHERE order_id=? ORDER BY id",
    )
    .bind(&oid)
    .fetch_all(&s.db)
    .await?;
    Ok(Json(
        json!({"order":o,"items":items,"payments":payments,"refunds":refunds,"adjustments":adjustments,"substitutions":substitutions,"history":history}),
    ))
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckoutItem {
    pub product_id: i64,
    pub quantity_milli: i64,
    pub expected_price_centavos: i64,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Checkout {
    pub quote_id: String,
    pub idempotency_key: String,
    pub items: Vec<CheckoutItem>,
}
#[derive(sqlx::FromRow)]
struct Quote {
    id: String,
    customer_id: String,
    address: String,
    lat: f64,
    lng: f64,
    distance_meters: i64,
    fee_centavos: i64,
    expires_at: i64,
    consumed: bool,
}
pub async fn create(
    State(s): State<App>,
    u: User,
    Json(mut v): Json<Checkout>,
) -> Result<Json<Value>> {
    u.require(&["customer"])?;
    text(&v.idempotency_key, 8, 100)?;
    if v.items.is_empty() || v.items.len() > 100 {
        return Err(ApiError::bad("Choose 1–100 products"));
    }
    v.items.sort_by_key(|i| i.product_id);
    if v.items
        .windows(2)
        .any(|w| w[0].product_id == w[1].product_id)
    {
        return Err(ApiError::bad("Duplicate product lines"));
    }
    let digest = hash(&serde_json::to_string(&v).unwrap());
    let mut tx = s.db.begin().await?;
    // Serialize a customer's order submissions before looking up the idempotency key.
    sqlx::query("SELECT id FROM users WHERE id=? FOR UPDATE")
        .bind(&u.id)
        .fetch_one(&mut *tx)
        .await?;
    let existing: Option<(String, String)> = sqlx::query_as(
        "SELECT id,request_hash FROM orders WHERE customer_id=? AND idempotency_key=?",
    )
    .bind(&u.id)
    .bind(&v.idempotency_key)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some((oid, h)) = existing {
        if h != digest {
            return Err(ApiError::conflict(
                "Idempotency key was used for different order details",
            ));
        }
        return Ok(Json(json!({"id":oid,"replayed":true})));
    }
    let q: Quote = sqlx::query_as("SELECT * FROM delivery_quotes WHERE id=? FOR UPDATE")
        .bind(&v.quote_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(ApiError::bad("Delivery quote not found"))?;
    if q.customer_id != u.id || q.expires_at < now() || q.consumed {
        return Err(ApiError::bad(
            "Delivery quote expired or unavailable; request a new quote",
        ));
    }
    let settings = crate::admin::load(&s.db).await?;
    crate::delivery::coverage(&settings, q.lat, q.lng)?;
    if q.distance_meters > settings.max_distance_meters {
        return Err(ApiError::bad("Delivery exceeds current service coverage"));
    }
    if settings.gcash_qr_url.is_empty() {
        return Err(ApiError(
            StatusCode::SERVICE_UNAVAILABLE,
            "The operator has not configured GCash payments".into(),
        ));
    }
    let mut snapshots = Vec::new();
    let mut subtotal = 0i64;
    for line in &v.items {
        if line.quantity_milli <= 0 {
            return Err(ApiError::bad("Quantity must be positive"));
        }
        let p: crate::catalog::Product =
            sqlx::query_as("SELECT p.*,v.shop_name AS vendor FROM products p JOIN vendor_profiles v ON v.user_id=p.vendor_id JOIN users u ON u.id=p.vendor_id WHERE u.status='active' AND p.id=? FOR UPDATE")
                .bind(line.product_id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or(ApiError::bad("A product is no longer available"))?;
        if !p.available || p.price_centavos != line.expected_price_centavos {
            return Err(ApiError::conflict(
                "A product's price or availability changed; refresh your basket",
            ));
        }
        let value = cost(p.price_centavos, line.quantity_milli)?;
        subtotal += value;
        snapshots.push((p, line.quantity_milli, value));
    }
    if subtotal < settings.minimum_centavos || subtotal > 100_000_000 {
        return Err(ApiError::bad(
            "Goods subtotal must meet the minimum and supported spending limit",
        ));
    }
    let oid = id();
    sqlx::query("INSERT INTO orders(id,customer_id,status,address,lat,lng,distance_meters,delivery_fee_centavos,goods_subtotal_centavos,estimated_total_centavos,authorized_goods_centavos,idempotency_key,request_hash,created_at,updated_at) VALUES (?,?,'awaiting_payment',?,?,?,?,?,?,?,?,?,?,?,?)").bind(&oid).bind(&u.id).bind(&q.address).bind(q.lat).bind(q.lng).bind(q.distance_meters).bind(q.fee_centavos).bind(subtotal).bind(subtotal+q.fee_centavos).bind(subtotal).bind(&v.idempotency_key).bind(digest).bind(now()).bind(now()).execute(&mut *tx).await?;
    for (p, qty, value) in snapshots {
        sqlx::query("INSERT INTO order_items(id,order_id,product_id,vendor_id,name,unit,unit_price_centavos,quantity_milli,estimated_cost_centavos) VALUES (?,?,?,?,?,?,?,?,?)").bind(id()).bind(&oid).bind(p.id).bind(p.vendor_id).bind(p.name).bind(p.unit).bind(p.price_centavos).bind(qty).bind(value).execute(&mut *tx).await?;
    }
    sqlx::query("UPDATE delivery_quotes SET consumed=TRUE WHERE id=?")
        .bind(q.id)
        .execute(&mut *tx)
        .await?;
    history(&mut tx, &u, &oid, "awaiting_payment").await?;
    tx.commit().await?;
    Ok(Json(json!({"id":oid,"replayed":false})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    courier_id: String,
}
pub async fn assign(
    State(s): State<App>,
    u: User,
    Path(oid): Path<String>,
    Json(v): Json<Assignment>,
) -> Result<Json<Value>> {
    u.require(&["admin"])?;
    let mut tx = s.db.begin().await?;
    let o = locked(&mut tx, &u, &oid).await?;
    if !["paid", "shopping", "ready_for_delivery", "price_adjustment"].contains(&o.status.as_str())
    {
        return Err(ApiError::conflict(
            "Only paid active orders can be assigned",
        ));
    }
    let valid: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE id=? AND role IN ('courier','admin') AND status='active'",
    )
    .bind(&v.courier_id)
    .fetch_one(&mut *tx)
    .await?;
    if valid != 1 {
        return Err(ApiError::bad("Courier is unavailable"));
    }
    sqlx::query("UPDATE orders SET courier_id=? WHERE id=?")
        .bind(&v.courier_id)
        .bind(&oid)
        .execute(&mut *tx)
        .await?;
    audit(
        &mut tx,
        &u.id,
        "order.assign",
        &oid,
        json!({"courier_id":v.courier_id}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub status: String,
}
pub fn allowed_transition(from: &str, to: &str) -> bool {
    matches!(
        (from, to),
        ("paid", "shopping")
            | ("ready_for_delivery", "out_for_delivery")
            | ("out_for_delivery", "delivered")
            | ("delivered", "completed")
    )
}
pub async fn transition(
    State(s): State<App>,
    u: User,
    Path(oid): Path<String>,
    Json(v): Json<Status>,
) -> Result<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let o = locked(&mut tx, &u, &oid).await?;
    if v.status == "canceled" {
        if ["completed", "delivered", "canceled"].contains(&o.status.as_str())
            || !(u.role == "admin"
                || u.role == "customer"
                    && ["awaiting_payment", "payment_verification", "paid"]
                        .contains(&o.status.as_str()))
        {
            return Err(ApiError::forbidden());
        }
        let pending: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM payments WHERE order_id=? AND status='under_verification'",
        )
        .bind(&oid)
        .fetch_one(&mut *tx)
        .await?;
        if pending > 0 {
            return Err(ApiError::conflict(
                "A payment is under verification; verify or reject it before canceling",
            ));
        }
        let received = crate::payments::received(&mut tx, &oid).await?;
        if received > 0 {
            refund_obligation(&mut tx, &oid, received).await?;
        }
    } else {
        u.require(&["courier", "admin"])?;
        if !allowed_transition(&o.status, &v.status) {
            return Err(ApiError::conflict("Invalid order status transition"));
        }
        if !["paid", "partially_refunded"].contains(&o.payment_status.as_str()) {
            return Err(ApiError::conflict(
                "Payment must be verified before shopping or delivery",
            ));
        }
        if v.status == "out_for_delivery" && o.courier_id.is_none() {
            return Err(ApiError::conflict("Assign a courier before delivery"));
        }
    }
    history(&mut tx, &u, &oid, &v.status).await?;
    if ["delivered", "canceled"].contains(&v.status.as_str()) {
        sqlx::query("DELETE FROM delivery_locations WHERE order_id=?")
            .bind(&oid)
            .execute(&mut *tx)
            .await?;
        if v.status == "delivered" {
            sqlx::query("UPDATE orders SET delivered_at=? WHERE id=?")
                .bind(now())
                .bind(&oid)
                .execute(&mut *tx)
                .await?;
        }
    }
    tx.commit().await?;
    let _ = s
        .events
        .send((oid, json!({"type":"status","status":v.status})));
    Ok(Json(json!({"ok":true})))
}
fn shopping(u: &User, o: &Order) -> Result<()> {
    u.require(&["courier", "admin"])?;
    if o.status != "shopping" || o.payment_status != "paid" {
        return Err(ApiError::conflict("Order must be paid and shopping"));
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Purchase {
    actual_quantity_milli: i64,
    actual_unit_price_centavos: i64,
    unavailable: bool,
}
pub async fn purchase(
    State(s): State<App>,
    u: User,
    Path((oid, iid)): Path<(String, String)>,
    Json(v): Json<Purchase>,
) -> Result<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let o = locked(&mut tx, &u, &oid).await?;
    shopping(&u, &o)?;
    if v.unavailable && (v.actual_quantity_milli != 0 || v.actual_unit_price_centavos != 0) {
        return Err(ApiError::bad(
            "Unavailable items must have zero actual quantity and price",
        ));
    }
    if !v.unavailable && v.actual_quantity_milli <= 0 {
        return Err(ApiError::bad("Purchased quantity must be positive"));
    }
    let value = cost(v.actual_unit_price_centavos, v.actual_quantity_milli)?;
    let item: Option<Item> = sqlx::query_as("SELECT * FROM order_items WHERE id=? AND order_id=?")
        .bind(&iid)
        .bind(&oid)
        .fetch_optional(&mut *tx)
        .await?;
    if item.is_none() {
        return Err(ApiError::bad("Item not found"));
    }
    let proposed:Option<(i64,i64,String)>=sqlx::query_as("SELECT quantity_milli,unit_price_centavos,status FROM substitutions WHERE item_id=? ORDER BY id DESC LIMIT 1").bind(&iid).fetch_optional(&mut *tx).await?;
    if let Some((qty, price, status)) = proposed
        && (status == "pending"
            || status == "rejected" && !v.unavailable
            || status == "approved"
                && (v.actual_quantity_milli > qty || v.actual_unit_price_centavos > price))
    {
        return Err(ApiError::conflict(
            "Substitution requires customer approval for the purchased quantity and price",
        ));
    }
    let recorded:i64=sqlx::query_scalar("SELECT CAST(COALESCE(SUM(actual_cost_centavos),0) AS SIGNED) FROM order_items WHERE order_id=? AND id<>?").bind(&oid).bind(&iid).fetch_one(&mut *tx).await?;
    if recorded + value > o.authorized_goods_centavos {
        return Err(ApiError::conflict(
            "Customer spending approval is required before recording this purchase",
        ));
    }
    sqlx::query("UPDATE order_items SET actual_quantity_milli=?,actual_unit_price_centavos=?,actual_cost_centavos=?,unavailable=? WHERE id=? AND order_id=?").bind(v.actual_quantity_milli).bind(v.actual_unit_price_centavos).bind(value).bind(v.unavailable).bind(&iid).bind(&oid).execute(&mut *tx).await?;
    audit(&mut tx,&u.id,"purchase.record",&iid,json!({"quantity_milli":v.actual_quantity_milli,"unit_price_centavos":v.actual_unit_price_centavos,"cost_centavos":value})).await?;
    tx.commit().await?;
    Ok(Json(json!({"cost_centavos":value})))
}
pub async fn refund_obligation(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    oid: &str,
    total_due: i64,
) -> Result<()> {
    let existing: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(amount_centavos),0) AS SIGNED) FROM refunds WHERE order_id=?",
    )
    .bind(oid)
    .fetch_one(&mut **tx)
    .await?;
    if total_due > existing {
        sqlx::query("INSERT INTO refunds(id,order_id,amount_centavos,created_at) VALUES (?,?,?,?)")
            .bind(id())
            .bind(oid)
            .bind(total_due - existing)
            .bind(now())
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}
pub async fn reconcile(
    State(s): State<App>,
    u: User,
    Path(oid): Path<String>,
) -> Result<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let o = locked(&mut tx, &u, &oid).await?;
    shopping(&u, &o)?;
    let items: Vec<Item> = sqlx::query_as("SELECT * FROM order_items WHERE order_id=?")
        .bind(&oid)
        .fetch_all(&mut *tx)
        .await?;
    if items.iter().any(|i| i.actual_cost_centavos.is_none()) {
        return Err(ApiError::conflict(
            "Record every purchased or unavailable item first",
        ));
    }
    let total: i64 = items
        .iter()
        .map(|i| i.actual_cost_centavos.unwrap_or(0))
        .sum();
    if total > 100_000_000 {
        return Err(ApiError::bad("Purchase exceeds spending limit"));
    }
    let paid = crate::payments::received(&mut tx, &oid).await?;
    sqlx::query("UPDATE orders SET final_goods_centavos=? WHERE id=?")
        .bind(total)
        .bind(&oid)
        .execute(&mut *tx)
        .await?;
    let status = if total > o.authorized_goods_centavos {
        sqlx::query("INSERT INTO adjustments(id,order_id,proposed_goods_centavos,reason) VALUES (?,?,?,'Actual quantity or purchase price changed')").bind(id()).bind(&oid).bind(total).execute(&mut *tx).await?;
        "price_adjustment"
    } else if paid < total + o.delivery_fee_centavos {
        "price_adjustment"
    } else {
        refund_obligation(
            &mut tx,
            &oid,
            (paid - total - o.delivery_fee_centavos).max(0),
        )
        .await?;
        "ready_for_delivery"
    };
    history(&mut tx, &u, &oid, status).await?;
    audit(
        &mut tx,
        &u.id,
        "order.reconcile",
        &oid,
        json!({"actual_goods_centavos":total}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(
        json!({"status":status,"final_goods_centavos":total,"difference_centavos":total-o.goods_subtotal_centavos}),
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    approved: bool,
}
pub async fn decide_adjustment(
    State(s): State<App>,
    u: User,
    Path((oid, aid)): Path<(String, String)>,
    Json(v): Json<Decision>,
) -> Result<Json<Value>> {
    u.require(&["customer"])?;
    let mut tx = s.db.begin().await?;
    let o = locked(&mut tx, &u, &oid).await?;
    if o.status != "price_adjustment" {
        return Err(ApiError::conflict(
            "No price adjustment is awaiting approval",
        ));
    }
    let total:Option<i64>=sqlx::query_scalar("SELECT proposed_goods_centavos FROM adjustments WHERE id=? AND order_id=? AND status='pending'").bind(&aid).bind(&oid).fetch_optional(&mut *tx).await?;
    let total = total.ok_or(ApiError::conflict(
        "Adjustment already decided or not found",
    ))?;
    sqlx::query("UPDATE adjustments SET status=?,customer_decided_at=? WHERE id=?")
        .bind(if v.approved { "approved" } else { "rejected" })
        .bind(now())
        .bind(aid)
        .execute(&mut *tx)
        .await?;
    if v.approved {
        sqlx::query("UPDATE orders SET authorized_goods_centavos=? WHERE id=?")
            .bind(total)
            .bind(&oid)
            .execute(&mut *tx)
            .await?;
        if crate::payments::received(&mut tx, &oid).await? >= total + o.delivery_fee_centavos {
            history(&mut tx, &u, &oid, "shopping").await?;
        }
    }
    audit(
        &mut tx,
        &u.id,
        "adjustment.decide",
        &oid,
        json!({"approved":v.approved}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Substitution {
    item_id: String,
    name: String,
    unit: String,
    quantity_milli: i64,
    unit_price_centavos: i64,
}
pub async fn propose_substitution(
    State(s): State<App>,
    u: User,
    Path(oid): Path<String>,
    Json(v): Json<Substitution>,
) -> Result<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let o = locked(&mut tx, &u, &oid).await?;
    shopping(&u, &o)?;
    text(&v.name, 2, 120)?;
    text(&v.unit, 1, 30)?;
    if v.quantity_milli <= 0 {
        return Err(ApiError::bad("Quantity must be positive"));
    }
    cost(v.unit_price_centavos, v.quantity_milli)?;
    let existing: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM substitutions WHERE item_id=?")
        .bind(&v.item_id)
        .fetch_one(&mut *tx)
        .await?;
    if existing > 0 {
        return Err(ApiError::conflict(
            "An item already has a substitution proposal",
        ));
    }
    let item: Option<Item> = sqlx::query_as("SELECT * FROM order_items WHERE id=? AND order_id=?")
        .bind(&v.item_id)
        .bind(&oid)
        .fetch_optional(&mut *tx)
        .await?;
    if item.is_none() || item.unwrap().actual_cost_centavos.is_some() {
        return Err(ApiError::conflict("Item not found or already purchased"));
    }
    let sid = id();
    sqlx::query("INSERT INTO substitutions(id,order_id,item_id,name,unit,quantity_milli,unit_price_centavos) VALUES(?,?,?,?,?,?,?)").bind(&sid).bind(&oid).bind(&v.item_id).bind(&v.name).bind(&v.unit).bind(v.quantity_milli).bind(v.unit_price_centavos).execute(&mut *tx).await?;
    audit(
        &mut tx,
        &u.id,
        "substitution.propose",
        &sid,
        json!({"name":v.name,"order_id":oid}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"id":sid})))
}
pub async fn decide_substitution(
    State(s): State<App>,
    u: User,
    Path((oid, sid)): Path<(String, String)>,
    Json(v): Json<Decision>,
) -> Result<Json<Value>> {
    u.require(&["customer"])?;
    let mut tx = s.db.begin().await?;
    let o = locked(&mut tx, &u, &oid).await?;
    if o.status != "shopping" {
        return Err(ApiError::conflict("Order is not shopping"));
    }
    let changed=sqlx::query("UPDATE substitutions SET status=?,decided_at=? WHERE id=? AND order_id=? AND status='pending'").bind(if v.approved{"approved"}else{"rejected"}).bind(now()).bind(&sid).bind(&oid).execute(&mut *tx).await?.rows_affected();
    if changed != 1 {
        return Err(ApiError::conflict(
            "Substitution is already decided or not found",
        ));
    }
    audit(
        &mut tx,
        &u.id,
        "substitution.decide",
        &sid,
        json!({"approved":v.approved}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn status_graph() {
        assert!(allowed_transition("paid", "shopping"));
        assert!(!allowed_transition("awaiting_payment", "shopping"));
        assert!(!allowed_transition("shopping", "delivered"));
        assert!(!allowed_transition("completed", "shopping"));
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdjustmentProposal {
    proposed_goods_centavos: i64,
    reason: String,
}
pub async fn propose_adjustment(
    State(s): State<App>,
    u: User,
    Path(oid): Path<String>,
    Json(v): Json<AdjustmentProposal>,
) -> Result<Json<Value>> {
    let mut tx = s.db.begin().await?;
    let o = locked(&mut tx, &u, &oid).await?;
    shopping(&u, &o)?;
    let reason = text(&v.reason, 5, 1000)?;
    if v.proposed_goods_centavos <= o.authorized_goods_centavos
        || v.proposed_goods_centavos > 100_000_000
    {
        return Err(ApiError::bad(
            "Propose a higher goods spending limit within the supported maximum",
        ));
    }
    let aid = id();
    sqlx::query(
        "INSERT INTO adjustments(id,order_id,proposed_goods_centavos,reason) VALUES(?,?,?,?)",
    )
    .bind(&aid)
    .bind(&oid)
    .bind(v.proposed_goods_centavos)
    .bind(&reason)
    .execute(&mut *tx)
    .await?;
    history(&mut tx, &u, &oid, "price_adjustment").await?;
    audit(
        &mut tx,
        &u.id,
        "adjustment.propose",
        &aid,
        json!({"goods_centavos":v.proposed_goods_centavos,"reason":reason}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"id":aid})))
}
