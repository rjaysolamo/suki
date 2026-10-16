use crate::auth::User;
use crate::*;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
#[derive(Serialize, sqlx::FromRow)]
pub struct Product {
    pub id: i64,
    pub vendor_id: String,
    pub vendor: String,
    pub name: String,
    pub local_name: String,
    pub description: String,
    pub category: String,
    pub unit: String,
    pub price_centavos: i64,
    pub image: String,
    pub available: bool,
    pub updated_at: i64,
}
pub async fn list(State(s): State<App>) -> Result<Json<Vec<Product>>> {
    Ok(Json(
        sqlx::query_as("SELECT p.*,v.shop_name AS vendor FROM products p JOIN vendor_profiles v ON v.user_id=p.vendor_id JOIN users u ON u.id=p.vendor_id WHERE u.status='active' ORDER BY p.id")
            .fetch_all(&s.db)
            .await?,
    ))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductInput {
    pub name: String,
    pub local_name: String,
    pub description: String,
    pub category: String,
    pub unit: String,
    pub price_centavos: i64,
    pub image: String,
    pub available: bool,
}
impl ProductInput {
    fn validate(&self) -> Result<()> {
        text(&self.name, 2, 120)?;
        text(&self.local_name, 0, 100)?;
        text(&self.description, 0, 4000)?;
        text(&self.category, 2, 80)?;
        text(&self.unit, 1, 30)?;
        if self.price_centavos <= 0 || self.price_centavos > 100_000_000 {
            return Err(ApiError::bad("Invalid price"));
        }
        if !self.image.is_empty()
            && (!self.image.starts_with("/images/")
                || self.image.contains("..")
                || self.image.len() > 200)
        {
            return Err(ApiError::bad("Use a local /images/ path"));
        }
        Ok(())
    }
}
pub async fn create(
    State(s): State<App>,
    u: User,
    Json(p): Json<ProductInput>,
) -> Result<Json<Value>> {
    u.require(&["vendor"])?;
    p.validate()?;
    let mut tx = s.db.begin().await?;
    let category: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories WHERE name=?")
        .bind(&p.category)
        .fetch_one(&mut *tx)
        .await?;
    if category == 0 {
        return Err(ApiError::bad("Unknown category"));
    }
    let row=sqlx::query("INSERT INTO products(vendor_id,name,local_name,description,category,unit,price_centavos,image,available,updated_at) VALUES (?,?,?,?,?,?,?,?,?,?)").bind(&u.id).bind(&p.name).bind(&p.local_name).bind(&p.description).bind(&p.category).bind(&p.unit).bind(p.price_centavos).bind(&p.image).bind(p.available).bind(now()).execute(&mut *tx).await?;
    let pid = row.last_insert_id();
    audit(
        &mut tx,
        &u.id,
        "product.create",
        &pid.to_string(),
        json!({"price_centavos":p.price_centavos}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"id":pid})))
}
pub async fn update(
    State(s): State<App>,
    u: User,
    Path(pid): Path<i64>,
    Json(p): Json<ProductInput>,
) -> Result<Json<Value>> {
    u.require(&["vendor"])?;
    p.validate()?;
    let mut tx = s.db.begin().await?;
    let owner: Option<String> =
        sqlx::query_scalar("SELECT vendor_id FROM products WHERE id=? FOR UPDATE")
            .bind(pid)
            .fetch_optional(&mut *tx)
            .await?;
    if owner.as_deref() != Some(&u.id) {
        return Err(ApiError::forbidden());
    }
    let category: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM categories WHERE name=?")
        .bind(&p.category)
        .fetch_one(&mut *tx)
        .await?;
    if category == 0 {
        return Err(ApiError::bad("Unknown category"));
    }
    sqlx::query("UPDATE products SET name=?,local_name=?,description=?,category=?,unit=?,price_centavos=?,image=?,available=?,updated_at=? WHERE id=?").bind(&p.name).bind(&p.local_name).bind(&p.description).bind(&p.category).bind(&p.unit).bind(p.price_centavos).bind(&p.image).bind(p.available).bind(now()).bind(pid).execute(&mut *tx).await?;
    audit(
        &mut tx,
        &u.id,
        "product.update",
        &pid.to_string(),
        json!({"price_centavos":p.price_centavos,"available":p.available}),
    )
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
