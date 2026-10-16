CREATE TABLE users (
 id CHAR(36) PRIMARY KEY, name VARCHAR(100) NOT NULL, email VARCHAR(190) NOT NULL UNIQUE,
 phone VARCHAR(30) NOT NULL, password_hash VARCHAR(255) NOT NULL,
 role ENUM('customer','vendor','courier','admin') NOT NULL,
 status ENUM('pending','active','suspended','deactivated') NOT NULL DEFAULT 'active',
 created_at BIGINT NOT NULL
);
CREATE TABLE sessions (
 token_hash CHAR(64) PRIMARY KEY, user_id CHAR(36) NOT NULL, expires_at BIGINT NOT NULL,
 FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE, INDEX(expires_at)
);
CREATE TABLE login_attempts (email_hash CHAR(64) PRIMARY KEY, attempts INT NOT NULL, reset_at BIGINT NOT NULL);
CREATE TABLE vendor_profiles (user_id CHAR(36) PRIMARY KEY, shop_name VARCHAR(120) NOT NULL, FOREIGN KEY(user_id) REFERENCES users(id));
CREATE TABLE categories (name VARCHAR(80) PRIMARY KEY);
INSERT INTO categories VALUES ('Vegetables'),('Fruits'),('Meat & Poultry'),('Fish & Seafood'),('Pantry & Others');
CREATE TABLE products (
 id BIGINT PRIMARY KEY AUTO_INCREMENT, vendor_id CHAR(36) NOT NULL, name VARCHAR(120) NOT NULL,
 local_name VARCHAR(100) NOT NULL DEFAULT '', description TEXT NOT NULL,
 category VARCHAR(80) NOT NULL, unit VARCHAR(30) NOT NULL, price_centavos BIGINT NOT NULL CHECK(price_centavos > 0),
 image VARCHAR(200) NOT NULL DEFAULT '', available BOOLEAN NOT NULL DEFAULT TRUE, updated_at BIGINT NOT NULL,
 FOREIGN KEY(vendor_id) REFERENCES vendor_profiles(user_id), FOREIGN KEY(category) REFERENCES categories(name)
);
CREATE TABLE settings (
 id INT PRIMARY KEY CHECK(id=1), base_fee_centavos BIGINT NOT NULL DEFAULT 6000,
 per_km_centavos BIGINT NOT NULL DEFAULT 350, minimum_centavos BIGINT NOT NULL DEFAULT 45000,
 pickup_lat DOUBLE NOT NULL DEFAULT 7.0644, pickup_lng DOUBLE NOT NULL DEFAULT 125.6027,
 south DOUBLE NOT NULL DEFAULT 6.9, north DOUBLE NOT NULL DEFAULT 7.3,
 west DOUBLE NOT NULL DEFAULT 125.3, east DOUBLE NOT NULL DEFAULT 125.75,
 max_distance_meters BIGINT NOT NULL DEFAULT 30000, gcash_qr_url VARCHAR(500) NOT NULL DEFAULT '',
 chat_retention_days INT NOT NULL DEFAULT 30
);
INSERT INTO settings(id) VALUES (1);
CREATE TABLE delivery_quotes (
 id CHAR(36) PRIMARY KEY, customer_id CHAR(36) NOT NULL, address VARCHAR(500) NOT NULL,
 lat DOUBLE NOT NULL, lng DOUBLE NOT NULL, distance_meters BIGINT NOT NULL,
 fee_centavos BIGINT NOT NULL, base_fee_centavos BIGINT NOT NULL, per_km_centavos BIGINT NOT NULL,
 expires_at BIGINT NOT NULL, consumed BOOLEAN NOT NULL DEFAULT FALSE,
 FOREIGN KEY(customer_id) REFERENCES users(id)
);
CREATE TABLE orders (
 id CHAR(36) PRIMARY KEY, customer_id CHAR(36) NOT NULL, courier_id CHAR(36),
 status VARCHAR(40) NOT NULL, payment_status VARCHAR(40) NOT NULL DEFAULT 'awaiting_payment',
 address VARCHAR(500) NOT NULL, lat DOUBLE NOT NULL, lng DOUBLE NOT NULL,
 distance_meters BIGINT NOT NULL, delivery_fee_centavos BIGINT NOT NULL,
 goods_subtotal_centavos BIGINT NOT NULL, estimated_total_centavos BIGINT NOT NULL,
 final_goods_centavos BIGINT, authorized_goods_centavos BIGINT NOT NULL,
 idempotency_key VARCHAR(100) NOT NULL, request_hash CHAR(64) NOT NULL,
 created_at BIGINT NOT NULL, updated_at BIGINT NOT NULL, delivered_at BIGINT,
 UNIQUE(customer_id,idempotency_key), FOREIGN KEY(customer_id) REFERENCES users(id), FOREIGN KEY(courier_id) REFERENCES users(id)
);
CREATE TABLE order_items (
 id CHAR(36) PRIMARY KEY, order_id CHAR(36) NOT NULL, product_id BIGINT NOT NULL, vendor_id CHAR(36) NOT NULL,
 name VARCHAR(120) NOT NULL, unit VARCHAR(30) NOT NULL, unit_price_centavos BIGINT NOT NULL,
 quantity_milli BIGINT NOT NULL, estimated_cost_centavos BIGINT NOT NULL,
 actual_quantity_milli BIGINT, actual_unit_price_centavos BIGINT, actual_cost_centavos BIGINT,
 unavailable BOOLEAN NOT NULL DEFAULT FALSE,
 FOREIGN KEY(order_id) REFERENCES orders(id), FOREIGN KEY(product_id) REFERENCES products(id)
);
CREATE TABLE payments (
 id CHAR(36) PRIMARY KEY, order_id CHAR(36) NOT NULL,
 reference VARCHAR(100) NOT NULL UNIQUE, claimed_amount_centavos BIGINT NOT NULL,
 received_amount_centavos BIGINT, status VARCHAR(30) NOT NULL DEFAULT 'under_verification',
 verified_by CHAR(36), created_at BIGINT NOT NULL, verified_at BIGINT,
 FOREIGN KEY(order_id) REFERENCES orders(id), FOREIGN KEY(verified_by) REFERENCES users(id)
);
CREATE TABLE refunds (
 id CHAR(36) PRIMARY KEY, order_id CHAR(36) NOT NULL, amount_centavos BIGINT NOT NULL,
 status VARCHAR(30) NOT NULL DEFAULT 'pending', reference VARCHAR(100), processed_by CHAR(36), created_at BIGINT NOT NULL,
 FOREIGN KEY(order_id) REFERENCES orders(id)
);
CREATE TABLE adjustments (
 id CHAR(36) PRIMARY KEY, order_id CHAR(36) NOT NULL, proposed_goods_centavos BIGINT NOT NULL,
 reason VARCHAR(1000) NOT NULL, status VARCHAR(30) NOT NULL DEFAULT 'pending', customer_decided_at BIGINT,
 FOREIGN KEY(order_id) REFERENCES orders(id)
);
CREATE TABLE substitutions (
 id CHAR(36) PRIMARY KEY, order_id CHAR(36) NOT NULL, item_id CHAR(36) NOT NULL,
 name VARCHAR(120) NOT NULL, unit VARCHAR(30) NOT NULL, quantity_milli BIGINT NOT NULL,
 unit_price_centavos BIGINT NOT NULL, status VARCHAR(30) NOT NULL DEFAULT 'pending', decided_at BIGINT,
 FOREIGN KEY(order_id) REFERENCES orders(id), FOREIGN KEY(item_id) REFERENCES order_items(id)
);
CREATE TABLE order_status_history (
 id BIGINT PRIMARY KEY AUTO_INCREMENT, order_id CHAR(36) NOT NULL, status VARCHAR(40) NOT NULL,
 actor_id CHAR(36) NOT NULL, created_at BIGINT NOT NULL, FOREIGN KEY(order_id) REFERENCES orders(id)
);
CREATE TABLE chat_messages (
 id BIGINT PRIMARY KEY AUTO_INCREMENT, order_id CHAR(36) NOT NULL, sender_id CHAR(36) NOT NULL,
 content VARCHAR(2000) NOT NULL, created_at BIGINT NOT NULL,
 FOREIGN KEY(order_id) REFERENCES orders(id), FOREIGN KEY(sender_id) REFERENCES users(id), INDEX(order_id,id)
);
CREATE TABLE delivery_locations (
 order_id CHAR(36) PRIMARY KEY, courier_id CHAR(36) NOT NULL, lat DOUBLE NOT NULL, lng DOUBLE NOT NULL,
 reported_at BIGINT NOT NULL, FOREIGN KEY(order_id) REFERENCES orders(id)
);
CREATE TABLE expenses (
 id CHAR(36) PRIMARY KEY, order_id CHAR(36) NOT NULL, amount_centavos BIGINT NOT NULL,
 description VARCHAR(500) NOT NULL, actor_id CHAR(36) NOT NULL, created_at BIGINT NOT NULL,
 FOREIGN KEY(order_id) REFERENCES orders(id)
);
CREATE TABLE audit_logs (
 id BIGINT PRIMARY KEY AUTO_INCREMENT, actor_id CHAR(36) NOT NULL, action VARCHAR(80) NOT NULL,
 entity_id VARCHAR(100) NOT NULL, details TEXT NOT NULL, created_at BIGINT NOT NULL
);
