# System architecture



Dioxus components own screen state through signals. The native transport keeps its access cookie in memory and restores the session using a refresh credential protected by Android Keystore or iOS Keychain. The browser uses same-origin HttpOnly cookies. WebSocket handshakes require a valid access token and exact Origin; established sockets recheck the underlying device session and order permission so rotation does not disconnect an active delivery. Navigating away stops the subscription. On reconnect the client recovers persisted messages by cursor and refreshes the order and courier location.


Axum handlers own authorization and transactions. SQLx connects to MariaDB 11.8 LTS using its MySQL driver. Existing migration names and contents are unchanged so an existing database can keep its migration history. The separate PostgreSQL foundation prototype was removed from the product tree.

The shared crate owns wire types and exact centavo arithmetic. New quotes combine road distance from Bankerohan Market and estimated basket weight. Within checkout the API revalidates availability, price, weight, vendor identity, and the minimum goods subtotal. Retries preserve the payload and idempotency key. GCash references require manual confirmation; COD is collected after goods reconciliation and before delivery is marked complete. For new orders, completion requires a recorded vendor settlement.

Vendor participation is derived from the order’s snapshotted vendor ID. It gives access to chat and the vendor’s own items without exposing the customer address, payment details, or GPS. The original migration files remain unchanged; migration 0003 adds marketplace fields, preserves existing orders, and gives older product rows a default selling unit weight of 1 kg. Vendors should review that weight before offering existing listings.

