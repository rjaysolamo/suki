use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tower::ServiceExt;

// Run only against a disposable database: migrations create schema and fixtures.
#[tokio::test]
#[ignore = "requires SUKI_TEST_DATABASE_URL pointing to a disposable database"]
async fn migration_enforces_identity_roles_and_application_constraints()
-> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("SUKI_TEST_DATABASE_URL")?;
    let mut db = PgConnection::connect(&url).await?;
    let database: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&mut db)
        .await?;
    if !database.ends_with("_test") {
        return Err("test database name must end with _test".into());
    }
    sqlx::migrate!("./migrations").run(&mut db).await?;
    let pool = PgPool::connect(&url).await?;
    let response = suki_foundation_api::router(pool.clone())
        .oneshot(
            Request::builder()
                .uri("/health/ready")
                .body(Body::empty())?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    pool.close().await;
    let mut tx = db.begin().await?;
    tx.execute("INSERT INTO users(id,google_issuer,google_subject,email,display_name) VALUES ('00000000-0000-0000-0000-000000000001','https://accounts.google.com','test-subject','test@example.invalid','Test User')").await?;
    let duplicate_identity = sqlx::query("INSERT INTO users(id,google_issuer,google_subject,email,display_name) VALUES ('00000000-0000-0000-0000-000000000002','https://accounts.google.com','test-subject','another@example.invalid','Other')").execute(&mut *tx).await;
    assert!(duplicate_identity.is_err());
    // PostgreSQL aborts the transaction after a constraint violation; start fresh.
    tx.rollback().await?;
    let mut tx = db.begin().await?;
    tx.execute("INSERT INTO users(id,google_issuer,google_subject,email,display_name) VALUES ('00000000-0000-0000-0000-000000000001','https://accounts.google.com','test-subject','test@example.invalid','Test User')").await?;
    let invalid_role = sqlx::query("INSERT INTO role_grants(user_id,role) VALUES ('00000000-0000-0000-0000-000000000001','superuser')").execute(&mut *tx).await;
    assert!(invalid_role.is_err());
    tx.rollback().await?;
    let mut tx = db.begin().await?;
    tx.execute("INSERT INTO users(id,google_issuer,google_subject,email,display_name) VALUES ('00000000-0000-0000-0000-000000000001','https://accounts.google.com','test-subject','test@example.invalid','Test User')").await?;
    let invalid_app = sqlx::query("INSERT INTO applications(id,user_id,kind,full_name,contact_number) VALUES ('00000000-0000-0000-0000-000000000003','00000000-0000-0000-0000-000000000001','courier','Test User','1234567')").execute(&mut *tx).await;
    assert!(invalid_app.is_err());
    tx.rollback().await?;
    Ok(())
}
