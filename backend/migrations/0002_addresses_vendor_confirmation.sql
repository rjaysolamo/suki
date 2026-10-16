CREATE TABLE customer_addresses (
 id CHAR(36) PRIMARY KEY, customer_id CHAR(36) NOT NULL, label VARCHAR(80) NOT NULL,
 address VARCHAR(500) NOT NULL, lat DOUBLE NOT NULL, lng DOUBLE NOT NULL,
 FOREIGN KEY(customer_id) REFERENCES users(id), INDEX(customer_id)
);
ALTER TABLE order_items ADD available_quantity_milli BIGINT NULL;
