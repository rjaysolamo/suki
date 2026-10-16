-- Run daily using a restricted maintenance account after backing up.
DELETE FROM sessions WHERE expires_at < UNIX_TIMESTAMP();
DELETE FROM login_attempts WHERE reset_at < UNIX_TIMESTAMP() - 86400;
DELETE FROM delivery_quotes WHERE expires_at < UNIX_TIMESTAMP() - 86400;
DELETE m FROM chat_messages m
JOIN orders o ON o.id=m.order_id
JOIN settings s ON s.id=1
WHERE o.status IN ('completed','canceled')
AND o.updated_at < UNIX_TIMESTAMP() - s.chat_retention_days * 86400;
-- Locations are deleted transactionally on delivery or cancellation.
-- Financial/order/audit and account deletion requires the operator's legal retention policy.
