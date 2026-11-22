//! Real MariaDB integration test. Run only against a dedicated test database.
use axum::{
    Json,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::sync::Arc;
use suki_api::{App, AppState, id, now};
use tower::ServiceExt;

async fn call(
    app: &axum::Router,
    method: &str,
    path: &str,
    cookie: &str,
    body: Value,
) -> (StatusCode, Value, String) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .header("x-suki-request", "1")
        .header("origin", "http://localhost:3000")
        .header("cookie", cookie)
        .body(if body.is_null() {
            Body::empty()
        } else {
            Body::from(body.to_string())
        })
        .unwrap();
    static NEXT_IP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
    let ip = std::net::Ipv4Addr::from(NEXT_IP.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    req.extensions_mut()
        .insert(axum::extract::ConnectInfo(std::net::SocketAddr::from((
            ip, 1234,
        ))));
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let cookie = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok()?.split(';').next())
        .collect::<Vec<_>>()
        .join("; ");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| json!({"raw":String::from_utf8_lossy(&bytes)}));
    (status, value, cookie)
}
async fn ok(app: &axum::Router, method: &str, path: &str, cookie: &str, body: Value) -> Value {
    let (status, value, _) = call(app, method, path, cookie, body).await;
    assert!(status.is_success(), "{method} {path}: {status}: {value}");
    value
}
async fn account(app: &axum::Router, role: &str) -> (String, String) {
    let email = format!("{}@test.invalid", id());
    let v=ok(app,"POST","/api/auth/register","",json!({"name":"Synthetic Test User","email":email,"phone":"09000000000","password":"Synthetic-test-password-42","role":role,"shop_name":"Test Fresh Picks"})).await;
    (v["id"].as_str().unwrap().into(), email)
}
async fn login(app: &axum::Router, email: &str) -> String {
    let (status, v, c) = call(
        app,
        "POST",
        "/api/auth/login",
        "",
        json!({"email":email,"password":"Synthetic-test-password-42"}),
    )
    .await;
    assert_eq!(status, 200, "{v}");
    assert!(!c.is_empty());
    c
}
async fn approve(app: &axum::Router, admin: &str, uid: &str) {
    ok(
        app,
        "POST",
        &format!("/api/admin/users/{uid}/approve"),
        admin,
        Value::Null,
    )
    .await;
}
fn product(price: i64) -> Value {
    json!({"name":"Test Mangoes","local_name":"Mangga","description":"Synthetic fixture","category":"Fruits","unit":"kg","price_centavos":price,"image":"/images/mangoes.jpg","available":true})
}
async fn quote(app: &axum::Router, cookie: &str) -> Value {
    ok(
        app,
        "POST",
        "/api/delivery/quote",
        cookie,
        json!({"address":"123 Synthetic Street, Matina, Davao City","lat":7.05,"lng":125.59}),
    )
    .await
}

async fn device_sessions(
    app: &axum::Router,
    state: &AppState,
    owner: &str,
    email: &str,
    stranger: &str,
) {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let pos = owner
        .split(';')
        .find_map(|v| v.trim().strip_prefix("suki_session="))
        .unwrap();
    let payload = URL_SAFE_NO_PAD
        .decode(pos.split('.').nth(1).unwrap())
        .unwrap();
    let claims: Value = serde_json::from_slice(&payload).unwrap();
    assert_eq!(
        claims["exp"].as_i64().unwrap() - claims["iat"].as_i64().unwrap(),
        900
    );

    let refresh_only = |cookie: &str| {
        cookie
            .split(';')
            .find(|v| v.trim().starts_with("suki_refresh="))
            .unwrap()
            .trim()
            .to_owned()
    };
    let tablet = login(app, email).await;
    let old = refresh_only(&tablet);
    let listing = ok(app, "GET", "/api/auth/sessions", owner, Value::Null).await;
    assert_eq!(listing.as_array().unwrap().len(), 2);
    let other = listing
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["current"] == false)
        .unwrap();
    assert_eq!(
        other["expires_at"].as_i64().unwrap() - other["created_at"].as_i64().unwrap(),
        30 * 86_400
    );
    let id = other["id"].as_str().unwrap();
    let path = format!("/api/auth/sessions/{id}");
    assert_eq!(
        call(app, "DELETE", &path, stranger, Value::Null).await.0,
        404
    );
    let (status, _, rotated) = call(app, "POST", "/api/auth/refresh", &old, Value::Null).await;
    assert_eq!(status, 200);
    assert_ne!(refresh_only(&rotated), old);
    assert_ne!(rotated.split(';').next(), tablet.split(';').next());
    assert_eq!(
        call(app, "POST", "/api/auth/refresh", &old, Value::Null)
            .await
            .0,
        409
    );
    let raw = refresh_only(&rotated);
    let stored: String =
        sqlx::query_scalar("SELECT refresh_token_hash FROM user_sessions WHERE id=?")
            .bind(id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(
        stored,
        suki_api::hash(raw.trim_start_matches("suki_refresh="))
    );
    sqlx::query("UPDATE spent_refresh_tokens SET used_at=? WHERE session_id=?")
        .bind(now() - 10)
        .bind(id)
        .execute(&state.db)
        .await
        .unwrap();
    assert_eq!(
        call(app, "POST", "/api/auth/refresh", &old, Value::Null)
            .await
            .0,
        401
    );
    assert_eq!(
        call(app, "GET", "/api/auth/me", &rotated, Value::Null)
            .await
            .0,
        401
    );
    assert_eq!(
        call(app, "POST", "/api/auth/refresh", &raw, Value::Null)
            .await
            .0,
        401
    );
    assert_eq!(
        call(app, "GET", "/api/auth/me", owner, Value::Null).await.0,
        200
    );

    let tablet = login(app, email).await;
    let listing = ok(app, "GET", "/api/auth/sessions", owner, Value::Null).await;
    let id = listing
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["current"] == false)
        .unwrap()["id"]
        .as_str()
        .unwrap();
    ok(
        app,
        "DELETE",
        &format!("/api/auth/sessions/{id}"),
        owner,
        Value::Null,
    )
    .await;
    assert_eq!(
        call(app, "GET", "/api/auth/me", &tablet, Value::Null)
            .await
            .0,
        401
    );
    assert_eq!(
        call(
            app,
            "POST",
            "/api/auth/refresh",
            &refresh_only(&tablet),
            Value::Null
        )
        .await
        .0,
        401
    );

    let tablet = login(app, email).await;
    ok(
        app,
        "POST",
        "/api/auth/logout",
        &refresh_only(&tablet),
        Value::Null,
    )
    .await;
    assert_eq!(
        call(app, "GET", "/api/auth/me", &tablet, Value::Null)
            .await
            .0,
        401
    );
}

const LEGACY_ADDRESS: &str = "123 Synthetic Legacy Address, Davao City";

async fn seed_legacy(
    db: &sqlx::MySqlPool,
    key: &suki_shared::crypto::IntegrityKey,
) -> (String, String) {
    use suki_shared::{
        crypto::{SNAPSHOT_AUTH, TRANSACTION_AUTH, hash_snapshot, hash_transaction},
        payments::PaymentMethod,
        snapshots::{SnapshotPayload, delivery_breakdown},
    };
    let customer = id();
    let order = id();
    let created = now();
    sqlx::query("INSERT INTO users (id,name,email,phone,password_hash,role,status,created_at) VALUES (?,'Legacy Test Customer',?,'','unused','customer','active',?)")
        .bind(&customer).bind(format!("{customer}@test.invalid")).bind(created).execute(db).await.unwrap();
    let quote = id();
    sqlx::query("INSERT INTO delivery_quotes(id,customer_id,address,lat,lng,distance_meters,fee_centavos,base_fee_centavos,per_km_centavos,expires_at) VALUES (?,?,?,7.05,125.59,1000,6350,6000,350,?)")
        .bind(&quote).bind(&customer).bind(LEGACY_ADDRESS).bind(created + 300).execute(db).await.unwrap();
    sqlx::query("INSERT INTO customer_addresses(id,customer_id,label,address,lat,lng) VALUES (?,?,'Home',?,7.05,125.59)")
        .bind(id()).bind(&customer).bind(LEGACY_ADDRESS).execute(db).await.unwrap();
    sqlx::query("INSERT INTO orders(id,customer_id,status,address,lat,lng,distance_meters,delivery_fee_centavos,goods_subtotal_centavos,estimated_total_centavos,authorized_goods_centavos,idempotency_key,request_hash,created_at,updated_at) VALUES (?,?,'awaiting_payment',?,7.05,125.59,1000,6350,50000,56350,50000,'legacy-test',?, ?, ?)")
        .bind(&order).bind(&customer).bind(LEGACY_ADDRESS).bind("0".repeat(64)).bind(created).bind(created).execute(db).await.unwrap();
    let payload = serde_json::to_vec(&SnapshotPayload {
        order_id: order.clone(),
        customer_id: customer.clone(),
        vendor_id: None,
        address: LEGACY_ADDRESS.into(),
        payment_method: PaymentMethod::Gcash,
        items: vec![],
        delivery: delivery_breakdown(quote, 1000, None, 6000, 350).unwrap(),
        goods_subtotal_centavos: 50000,
        total_centavos: 56350,
        created_at: created,
    })
    .unwrap();
    let hash = hash_snapshot(&payload);
    sqlx::query(
        "INSERT INTO order_snapshots (order_id,payload,snapshot_hash,signature) VALUES (?,?,?,?)",
    )
    .bind(&order)
    .bind(payload)
    .bind(&hash)
    .bind(key.sign(SNAPSHOT_AUTH, &hash))
    .execute(db)
    .await
    .unwrap();
    let mut previous = "genesis".to_owned();
    let entries: Vec<(i64, String, String, i64)> = sqlx::query_as(
        "SELECT id,event,payload,created_at FROM order_integrity WHERE order_id=? ORDER BY id",
    )
    .bind(&order)
    .fetch_all(db)
    .await
    .unwrap();
    for (id, event, payload, created) in entries {
        let tag = key.sign(
            TRANSACTION_AUTH,
            &hash_transaction(&order, id, &event, &customer, created, &payload, &previous),
        );
        sqlx::query("UPDATE order_integrity SET actor_id=?,tag=? WHERE id=?")
            .bind(&customer)
            .bind(&tag)
            .bind(id)
            .execute(db)
            .await
            .unwrap();
        previous = tag;
    }
    (order, hash)
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing at a dedicated MariaDB database"]
async fn full_order_and_authorization() {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL required");
    let db = sqlx::MySqlPool::connect(&url).await.unwrap();
    let name: String = sqlx::query_scalar("SELECT DATABASE()")
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(name.ends_with("_test"), "Test database must end in _test");
    let migrator = sqlx::migrate!("./migrations");
    let signing = suki_shared::crypto::IntegrityKey::from_hex(&"12".repeat(32)).unwrap();
    let upgraded: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM information_schema.TABLES WHERE TABLE_SCHEMA=DATABASE() AND TABLE_NAME='data_encryption_check')",
    ).fetch_one(&db).await.unwrap();
    let old = if upgraded {
        None
    } else {
        migrator.run_to(7, &db).await.unwrap();
        Some(seed_legacy(&db, &signing).await)
    };
    migrator.run(&db).await.unwrap();
    let field_cipher =
        Arc::new(suki_api::protected::FieldCipher::from_hex(&"ab".repeat(32)).unwrap());
    suki_api::protected::backfill(&db, &field_cipher)
        .await
        .unwrap();
    suki_api::protected::backfill(&db, &field_cipher)
        .await
        .unwrap();
    let wrong = suki_api::protected::FieldCipher::from_hex(&"bc".repeat(32)).unwrap();
    assert!(suki_api::protected::backfill(&db, &wrong).await.is_err());
    let fixture = axum::Router::new().route(
        "/route",
        axum::routing::post(|Json(v): Json<Value>| async move {
            assert_eq!(v["travelMode"], "DRIVE");
            assert!(v.pointer("/destination/location/latLng/latitude").is_some());
            Json(json!({"routes":[{"distanceMeters":1234,"duration":"480s"}]}))
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async { axum::serve(listener, fixture).await.unwrap() });
    // A small S3 wire fixture keeps CI independent of external credentials. An
    // optional endpoint allows the same flow to run against a local S3 emulator.
    let objects = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::<
        String,
        Vec<u8>,
    >::new()));
    let storage_fixture = axum::Router::new().route(
        "/{bucket}/{*key}",
        axum::routing::any({
            let objects = objects.clone();
            move |axum::extract::Path((_bucket, key)): axum::extract::Path<(String, String)>,
                  request: Request<Body>| {
                let objects = objects.clone();
                async move {
                    assert!(
                        request
                            .headers()
                            .get("authorization")
                            .and_then(|v| v.to_str().ok())
                            .is_some_and(|v| v.starts_with("AWS4-HMAC-SHA256"))
                    );
                    if request.method() == "PUT" {
                        let body = request
                            .into_body()
                            .collect()
                            .await
                            .unwrap()
                            .to_bytes()
                            .to_vec();
                        objects.lock().await.insert(key, body);
                        (StatusCode::OK, Vec::new())
                    } else if request.method() == "GET" {
                        match objects.lock().await.get(&key).cloned() {
                            Some(body) => (StatusCode::OK, body),
                            None => (StatusCode::NOT_FOUND, Vec::new()),
                        }
                    } else {
                        (StatusCode::METHOD_NOT_ALLOWED, Vec::new())
                    }
                }
            }
        }),
    );
    let storage_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let storage_address = storage_listener.local_addr().unwrap();
    let storage_task = tokio::spawn(async move {
        axum::serve(storage_listener, storage_fixture)
            .await
            .unwrap()
    });
    let endpoint =
        std::env::var("TEST_R2_ENDPOINT").unwrap_or_else(|_| format!("http://{storage_address}"));
    let r2 = suki_api::storage::R2::new(
        &endpoint,
        "emulate-default",
        &"https://media.example.invalid".parse().unwrap(),
        "AKIAIOSFODNN7EXAMPLE",
        "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
    )
    .unwrap();
    let admin_email = format!("{}@test.invalid", id());
    let s: App = Arc::new(AppState {
        db: db.clone(),
        rate_limits: Default::default(),
        admin_emails: suki_api::auth::AdminEmails::parse(&format!(
            "  {}  , second@test.invalid",
            admin_email.to_uppercase()
        ))
        .unwrap(),
        integrity_key: signing,
        field_cipher: field_cipher.clone(),
        session_key: Arc::new(suki_api::sessions::SessionKey::from_hex(&"21".repeat(32)).unwrap()),
        client: reqwest::Client::new(),
        secure_cookies: false,
        origin: "http://localhost:3000".into(),
        maps_key: "synthetic-test-key".into(),
        maps_url: format!("http://{address}/route"),
        legacy_quotes: true,
        r2: Some(r2),
        media_dir: std::env::temp_dir().join(format!("suki-media-{}", id())),
        events: tokio::sync::broadcast::channel(256).0,
    });
    let app = suki_api::router(s.clone());
    let uid = id();
    let phc = suki_api::auth::password_hash("Synthetic-test-password-42".into())
        .await
        .unwrap();
    sqlx::query("INSERT INTO users (id,name,email,phone,password_hash,role,status,created_at) VALUES (?,?,?,'',?,'admin','active',?)")
        .bind(&uid)
        .bind("Test Admin")
        .bind(&admin_email)
        .bind(phc)
        .bind(now())
        .execute(&db)
        .await
        .unwrap();
    let admin = login(&app, &admin_email).await;
    if let Some((order, hash)) = old {
        let detail = ok(
            &app,
            "GET",
            &format!("/api/orders/{order}"),
            &admin,
            Value::Null,
        )
        .await;
        assert_eq!(detail["order"]["address"], LEGACY_ADDRESS);
        assert_eq!(detail["order"]["estimated_total_centavos"], 56350);
        assert_eq!(detail["snapshot"]["snapshot_hash"], hash);
        let inspection = suki_api::router(Arc::new(AppState {
            rate_limits: Default::default(),
            ..(*s).clone()
        }));
        let report = ok(
            &inspection,
            "GET",
            &format!("/api/admin/orders/{order}/integrity"),
            &admin,
            Value::Null,
        )
        .await;
        assert_eq!(report["snapshot"], "verified");
        assert_eq!(report["transactions_verified"], 2);
        let protected: (bool,bool) = sqlx::query_as("SELECT address IS NULL AND lat IS NULL AND lng IS NULL AND estimated_total_centavos IS NULL, total_amount_centavos_encrypted IS NOT NULL FROM orders WHERE id=?")
            .bind(&order).fetch_one(&db).await.unwrap();
        assert_eq!(protected, (true, true));
        let plaintext: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM order_integrity WHERE order_id=? AND JSON_CONTAINS_PATH(payload,'one','$.estimated_total_centavos')")
            .bind(&order).fetch_one(&db).await.unwrap();
        assert_eq!(plaintext, 0);
    }
    let unknown_email = format!("{}@test.invalid", id());
    for email in [&admin_email, &unknown_email, &unknown_email] {
        assert_eq!(
            call(
                &app,
                "POST",
                "/api/auth/login",
                "",
                json!({"email":email,"password":"Invalid-test-password-42"}),
            )
            .await
            .0,
            401
        );
    }

    let (customer_id, customer_email) = account(&app, "customer").await;
    let customer = login(&app, &customer_email).await;
    let (_, other_email) = account(&app, "customer").await;
    let other = login(&app, &other_email).await;
    device_sessions(&app, &s, &customer, &customer_email, &other).await;
    let saved_address=ok(&app,"POST","/api/addresses",&customer,json!({"label":"Home","address":"123 Synthetic Address, Davao City","lat":7.05,"lng":125.59})).await;
    assert!(saved_address["id"].is_string());
    let protected: bool = sqlx::query_scalar("SELECT address IS NULL AND label IS NULL AND lat IS NULL AND lng IS NULL AND address_encrypted IS NOT NULL FROM customer_addresses WHERE id=?")
        .bind(saved_address["id"].as_str().unwrap()).fetch_one(&db).await.unwrap();
    assert!(protected);
    assert_eq!(
        ok(&app, "GET", "/api/addresses", &customer, Value::Null)
            .await
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        ok(&app, "GET", "/api/addresses", &other, Value::Null)
            .await
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let unsafe_request = Request::builder()
        .method("POST")
        .uri("/api/auth/logout")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(unsafe_request).await.unwrap().status(),
        403
    );
    let foreign_origin = Request::builder()
        .method("POST")
        .uri("/api/auth/logout")
        .header("x-suki-request", "1")
        .header("origin", "https://untrusted.invalid")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(foreign_origin).await.unwrap().status(),
        403
    );
    let (vendor_id, vendor_email) = account(&app, "vendor").await;
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/auth/login",
            "",
            json!({"email":vendor_email,"password":"Synthetic-test-password-42"})
        )
        .await
        .0,
        403
    );
    let queue = suki_api::router(Arc::new(AppState {
        rate_limits: Default::default(),
        ..(*s).clone()
    }));
    let pending_path = "/api/admin/pending-approvals";
    let pending_rows = ok(&queue, "GET", pending_path, &admin, Value::Null).await;
    let applicant = pending_rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == vendor_id)
        .unwrap();
    assert_eq!(applicant["email"], vendor_email);
    assert_eq!(applicant["name"], "Synthetic Test User");
    assert_eq!(applicant["phone"], "09000000000");
    assert_eq!(applicant["role"], "vendor");
    assert_eq!(applicant["stall_name"], "Test Fresh Picks");
    assert!(applicant["created_at"].as_i64().unwrap() > 0);
    assert!(
        !pending_rows
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["id"] == customer_id)
    );
    assert_eq!(
        call(&queue, "GET", "/admin", &admin, Value::Null).await.0,
        303
    );
    let approve_path = format!("/api/admin/users/{vendor_id}/approve");
    assert_eq!(
        call(&app, "POST", &approve_path, &customer, Value::Null)
            .await
            .0,
        403
    );
    assert_eq!(
        call(
            &app,
            "PATCH",
            &format!("/api/admin/users/{vendor_id}"),
            &admin,
            json!({"role":"vendor","status":"active"})
        )
        .await
        .0,
        409
    );
    approve(&app, &admin, &vendor_id).await;
    let vendor = login(&app, &vendor_email).await;
    assert!(ok(&app, "GET", "/api/auth/me", &vendor, Value::Null).await["approved_at"].is_i64());
    let (rejected_id, rejected_email) = account(&app, "rider").await;
    let pending_rows = ok(&queue, "GET", pending_path, &admin, Value::Null).await;
    let rows = pending_rows.as_array().unwrap();
    assert!(!rows.iter().any(|r| r["id"] == vendor_id));
    let rider = rows.iter().find(|r| r["id"] == rejected_id).unwrap();
    assert_eq!(rider["role"], "courier");
    assert_eq!(rider["stall_name"], Value::Null);
    let (pending, msg, cookie) = call(
        &app,
        "POST",
        "/api/auth/login",
        "",
        json!({"email":rejected_email,"password":"Synthetic-test-password-42"}),
    )
    .await;
    assert_eq!(pending, 403);
    assert_eq!(msg["error"], "Account pending admin approval.");
    assert!(cookie.is_empty());
    let onboarding = suki_api::router(Arc::new(AppState {
        rate_limits: Default::default(),
        ..(*s).clone()
    }));
    ok(
        &onboarding,
        "POST",
        &format!("/api/admin/users/{rejected_id}/reject"),
        &admin,
        Value::Null,
    )
    .await;
    assert_eq!(
        call(
            &onboarding,
            "POST",
            &format!("/api/admin/users/{rejected_id}/approve"),
            &admin,
            Value::Null
        )
        .await
        .0,
        409
    );
    let (status, msg, _) = call(
        &app,
        "POST",
        "/api/auth/login",
        "",
        json!({"email":rejected_email,"password":"Synthetic-test-password-42"}),
    )
    .await;
    assert_eq!(status, 403);
    assert_eq!(msg["error"], "Account application was rejected.");
    let pending_rows = ok(&queue, "GET", pending_path, &admin, Value::Null).await;
    assert!(
        !pending_rows
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["id"] == rejected_id)
    );
    for _ in 0..8 {
        assert_eq!(
            call(&onboarding, "GET", "/api/admin/users", &admin, Value::Null)
                .await
                .0,
            200
        );
    }
    assert_eq!(
        call(
            &onboarding,
            "POST",
            &format!("/api/admin/users/{rejected_id}/reject"),
            &admin,
            Value::Null
        )
        .await
        .0,
        429
    );
    assert_eq!(
        call(&app, "POST", "/api/products", &customer, product(16000))
            .await
            .0,
        403
    );
    let pid = ok(&app, "POST", "/api/products", &vendor, product(16000)).await["id"]
        .as_u64()
        .unwrap();
    let (vendor2_id, vendor2_email) = account(&app, "vendor").await;
    approve(&app, &admin, &vendor2_id).await;
    let vendor2 = login(&app, &vendor2_email).await;
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!("/api/products/{pid}"),
            &vendor2,
            product(1)
        )
        .await
        .0,
        403
    );
    // A stale availability action must never restore an older product price.
    ok(
        &app,
        "PUT",
        &format!("/api/products/{pid}"),
        &vendor,
        product(17000),
    )
    .await;
    for denied in [&vendor2, &customer] {
        assert_eq!(
            call(
                &app,
                "PATCH",
                &format!("/api/products/{pid}/availability"),
                denied,
                json!({"available":false})
            )
            .await
            .0,
            403
        );
    }
    assert_eq!(
        call(
            &app,
            "PATCH",
            &format!("/api/products/{pid}/availability"),
            &vendor,
            json!({"available":false,"price_centavos":16000})
        )
        .await
        .0,
        422
    );
    ok(
        &app,
        "PATCH",
        &format!("/api/products/{pid}/availability"),
        &vendor,
        json!({"available":false}),
    )
    .await;
    let (price, available): (i64, bool) =
        sqlx::query_as("SELECT price_centavos,available FROM products WHERE id=?")
            .bind(pid)
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!((price, available), (17000, false));
    ok(
        &app,
        "PUT",
        &format!("/api/products/{pid}"),
        &vendor,
        product(16000),
    )
    .await;

    let (courier_id, courier_email) = account(&app, "rider").await;
    approve(&app, &admin, &courier_id).await;
    let courier = login(&app, &courier_email).await;
    let blocked_admin = suki_api::router(Arc::new(AppState {
        admin_emails: suki_api::auth::AdminEmails::parse(&format!(
            "{customer_email},{vendor_email},{courier_email}"
        ))
        .unwrap(),
        ..(*s).clone()
    }));
    for path in [
        "/admin",
        "/admin/pending-approvals",
        pending_path,
        "/api/admin/users",
        "/api/admin/reports",
        "/api/admin/audit",
        "/api/admin/orders/not-an-order/integrity",
        "/api/admin/payments/pending",
    ] {
        assert_eq!(call(&app, "GET", path, "", Value::Null).await.0, 401);
        for cookie in [&customer, &vendor, &courier, &admin] {
            assert_eq!(
                call(&blocked_admin, "GET", path, cookie, Value::Null)
                    .await
                    .0,
                403
            );
        }
    }
    assert_eq!(
        call(&blocked_admin, "PUT", "/api/settings", &admin, Value::Null)
            .await
            .0,
        403
    );
    assert_eq!(
        call(&blocked_admin, "GET", "/api/auth/me", &admin, Value::Null)
            .await
            .0,
        403
    );
    let (status, _, cookie) = call(
        &blocked_admin,
        "POST",
        "/api/auth/login",
        "",
        json!({"email":admin_email,"password":"Synthetic-test-password-42"}),
    )
    .await;
    assert_eq!(status, 403);
    assert!(cookie.is_empty());
    assert_eq!(
        ok(&app, "GET", "/api/auth/me", &admin, Value::Null).await["admin_access"],
        true
    );
    assert_eq!(
        ok(&app, "GET", "/api/auth/me", &customer, Value::Null).await["admin_access"],
        false
    );

    // Exercise limits through the middleware, including spoofed proxy headers.
    let limited = suki_api::router(Arc::new(AppState {
        rate_limits: Default::default(),
        ..(*s).clone()
    }));
    let limited_call = |route: String, method: &'static str, cookie: String, ip: [u8; 4]| {
        let app = limited.clone();
        async move {
            let mut request = Request::builder()
                .uri(route)
                .method(method)
                .header("cookie", cookie)
                .header("x-suki-request", "1")
                .header("x-forwarded-for", "203.0.113.99")
                .body(Body::empty())
                .unwrap();
            request.extensions_mut().insert(axum::extract::ConnectInfo(
                std::net::SocketAddr::from((ip, 1234)),
            ));
            app.oneshot(request).await.unwrap().status()
        }
    };
    for _ in 0..5 {
        assert_eq!(
            limited_call("/api/auth/me".into(), "GET", String::new(), [127, 0, 0, 8]).await,
            StatusCode::UNAUTHORIZED
        );
    }
    assert_eq!(
        limited_call("/api/auth/me".into(), "GET", String::new(), [127, 0, 0, 8]).await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        limited_call("/api/auth/me".into(), "GET", String::new(), [127, 0, 0, 9]).await,
        StatusCode::UNAUTHORIZED
    );
    for _ in 0..10 {
        assert_ne!(
            limited_call(
                "/api/upload/avatar".into(),
                "POST",
                customer.clone(),
                [127, 0, 0, 8]
            )
            .await,
            StatusCode::TOO_MANY_REQUESTS
        );
    }
    assert_eq!(
        limited_call(
            "/api/upload/avatar".into(),
            "POST",
            customer.clone(),
            [127, 0, 0, 8]
        )
        .await,
        StatusCode::TOO_MANY_REQUESTS
    );
    let second_customer_session = login(&app, &customer_email).await;
    assert_eq!(
        limited_call(
            "/api/upload/avatar".into(),
            "POST",
            second_customer_session,
            [127, 0, 0, 9]
        )
        .await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_ne!(
        limited_call(
            "/api/upload/avatar".into(),
            "POST",
            vendor.clone(),
            [127, 0, 0, 8]
        )
        .await,
        StatusCode::TOO_MANY_REQUESTS
    );
    for _ in 0..120 {
        assert_ne!(
            limited_call(
                "/api/orders/missing/location".into(),
                "PUT",
                courier.clone(),
                [127, 0, 0, 8]
            )
            .await,
            StatusCode::TOO_MANY_REQUESTS
        );
    }
    assert_eq!(
        limited_call(
            "/api/orders/missing/location".into(),
            "PUT",
            courier.clone(),
            [127, 0, 0, 8]
        )
        .await,
        StatusCode::TOO_MANY_REQUESTS
    );

    let second_courier_session = login(&app, &courier_email).await;
    assert_ne!(
        limited_call(
            "/api/orders/missing/location".into(),
            "PUT",
            second_courier_session,
            [127, 0, 0, 8]
        )
        .await,
        StatusCode::TOO_MANY_REQUESTS
    );

    // Authenticated photo uploads are bound to the session identity and product owner.
    let upload_photo = |route: String, cookie: String, bytes: Vec<u8>| {
        let app = app.clone();
        async move {
            let boundary = "avatar-fixture";
            let mut body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"photo.png\"\r\nContent-Type: image/png\r\n\r\n").into_bytes();
            body.extend(bytes);
            body.extend(format!("\r\n--{boundary}--\r\n").bytes());
            app.oneshot(
                Request::post(route)
                    .header("cookie", cookie)
                    .header("x-suki-request", "1")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap()
        }
    };
    let mut png = std::io::Cursor::new(Vec::new());
    image::RgbImage::from_pixel(3, 2, image::Rgb([1, 2, 3]))
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    let png = png.into_inner();
    for cookie in [&customer, &vendor, &courier] {
        let response = upload_photo("/api/upload/avatar".into(), cookie.clone(), png.clone()).await;
        assert_eq!(response.status(), StatusCode::OK);
        let result: Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        let path = result["image"].as_str().unwrap();
        let photo = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(photo.status(), StatusCode::OK);
        assert_eq!(photo.headers()["content-type"], "image/jpeg");
        assert_eq!(
            ok(&app, "GET", "/api/upload/avatar", cookie, Value::Null).await["image"],
            path
        );
    }
    for cookie in [&customer, &courier, &vendor2] {
        let response = upload_photo(
            format!("/api/upload/products/{pid}"),
            cookie.clone(),
            png.clone(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    let mut webp = std::io::Cursor::new(Vec::new());
    image::RgbImage::from_pixel(2, 2, image::Rgb([70, 90, 10]))
        .write_to(&mut webp, image::ImageFormat::WebP)
        .unwrap();
    let webp = upload_photo(
        "/api/upload/avatar".into(),
        customer.clone(),
        webp.into_inner(),
    )
    .await;
    assert_eq!(webp.status(), StatusCode::OK);
    let invalid = upload_photo(
        "/api/upload/avatar".into(),
        customer.clone(),
        b"not an image".to_vec(),
    )
    .await;
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let oversized = upload_photo(
        "/api/upload/avatar".into(),
        customer.clone(),
        vec![0; 5 * 1024 * 1024 + 1],
    )
    .await;
    assert_eq!(oversized.status(), StatusCode::BAD_REQUEST);
    let product_photo = upload_photo(
        format!("/api/upload/products/{pid}"),
        vendor.clone(),
        png.clone(),
    )
    .await;
    assert_eq!(product_photo.status(), StatusCode::OK);
    let image_url: Option<String> = sqlx::query_scalar("SELECT image_url FROM products WHERE id=?")
        .bind(pid)
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(
        image_url
            .as_deref()
            .unwrap()
            .starts_with("https://media.example.invalid/images/")
    );

    let listing = ok(&app, "GET", "/api/products", "", Value::Null).await;
    let listed = listing
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == pid)
        .unwrap();
    assert_eq!(listed["image_url"], image_url.unwrap());
    let mut update = product(16000);
    update["image"] = listed["image"].clone();
    ok(
        &app,
        "PUT",
        &format!("/api/products/{pid}"),
        &vendor,
        update,
    )
    .await;
    let kept: Option<String> = sqlx::query_scalar("SELECT image_url FROM products WHERE id=?")
        .bind(pid)
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(kept.is_some());

    let mut settings = ok(&app, "GET", "/api/settings", "", Value::Null).await;
    settings["gcash_qr_url"] = json!("/images/synthetic-test-qr.png");
    ok(&app, "PUT", "/api/settings", &admin, settings).await;
    let unavailable: App = Arc::new(AppState {
        admin_emails: s.admin_emails.clone(),
        db: db.clone(),
        rate_limits: Default::default(),
        integrity_key: s.integrity_key.clone(),
        field_cipher: field_cipher.clone(),
        session_key: s.session_key.clone(),
        client: reqwest::Client::new(),
        secure_cookies: false,
        origin: "http://localhost:3000".into(),
        maps_key: String::new(),
        maps_url: "http://127.0.0.1:1".into(),
        legacy_quotes: true,
        r2: None,
        media_dir: std::env::temp_dir().join(format!("suki-media-{}", id())),
        events: tokio::sync::broadcast::channel(4).0,
    });
    assert_eq!(
        call(
            &suki_api::router(unavailable),
            "POST",
            "/api/delivery/quote",
            &customer,
            json!({"address":"123 Synthetic Address, Davao City","lat":7.05,"lng":125.59})
        )
        .await
        .0,
        503
    );
    assert_eq!(
        ok(&app, "GET", "/api/health", "", Value::Null).await["status"],
        "ok"
    );
    let q = quote(&app, &customer).await;
    assert_eq!(q["distance_meters"], 1234);
    assert_eq!(q["delivery_fee_centavos"], 6432);
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/delivery/quote",
            &customer,
            json!({"address":"Outside coverage test address","lat":10.0,"lng":125.0})
        )
        .await
        .0,
        400
    );
    let mut body = json!({"quote_id":q["id"],"idempotency_key":id(),"items":[{"product_id":pid,"quantity_milli":2000,"expected_price_centavos":16000}]});
    assert_eq!(
        call(&app, "POST", "/api/orders", &customer, body.clone())
            .await
            .0,
        400
    );
    body["items"][0]["quantity_milli"] = json!(3000);
    body["items"][0]["expected_price_centavos"] = json!(15999);
    assert_eq!(
        call(&app, "POST", "/api/orders", &customer, body.clone())
            .await
            .0,
        409
    );
    body["items"][0]["expected_price_centavos"] = json!(16000);
    let created = ok(&app, "POST", "/api/orders", &customer, body.clone()).await;
    let oid = created["id"].as_str().unwrap();
    let path = format!("/api/orders/{oid}");
    let replay = ok(&app, "POST", "/api/orders", &customer, body.clone()).await;
    assert_eq!(created["id"], replay["id"]);
    assert_eq!(replay["replayed"], true);
    body["items"][0]["quantity_milli"] = json!(4000);
    assert_eq!(
        call(&app, "POST", "/api/orders", &customer, body).await.0,
        409
    );
    assert_eq!(call(&app, "GET", &path, &other, Value::Null).await.0, 403);
    assert_eq!(call(&app, "GET", &path, &courier, Value::Null).await.0, 403);
    assert_eq!(call(&app, "GET", &path, "", Value::Null).await.0, 401);
    ok(
        &app,
        "PUT",
        &format!("/api/products/{pid}"),
        &vendor,
        product(20000),
    )
    .await;
    let detail = ok(&app, "GET", &path, &customer, Value::Null).await;
    assert_eq!(detail["items"][0]["unit_price_centavos"], 16000);
    assert_eq!(detail["order"]["estimated_total_centavos"], 54432);
    let item = detail["items"][0]["id"].as_str().unwrap();
    assert_eq!(
        call(
            &app,
            "PATCH",
            &format!("{path}/status"),
            &admin,
            json!({"status":"shopping"})
        )
        .await
        .0,
        409
    );
    let reference = format!("TX{}", id().replace('-', ""));
    let payment = ok(
        &app,
        "POST",
        &format!("{path}/payments"),
        &customer,
        json!({"reference":reference,"amount_centavos":54432}),
    )
    .await;
    let payid = payment["id"].as_str().unwrap();
    let verify = format!("/api/payments/{payid}/verify");
    assert_eq!(payment["status"], "under_verification");
    // Pending evidence must remain verifiable, even if either role tries to cancel.
    for actor in [&customer, &admin] {
        assert_eq!(
            call(
                &app,
                "PATCH",
                &format!("{path}/status"),
                actor,
                json!({"status":"canceled"}),
            )
            .await
            .0,
            409
        );
    }
    let pending = ok(&app, "GET", &path, &customer, Value::Null).await;
    assert_eq!(pending["order"]["status"], "payment_verification");
    assert_eq!(pending["order"]["payment_status"], "under_verification");
    assert!(pending["refunds"].as_array().unwrap().is_empty());
    assert_eq!(
        call(
            &app,
            "POST",
            &verify,
            &customer,
            json!({"approved":true,"received_amount_centavos":54432})
        )
        .await
        .0,
        403
    );
    assert_eq!(
        call(
            &app,
            "POST",
            &verify,
            &admin,
            json!({"approved":true,"received_amount_centavos":54431})
        )
        .await
        .0,
        400
    );
    ok(
        &app,
        "POST",
        &verify,
        &admin,
        json!({"approved":true,"received_amount_centavos":54432}),
    )
    .await;
    assert_eq!(
        call(
            &app,
            "POST",
            &verify,
            &admin,
            json!({"approved":true,"received_amount_centavos":54432})
        )
        .await
        .0,
        409
    );
    ok(
        &app,
        "PUT",
        &format!("{path}/assign"),
        &admin,
        json!({"courier_id":courier_id}),
    )
    .await;
    ok(
        &app,
        "PATCH",
        &format!("{path}/status"),
        &courier,
        json!({"status":"shopping"}),
    )
    .await;
    assert_eq!(call(&app,"PUT",&format!("{path}/items/{item}/purchase"),&courier,json!({"actual_quantity_milli":3000,"actual_unit_price_centavos":20000,"unavailable":false})).await.0,409);
    let requested = ok(&app, "GET", "/api/vendor/items", &vendor, Value::Null).await;
    assert!(
        requested
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["item_id"] == item)
    );
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!("/api/vendor/items/{item}"),
            &vendor2,
            json!({"available_quantity_milli":3000})
        )
        .await
        .0,
        403
    );
    ok(
        &app,
        "PUT",
        &format!("/api/vendor/items/{item}"),
        &vendor,
        json!({"available_quantity_milli":3000}),
    )
    .await;
    // The courier proposes a substitution. No purchase can be recorded before approval.
    let sub=ok(&app,"POST",&format!("{path}/substitutions"),&courier,json!({"item_id":item,"name":"Alternative mangoes","unit":"kg","quantity_milli":3000,"unit_price_centavos":15000})).await;
    assert_eq!(call(&app,"PUT",&format!("{path}/items/{item}/purchase"),&courier,json!({"actual_quantity_milli":3000,"actual_unit_price_centavos":15000,"unavailable":false})).await.0,409);
    ok(
        &app,
        "POST",
        &format!("{path}/substitutions/{}", sub["id"].as_str().unwrap()),
        &customer,
        json!({"approved":true}),
    )
    .await;
    ok(&app,"PUT",&format!("{path}/items/{item}/purchase"),&courier,json!({"actual_quantity_milli":3000,"actual_unit_price_centavos":15000,"unavailable":false})).await;
    let rec = ok(
        &app,
        "POST",
        &format!("{path}/reconcile"),
        &courier,
        Value::Null,
    )
    .await;
    assert_eq!(rec["status"], "ready_for_delivery");
    assert_eq!(rec["difference_centavos"], -3000);
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("{path}/messages"),
            &other,
            Value::Null
        )
        .await
        .0,
        403
    );
    ok(
        &app,
        "POST",
        &format!("{path}/messages"),
        &customer,
        json!({"content":"Please ring the bell."}),
    )
    .await;
    let messages = ok(
        &app,
        "GET",
        &format!("{path}/messages?after=0"),
        &courier,
        Value::Null,
    )
    .await;
    assert_eq!(messages[0]["content"], "Please ring the bell.");
    assert_eq!(messages[0]["sender_id"], customer_id);
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("{path}/location"),
            &customer,
            Value::Null
        )
        .await
        .0,
        409
    );
    ok(
        &app,
        "PATCH",
        &format!("{path}/status"),
        &courier,
        json!({"status":"out_for_delivery"}),
    )
    .await;
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!("{path}/location"),
            &other,
            json!({"lat":7.05,"lng":125.59})
        )
        .await
        .0,
        403
    );
    ok(
        &app,
        "PUT",
        &format!("{path}/location"),
        &courier,
        json!({"lat":7.05,"lng":125.59}),
    )
    .await;
    assert_eq!(
        call(
            &app,
            "PUT",
            &format!("{path}/location"),
            &courier,
            json!({"lat":7.05,"lng":125.59})
        )
        .await
        .0,
        429
    );
    let location = ok(
        &app,
        "GET",
        &format!("{path}/location"),
        &customer,
        Value::Null,
    )
    .await;
    assert_eq!(location["lat"], 7.05);
    ok(
        &app,
        "PATCH",
        &format!("{path}/status"),
        &courier,
        json!({"status":"delivered"}),
    )
    .await;
    assert_eq!(
        call(
            &app,
            "GET",
            &format!("{path}/location"),
            &customer,
            Value::Null
        )
        .await
        .0,
        409
    );
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM delivery_locations WHERE order_id=?")
        .bind(oid)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(count, 0);
    let detail = ok(&app, "GET", &path, &admin, Value::Null).await;
    assert_eq!(detail["refunds"][0][1], 3000);
    let rid = detail["refunds"][0][0].as_str().unwrap();
    ok(
        &app,
        "POST",
        &format!("/api/refunds/{rid}/process"),
        &admin,
        json!({"reference":"SYNTHETIC-REFUND-42"}),
    )
    .await;
    assert_eq!(
        call(
            &app,
            "POST",
            &format!("/api/refunds/{rid}/process"),
            &admin,
            json!({"reference":"SYNTHETIC-REFUND-42"})
        )
        .await
        .0,
        409
    );
    ok(
        &app,
        "PATCH",
        &format!("{path}/status"),
        &courier,
        json!({"status":"completed"}),
    )
    .await;
    let reports = ok(&app, "GET", "/api/admin/reports", &admin, Value::Null).await;
    assert!(
        reports["orders"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["order_id"] == oid && r["margin_centavos"] == 6432)
    );
    assert_eq!(
        call(&app, "GET", "/api/admin/reports", &customer, Value::Null)
            .await
            .0,
        403
    );
    // Exercise a real authenticated WebSocket handshake and event delivery.
    use futures_util::StreamExt;
    use tokio_tungstenite::{
        connect_async,
        tungstenite::{Message as WsMessage, client::IntoClientRequest},
    };
    let live_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let live_address = live_listener.local_addr().unwrap();
    let live_app = app.clone();
    let live_task =
        tokio::spawn(async move { axum::serve(live_listener, live_app).await.unwrap() });
    let ws_url = format!("ws://{live_address}{path}/events");
    let mut forbidden = ws_url.clone().into_client_request().unwrap();
    forbidden
        .headers_mut()
        .insert("origin", "http://localhost:3000".parse().unwrap());
    forbidden
        .headers_mut()
        .insert("cookie", other.parse().unwrap());
    assert!(connect_async(forbidden).await.is_err());
    let mut wrong_origin = ws_url.clone().into_client_request().unwrap();
    wrong_origin
        .headers_mut()
        .insert("origin", "https://untrusted.invalid".parse().unwrap());
    wrong_origin
        .headers_mut()
        .insert("cookie", customer.parse().unwrap());
    assert!(connect_async(wrong_origin).await.is_err());
    let mut request = ws_url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert("origin", "http://localhost:3000".parse().unwrap());
    request
        .headers_mut()
        .insert("cookie", customer.parse().unwrap());
    let (mut ws, _) = connect_async(request).await.unwrap();
    ok(
        &app,
        "POST",
        &format!("{path}/messages"),
        &courier,
        json!({"content":"Your delivery is complete."}),
    )
    .await;
    let event = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            if let Some(Ok(WsMessage::Text(text))) = ws.next().await {
                break serde_json::from_str::<Value>(&text).unwrap();
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(event["type"], "message");
    assert_eq!(event["message"]["content"], "Your delivery is complete.");
    let resumed = ok(
        &app,
        "GET",
        &format!("{path}/messages?after={}", messages[0]["id"]),
        &customer,
        Value::Null,
    )
    .await;
    assert_eq!(resumed[0]["content"], "Your delivery is complete.");

    // Reusing a verified payment reference on another order must fail.
    let q2 = quote(&app, &customer).await;
    let second_body = json!({"quote_id":q2["id"],"idempotency_key":id(),"items":[{"product_id":pid,"quantity_milli":3000,"expected_price_centavos":20000}]});
    let (one, two) = tokio::join!(
        call(&app, "POST", "/api/orders", &customer, second_body.clone()),
        call(&app, "POST", "/api/orders", &customer, second_body)
    );
    assert_eq!(one.0, 200);
    assert_eq!(two.0, 200);
    assert_eq!(one.1["id"], two.1["id"]);
    let oid2 = one.1["id"].as_str().unwrap();
    let path2 = format!("/api/orders/{oid2}");
    assert_eq!(
        call(
            &app,
            "POST",
            &format!("{path2}/payments"),
            &customer,
            json!({"reference":format!("  {}  ",reference.to_lowercase()),"amount_centavos":66432})
        )
        .await
        .0,
        409
    );
    let p2 = ok(
        &app,
        "POST",
        &format!("{path2}/payments"),
        &customer,
        json!({"reference":format!("TX{}",id().replace('-',"")),"amount_centavos":66432}),
    )
    .await;
    ok(
        &app,
        "POST",
        &format!("/api/payments/{}/verify", p2["id"].as_str().unwrap()),
        &admin,
        json!({"approved":true,"received_amount_centavos":66432}),
    )
    .await;
    ok(
        &app,
        "PUT",
        &format!("{path2}/assign"),
        &admin,
        json!({"courier_id":courier_id}),
    )
    .await;
    ok(
        &app,
        "PATCH",
        &format!("{path2}/status"),
        &courier,
        json!({"status":"shopping"}),
    )
    .await;
    let adjustment=ok(&app,"POST",&format!("{path2}/adjustments"),&courier,json!({"proposed_goods_centavos":63000,"reason":"Vendor reports higher actual weight; ask before buying."})).await;
    assert_eq!(
        call(
            &app,
            "POST",
            &format!("{path2}/adjustments/{}", adjustment["id"].as_str().unwrap()),
            &other,
            json!({"approved":true})
        )
        .await
        .0,
        403
    );
    ok(
        &app,
        "POST",
        &format!("{path2}/adjustments/{}", adjustment["id"].as_str().unwrap()),
        &customer,
        json!({"approved":true}),
    )
    .await;
    let p3 = ok(
        &app,
        "POST",
        &format!("{path2}/payments"),
        &customer,
        json!({"reference":format!("TX{}",id().replace('-',"")),"amount_centavos":3000}),
    )
    .await;
    ok(
        &app,
        "POST",
        &format!("/api/payments/{}/verify", p3["id"].as_str().unwrap()),
        &admin,
        json!({"approved":true,"received_amount_centavos":3000}),
    )
    .await;
    let second_detail = ok(&app, "GET", &path2, &customer, Value::Null).await;
    let second_item = second_detail["items"][0]["id"].as_str().unwrap();
    ok(&app,"PUT",&format!("{path2}/items/{second_item}/purchase"),&courier,json!({"actual_quantity_milli":3000,"actual_unit_price_centavos":21000,"unavailable":false})).await;
    assert_eq!(
        ok(
            &app,
            "POST",
            &format!("{path2}/reconcile"),
            &courier,
            Value::Null
        )
        .await["status"],
        "ready_for_delivery"
    );
    ok(
        &app,
        "PATCH",
        &format!("{path2}/status"),
        &admin,
        json!({"status":"canceled"}),
    )
    .await;
    let canceled = ok(&app, "GET", &path2, &customer, Value::Null).await;
    assert_eq!(canceled["refunds"][0][1], 69432);
    let marketplace = suki_api::router(Arc::new(AppState {
        rate_limits: Default::default(),
        ..(*s).clone()
    }));
    marketplace_workflow(
        &marketplace,
        s.as_ref(),
        live_address,
        [&customer, &vendor, &courier, &admin, &vendor2],
        &courier_id,
    )
    .await;

    ok(
        &app,
        "PATCH",
        &format!("/api/admin/users/{vendor2_id}"),
        &admin,
        json!({"role":"vendor","status":"active"}),
    )
    .await;
    assert_eq!(
        call(&app, "GET", "/api/auth/me", &vendor2, Value::Null)
            .await
            .0,
        401
    );

    ok(&app, "POST", "/api/auth/logout", &customer, Value::Null).await;
    assert_eq!(
        call(&app, "GET", &path, &customer, Value::Null).await.0,
        401
    );
    // Every accepted critical action is audited; passwords and session tokens are not returned.
    let logs = ok(&app, "GET", "/api/admin/audit", &admin, Value::Null).await;
    assert!(
        logs.as_array()
            .unwrap()
            .iter()
            .any(|r| r["action"] == "payment.verify")
    );
    live_task.abort();
    storage_task.abort();
    task.abort();
    db.close().await;
}

async fn marketplace_workflow(
    app: &axum::Router,
    state: &AppState,
    address: std::net::SocketAddr,
    [customer, vendor, rider, admin, stranger]: [&str; 5],
    rider_id: &str,
) {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{
        connect_async,
        tungstenite::{Message, client::IntoClientRequest},
    };
    let mut buffer = std::io::Cursor::new(Vec::new());
    image::RgbImage::from_pixel(3, 2, image::Rgb([160, 20, 60]))
        .write_to(&mut buffer, image::ImageFormat::Png)
        .unwrap();
    let fields = json!({
        "name": "Ripe Mangoes", "description":"A vendor listing", "category":"Fruits",
        "unit":"bag", "price_centavos":24000, "weight_grams":2000
    });
    let boundary = "suki-integration-upload";
    let mut body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"product\"\r\n\r\n{fields}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"photo\"; filename=\"../../sample.png\"\r\nContent-Type: image/png\r\n\r\n").into_bytes();
    body.extend(buffer.into_inner());
    body.extend(format!("\r\n--{boundary}--\r\n").bytes());
    let upload = Request::post("/api/vendor/goods")
        .header("cookie", vendor)
        .header("x-suki-request", "1")
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(Body::from(body))
        .unwrap();
    let response = app.clone().oneshot(upload).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let uploaded: Value =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    let pid = uploaded["id"].as_i64().unwrap();
    let (path, grams): (String, i64) =
        sqlx::query_as("SELECT image,weight_grams FROM products WHERE id=?")
            .bind(pid)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(grams, 2000);
    let photo = app
        .clone()
        .oneshot(Request::get(&path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(photo.headers()["content-type"], "image/jpeg");
    let bytes = photo.into_body().collect().await.unwrap().to_bytes();
    let decoded = image::load_from_memory(&bytes).unwrap();
    assert_eq!((decoded.width(), decoded.height()), (3, 2));
    assert!(
        !state
            .media_dir
            .join(path.strip_prefix("/images/vendor/").unwrap())
            .exists()
    );
    let (image_url, owner, product_id): (String, String, i64) = sqlx::query_as("SELECT m.public_url,m.owner_id,m.product_id FROM media_objects m JOIN products p ON p.image_url=m.public_url WHERE p.id=?")
        .bind(pid).fetch_one(&state.db).await.unwrap();
    assert!(image_url.starts_with("https://media.example.invalid/images/"));
    assert_eq!(product_id, pid);
    let vendor_id: String = sqlx::query_scalar("SELECT vendor_id FROM products WHERE id=?")
        .bind(pid)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(owner, vendor_id);

    let price_path = format!("/api/vendor/goods/{pid}/price");
    assert_eq!(
        call(
            app,
            "PATCH",
            &price_path,
            stranger,
            json!({"price_centavos":17000})
        )
        .await
        .0,
        403
    );
    ok(
        app,
        "PATCH",
        &price_path,
        vendor,
        json!({"price_centavos":25000}),
    )
    .await;
    ok(
        app,
        "PATCH",
        &format!("/api/vendor/goods/{pid}/availability"),
        vendor,
        json!({"available":true}),
    )
    .await;
    let make_quote = |grams| {
        json!({"address":"123 Synthetic Street, Matina, Davao City",
        "lat":7.05,"lng":125.59,"weight_grams":grams})
    };
    let make_checkout = |quote: Value, payment| {
        json!({"quote_id":quote["id"],"idempotency_key":id(),
        "payment_method":payment,"items":[{"product_id":pid,"quantity_milli":2000,"expected_price_centavos":25000}]})
    };
    let too_light = ok(
        app,
        "POST",
        "/api/quotes/delivery",
        customer,
        make_quote(1000),
    )
    .await;
    assert_eq!(
        call(
            app,
            "POST",
            "/api/orders",
            customer,
            make_checkout(too_light, "cash_on_delivery")
        )
        .await
        .0,
        409
    );
    let quote = ok(
        app,
        "POST",
        "/api/quotes/delivery",
        customer,
        make_quote(4000),
    )
    .await;
    assert_eq!(
        (
            quote["delivery_fee_centavos"].as_i64(),
            quote["weight_grams"].as_i64()
        ),
        (Some(8500), Some(4000))
    );
    let checkout_body = make_checkout(quote, "cash_on_delivery");
    let order = ok(
        app,
        "POST",
        "/api/orders/checkout",
        customer,
        checkout_body.clone(),
    )
    .await;
    let oid = order["id"].as_str().unwrap();
    let path = format!("/api/orders/{oid}");
    let receipt_path = format!("{path}/receipt");
    let receipt: suki_shared::snapshots::OrderSnapshot =
        serde_json::from_value(order["snapshot"].clone()).unwrap();
    assert!(receipt.valid_digest_for(oid));
    assert!(state.integrity_key.verify(
        suki_shared::crypto::SNAPSHOT_AUTH,
        &receipt.snapshot_hash,
        &receipt.signature
    ));
    assert_eq!(receipt.payload.total_centavos, 58500);
    assert_eq!(receipt.payload.delivery.total_centavos, 8500);
    assert_eq!(receipt.payload.delivery.distance_meters, 1234);
    assert_eq!(receipt.payload.delivery.weight_grams, Some(4000));
    assert_eq!(receipt.payload.items[0].unit_price_centavos, 25000);
    assert_eq!(receipt.payload.items[0].unit_weight_grams, 2000);

    // Changing a live listing cannot change the signed receipt or the vendor's locked price.
    ok(
        app,
        "PATCH",
        &price_path,
        vendor,
        json!({"price_centavos":26000}),
    )
    .await;
    ok(
        app,
        "PATCH",
        &format!("/api/vendor/goods/{pid}/availability"),
        vendor,
        json!({"available":false}),
    )
    .await;
    assert_eq!(
        ok(app, "GET", &receipt_path, customer, Value::Null).await,
        order["snapshot"]
    );
    for account in [vendor, rider, stranger] {
        assert_eq!(
            call(app, "GET", &receipt_path, account, Value::Null)
                .await
                .0,
            403
        );
    }
    let vendor_queue = ok(app, "GET", "/api/vendor/items", vendor, Value::Null).await;
    let line = vendor_queue
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["order_id"] == oid)
        .unwrap();
    assert_eq!(line["unit_price_centavos"], 25000);
    assert_eq!(line["line_total_centavos"], 50000);
    assert_eq!(line["snapshot_verified"], true);
    assert!(line.get("address").is_none());
    assert!(line.get("payment_method").is_none());

    let replay = ok(
        app,
        "POST",
        "/api/orders/checkout",
        customer,
        checkout_body.clone(),
    )
    .await;
    assert_eq!(replay["id"], oid);
    assert_eq!(replay["replayed"], true);
    assert_eq!(replay["snapshot"], order["snapshot"]);

    // If receipt authentication fails, a retry must still recover the committed order.
    let wrong_key_app = suki_api::router(Arc::new(AppState {
        integrity_key: suki_shared::crypto::IntegrityKey::from_hex(&"ab".repeat(32)).unwrap(),
        ..state.clone()
    }));
    let recovered = ok(
        &wrong_key_app,
        "POST",
        "/api/orders/checkout",
        customer,
        checkout_body,
    )
    .await;
    assert_eq!(recovered["id"], oid);
    assert_eq!(recovered["replayed"], true);
    assert_eq!(recovered["snapshot_invalid"], true);
    assert!(recovered["snapshot"].is_null());
    assert_eq!(
        call(&wrong_key_app, "GET", &receipt_path, customer, Value::Null)
            .await
            .0,
        409
    );
    assert!(
        sqlx::query("UPDATE order_snapshots SET payload='{}' WHERE order_id=?")
            .bind(oid)
            .execute(&state.db)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("DELETE FROM order_snapshots WHERE order_id=?")
            .bind(oid)
            .execute(&state.db)
            .await
            .is_err()
    );

    ok(
        app,
        "PATCH",
        &price_path,
        vendor,
        json!({"price_centavos":25000}),
    )
    .await;
    ok(
        app,
        "PATCH",
        &format!("/api/vendor/goods/{pid}/availability"),
        vendor,
        json!({"available":true}),
    )
    .await;
    let assign = format!("{path}/assign");
    let status = format!("{path}/status");
    ok(app, "PUT", &assign, admin, json!({"courier_id":rider_id})).await;
    assert_eq!(call(app, "GET", &path, vendor, Value::Null).await.0, 403);
    assert_eq!(
        call(
            app,
            "GET",
            &format!("{path}/messages"),
            stranger,
            Value::Null
        )
        .await
        .0,
        403
    );
    let ws_url = format!("ws://{address}{path}/chat/ws");
    let connect = |cookie: &str| {
        let mut request = ws_url.as_str().into_client_request().unwrap();
        request
            .headers_mut()
            .insert("origin", "http://localhost:3000".parse().unwrap());
        request
            .headers_mut()
            .insert("cookie", cookie.parse().unwrap());
        connect_async(request)
    };
    assert!(connect(stranger).await.is_err());
    let mut sockets = Vec::new();
    for participant in [customer, vendor, rider] {
        let (mut socket, _) = connect(participant).await.unwrap();
        assert!(matches!(
            tokio::time::timeout(std::time::Duration::from_secs(3), socket.next())
                .await
                .unwrap(),
            Some(Ok(Message::Ping(_)))
        ));
        sockets.push(socket);
    }
    for (sender, role) in [
        (vendor, "vendor"),
        (customer, "customer"),
        (rider, "courier"),
    ] {
        let content = format!("Pickup update from {role}");
        ok(
            app,
            "POST",
            &format!("{path}/messages"),
            sender,
            json!({"content":content}),
        )
        .await;
        for socket in &mut sockets {
            let response = tokio::time::timeout(std::time::Duration::from_secs(3), async {
                while let Some(Ok(frame)) = socket.next().await {
                    if let Message::Text(text) = frame {
                        return serde_json::from_str::<Value>(&text).unwrap();
                    }
                }
                panic!("Socket closed without broadcasting the order message");
            })
            .await
            .unwrap();
            assert_eq!(response["message"]["content"], content);
            assert_eq!(response["message"]["sender_role"], role);
            assert_eq!(response["message"]["order_id"], oid);
            let mut message: suki_shared::realtime::ChatMessage =
                serde_json::from_value(response["message"].clone()).unwrap();
            assert!(suki_api::integrity::valid_message(
                &state.integrity_key,
                &message
            ));
            message.content.push('!');
            assert!(!suki_api::integrity::valid_message(
                &state.integrity_key,
                &message
            ));
        }
    }

    assert_eq!(
        call(
            app,
            "POST",
            &format!("{path}/payments"),
            customer,
            json!({"reference":"COD-CANNOT","amount_centavos":58500})
        )
        .await
        .0,
        409
    );
    ok(app, "PATCH", &status, rider, json!({"status":"shopping"})).await;
    let detail = ok(app, "GET", &path, rider, Value::Null).await;
    let line_id = detail["items"][0]["id"].as_str().unwrap();
    ok(app, "PUT", &format!("{path}/items/{line_id}/purchase"), rider,
        json!({"actual_quantity_milli":2000,"actual_unit_price_centavos":24500,"unavailable":false})).await;
    let reconciliation = ok(
        app,
        "POST",
        &format!("{path}/reconcile"),
        rider,
        Value::Null,
    )
    .await;
    assert_eq!(reconciliation["final_goods_centavos"], 49000);
    ok(
        app,
        "PATCH",
        &status,
        rider,
        json!({"status":"out_for_delivery"}),
    )
    .await;
    let reviews_path = format!("{path}/reviews");
    assert_eq!(
        call(
            app,
            "POST",
            &format!("{reviews_path}/vendor"),
            customer,
            json!({"rating":5,"tags":[]})
        )
        .await
        .0,
        409
    );
    let tracking_url = format!("ws://{address}/api/rider/location/stream?order_id={oid}");
    let track = |cookie: &str| {
        let mut req = tracking_url.as_str().into_client_request().unwrap();
        req.headers_mut()
            .insert("origin", "http://localhost:3000".parse().unwrap());
        req.headers_mut().insert("cookie", cookie.parse().unwrap());
        connect_async(req)
    };
    assert!(track(stranger).await.is_err());
    assert!(track(vendor).await.is_err());
    let (mut forbidden_sender, _) = track(customer).await.unwrap();
    forbidden_sender
        .send(Message::Text(
            json!({"lat":7.05,"lng":125.59,"observed_at":now()})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while let Some(Ok(frame)) = forbidden_sender.next().await {
            if matches!(frame, Message::Close(_)) {
                return;
            }
        }
    })
    .await
    .expect("customer cannot publish location");
    let (mut tracking_customer, _) = track(customer).await.unwrap();
    let (mut tracking_rider, _) = track(rider).await.unwrap();
    tracking_rider
        .send(Message::Text(
            json!({"lat":7.05,"lng":125.59,"heading":360,"observed_at":now()})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    let rejection = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while let Some(Ok(frame)) = tracking_rider.next().await {
            if let Message::Text(text) = frame {
                return serde_json::from_str::<Value>(&text).unwrap();
            }
        }
        panic!("Expected invalid telemetry rejection")
    })
    .await
    .unwrap();
    assert!(rejection["error"].is_string());
    tracking_rider
        .send(Message::Text(
            json!({"lat":7.05,"lng":125.59,"heading":90.0,"speed":8.0,"observed_at":now()})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
    let telemetry = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while let Some(Ok(frame)) = tracking_customer.next().await {
            if let Message::Text(text) = frame {
                return serde_json::from_str::<Value>(&text).unwrap();
            }
        }
        panic!("Tracking stream closed before receiving telemetry")
    })
    .await
    .unwrap();
    assert_eq!(telemetry["type"], "location");
    assert_eq!(telemetry["heading"], 90.0);
    assert_eq!(telemetry["speed"], 8.0);
    assert_eq!(
        call(app, "GET", &format!("{path}/eta"), stranger, Value::Null)
            .await
            .0,
        403
    );
    let estimate = ok(app, "GET", &format!("{path}/eta"), customer, Value::Null).await;
    assert_eq!(estimate["distance_meters"], 1234);
    assert_eq!(estimate["duration_seconds"], 480);
    assert_eq!(
        call(app, "PATCH", &status, rider, json!({"status":"delivered"}))
            .await
            .0,
        409
    );
    let collection = format!("{path}/cash-collection");
    assert_eq!(
        call(
            app,
            "POST",
            &collection,
            rider,
            json!({"amount_centavos":57499})
        )
        .await
        .0,
        400
    );
    ok(
        app,
        "POST",
        &collection,
        rider,
        json!({"amount_centavos":57500}),
    )
    .await;
    assert_eq!(
        call(
            app,
            "POST",
            &collection,
            rider,
            json!({"amount_centavos":57500})
        )
        .await
        .0,
        409
    );
    sqlx::query("UPDATE users SET total_deliveries=9 WHERE id=?")
        .bind(rider_id)
        .execute(&state.db)
        .await
        .unwrap();
    ok(app, "PATCH", &status, rider, json!({"status":"delivered"})).await;
    for socket in [&mut tracking_customer, &mut tracking_rider] {
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while let Some(Ok(frame)) = socket.next().await {
                if matches!(frame, Message::Close(_)) {
                    return;
                }
            }
        })
        .await
        .expect("tracking closes on delivery");
    }
    assert!(track(customer).await.is_err());
    assert_eq!(
        call(app, "GET", &format!("{path}/eta"), customer, Value::Null)
            .await
            .0,
        403
    );
    assert_eq!(
        call(
            app,
            "POST",
            &format!("{reviews_path}/vendor"),
            rider,
            json!({"rating":5,"tags":[]})
        )
        .await
        .0,
        403
    );
    assert_eq!(
        call(
            app,
            "POST",
            &format!("{reviews_path}/vendor"),
            stranger,
            json!({"rating":5,"tags":[]})
        )
        .await
        .0,
        403
    );
    assert_eq!(
        call(
            app,
            "POST",
            &format!("{reviews_path}/vendor"),
            customer,
            json!({"rating":0,"tags":[]})
        )
        .await
        .0,
        400
    );
    assert_eq!(
        call(
            app,
            "POST",
            &format!("{reviews_path}/vendor"),
            customer,
            json!({"rating":5,"tags":["speed"]})
        )
        .await
        .0,
        400
    );
    ok(
        app,
        "POST",
        &format!("{reviews_path}/vendor"),
        customer,
        json!({"rating":5,"comment":"Fresh", "tags":["quality"]}),
    )
    .await;
    ok(
        app,
        "POST",
        &format!("{reviews_path}/rider"),
        customer,
        json!({"rating":4,"tags":["courtesy"]}),
    )
    .await;
    assert_eq!(
        call(
            app,
            "POST",
            &format!("{reviews_path}/vendor"),
            customer,
            json!({"rating":4,"tags":[]})
        )
        .await
        .0,
        409
    );
    assert_eq!(
        call(
            app,
            "POST",
            &format!("{reviews_path}/rider"),
            customer,
            json!({"rating":5,"tags":[]})
        )
        .await
        .0,
        409
    );
    let review = ok(app, "GET", &reviews_path, customer, Value::Null).await;
    assert_eq!(review["vendor"]["rating"], 5);
    assert_eq!(review["rider"]["rating"], 4);
    let vendor_reputation = ok(
        app,
        "GET",
        &format!("/api/reputation/vendor/{vendor_id}"),
        customer,
        Value::Null,
    )
    .await;
    assert_eq!(vendor_reputation["score"], "5.00");
    assert_eq!(vendor_reputation["total_reviews"], 1);
    assert_eq!(vendor_reputation["reputation_points"], 15);
    let rider_reputation = ok(
        app,
        "GET",
        &format!("/api/reputation/rider/{rider_id}"),
        customer,
        Value::Null,
    )
    .await;
    assert_eq!(rider_reputation["score"], "4.00");
    assert_eq!(rider_reputation["reputation_points"], 35);
    assert_eq!(rider_reputation["total_deliveries"], 10);
    assert_eq!(
        call(app, "PATCH", &status, rider, json!({"status":"completed"}))
            .await
            .0,
        409
    );
    ok(
        app,
        "POST",
        &format!("{path}/vendor-settlement"),
        rider,
        json!({"reference":"VENDOR-COD-001"}),
    )
    .await;
    ok(app, "PATCH", &status, rider, json!({"status":"completed"})).await;
    let delivered = ok(app, "GET", &path, customer, Value::Null).await;
    assert_eq!(delivered["snapshot"], order["snapshot"]);
    assert_eq!(delivered["settlement"]["amount_centavos"], 49000);
    assert_eq!(delivered["payments"][0]["received_amount_centavos"], 57500);

    ok(
        app,
        "PUT",
        "/api/vendor/payment-profile",
        vendor,
        json!({"gcash_number":"09123456789"}),
    )
    .await;
    let quote = ok(
        app,
        "POST",
        "/api/quotes/delivery",
        customer,
        make_quote(4000),
    )
    .await;
    let direct = ok(
        app,
        "POST",
        "/api/orders",
        customer,
        make_checkout(quote, "gcash"),
    )
    .await;
    let direct = format!("/api/orders/{}", direct["id"].as_str().unwrap());
    let detail = ok(app, "GET", &direct, customer, Value::Null).await;
    assert_eq!(detail["order"]["gcash_number"], "09123456789");
    ok(
        app,
        "PUT",
        &format!("{direct}/assign"),
        admin,
        json!({"courier_id":rider_id}),
    )
    .await;
    let reference = format!("GCASH-{}", id());
    let evidence = ok(
        app,
        "POST",
        &format!("{direct}/payments"),
        customer,
        json!({"reference":reference,"amount_centavos":58500}),
    )
    .await;
    let queue = ok(
        app,
        "GET",
        "/api/admin/payments/pending",
        admin,
        Value::Null,
    )
    .await;
    assert!(
        queue
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"] == evidence["id"]
                && p["reference_hash"] == suki_shared::crypto::gcash_reference_hash(&reference))
    );
    ok(
        app,
        "POST",
        &format!("/api/payments/{}/verify", evidence["id"].as_str().unwrap()),
        rider,
        json!({"approved":true,"received_amount_centavos":58500}),
    )
    .await;
    assert_eq!(
        ok(app, "GET", &direct, customer, Value::Null).await["order"]["status"],
        "paid"
    );
    let integrity_path = format!("/api/admin/orders/{oid}/integrity");
    assert_eq!(
        call(app, "GET", &integrity_path, customer, Value::Null)
            .await
            .0,
        403
    );
    let report = ok(app, "GET", &integrity_path, admin, Value::Null).await;
    assert_eq!(report["messages_verified"], 3);
    assert_eq!(report["snapshot"], "verified");
    assert_eq!(report["legacy_messages"], 0);
    assert!(report["transactions_verified"].as_u64().unwrap() >= 8);
    assert!(report["first_invalid_transaction"].is_null());
    assert!(report["first_invalid_message"].is_null());
    let entries = report["recent"].as_array().unwrap();
    assert!(
        entries
            .iter()
            .any(|entry| entry["event"] == "payments.insert")
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry["event"] == "vendor_settlements.insert")
    );
    assert!(
        sqlx::query("UPDATE order_integrity SET payload='{}' WHERE order_id=?")
            .bind(oid)
            .execute(&state.db)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("DELETE FROM order_integrity WHERE order_id=?")
            .bind(oid)
            .execute(&state.db)
            .await
            .is_err()
    );
    assert!(
        sqlx::query("UPDATE chat_messages SET content='changed' WHERE order_id=?")
            .bind(oid)
            .execute(&state.db)
            .await
            .is_err()
    );

    // Confirm this paid order is assignable before testing the integrity guard.
    let assignment = format!("{direct}/assign");
    let rider_assignment = json!({"courier_id":rider_id});
    ok(app, "PUT", &assignment, admin, rider_assignment.clone()).await;
    let paid_order_id = detail["order"]["id"].as_str().unwrap();
    let integrity_path = format!("/api/admin/orders/{paid_order_id}/integrity");
    let before = ok(app, "GET", &integrity_path, admin, Value::Null).await;
    assert!(before["first_invalid_transaction"].is_null());

    // A database edit creates an unsigned entry. A later API action must not seal it.
    sqlx::query("UPDATE orders SET delivery_fee_centavos=delivery_fee_centavos+1 WHERE id=?")
        .bind(paid_order_id)
        .execute(&state.db)
        .await
        .unwrap();
    let report = ok(app, "GET", &integrity_path, admin, Value::Null).await;
    assert!(report["first_invalid_transaction"].is_number());
    let (status, error, _) = call(app, "PUT", &assignment, admin, rider_assignment).await;
    assert_eq!(status, 409);
    assert_eq!(error["error"], "Order integrity needs administrator review");
    let after = ok(app, "GET", &integrity_path, admin, Value::Null).await;
    assert_eq!(
        report["first_invalid_transaction"],
        after["first_invalid_transaction"]
    );

    // A cancelled active delivery must close its separate tracking stream and erase GPS.
    let quote = ok(
        app,
        "POST",
        "/api/quotes/delivery",
        customer,
        make_quote(4000),
    )
    .await;
    let canceled_order = ok(
        app,
        "POST",
        "/api/orders/checkout",
        customer,
        make_checkout(quote, "cash_on_delivery"),
    )
    .await;
    let canceled_id = canceled_order["id"].as_str().unwrap();
    let canceled_path = format!("/api/orders/{canceled_id}");
    ok(
        app,
        "PUT",
        &format!("{canceled_path}/assign"),
        admin,
        json!({"courier_id":rider_id}),
    )
    .await;
    ok(
        app,
        "PATCH",
        &format!("{canceled_path}/status"),
        rider,
        json!({"status":"shopping"}),
    )
    .await;
    let detail = ok(app, "GET", &canceled_path, rider, Value::Null).await;
    let item = &detail["items"][0];
    ok(app, "PUT", &format!("{canceled_path}/items/{}/purchase", item["id"].as_str().unwrap()), rider,
        json!({"actual_quantity_milli": item["quantity_milli"], "actual_unit_price_centavos": item["unit_price_centavos"], "unavailable":false})).await;
    ok(
        app,
        "POST",
        &format!("{canceled_path}/reconcile"),
        rider,
        Value::Null,
    )
    .await;
    ok(
        app,
        "PATCH",
        &format!("{canceled_path}/status"),
        rider,
        json!({"status":"out_for_delivery"}),
    )
    .await;
    ok(
        app,
        "PUT",
        &format!("{canceled_path}/location"),
        rider,
        json!({"lat":7.05,"lng":125.59}),
    )
    .await;
    let cancel_url = format!("ws://{address}/api/rider/location/stream?order_id={canceled_id}");
    let mut request = cancel_url.as_str().into_client_request().unwrap();
    request
        .headers_mut()
        .insert("origin", "http://localhost:3000".parse().unwrap());
    request
        .headers_mut()
        .insert("cookie", customer.parse().unwrap());
    let (mut socket, _) = connect_async(request).await.unwrap();
    ok(
        app,
        "PATCH",
        &format!("{canceled_path}/status"),
        admin,
        json!({"status":"canceled"}),
    )
    .await;
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while let Some(Ok(frame)) = socket.next().await {
            if matches!(frame, Message::Close(_)) {
                return;
            }
        }
    })
    .await
    .expect("tracking closes on cancellation");
    let stored: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM delivery_locations WHERE order_id=?")
            .bind(canceled_id)
            .fetch_one(&state.db)
            .await
            .unwrap();
    assert_eq!(stored, 0);

    if state.media_dir.exists() {
        tokio::fs::remove_dir_all(&state.media_dir).await.unwrap();
    }
}
