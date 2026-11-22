use crate::{
    Api, State,
    api::{Live, delay},
    now,
};
use dioxus::prelude::*;
use futures_util::{FutureExt, StreamExt};
use reqwest::Method;
use suki_shared::{
    Acknowledged,
    auth::Role,
    orders::{Decision, Order, OrderDetail, Status},
    payments::{Evidence, EvidenceResult},
    pricing::peso,
    realtime::{
        ChatMessage, CourierLocation, LocationSnapshot, MessageInput, OrderEvent, Position,
        PositionResult,
    },
    snapshots::OrderSnapshot,
};

#[component]
pub fn OrderList(messages_only: bool) -> Element {
    let api = use_context::<Api>();
    let mut state = use_context::<State>();
    let mut orders = use_signal(Vec::<Order>::new);
    let mut error = use_signal(String::new);
    let mut loading = use_signal(|| true);
    let mut search = use_signal(String::new);
    let mut filter = use_signal(|| "all");
    let signed_in = state.account.read().is_some();
    let rider = state
        .account
        .read()
        .as_ref()
        .is_some_and(|u| u.has_role(Role::Rider));
    let list_api = api.clone();
    let refresh = move || {
        let api = list_api.clone();
        spawn(async move {
            loading.set(true);
            match api.get("/orders").await {
                Ok(items) => {
                    orders.set(items);
                    error.set(String::new());
                }
                Err(failure) => error.set(failure.to_string()),
            }
            loading.set(false);
        });
    };
    let first = refresh.clone();
    use_effect(move || {
        if state.account.read().is_some() {
            first();
        } else {
            loading.set(false);
        }
    });
    let term = search().trim().to_lowercase();
    let visible = orders
        .read()
        .iter()
        .filter(|order| {
            let matches_search =
                order.id.to_lowercase().contains(&term) || label(&order.status).contains(&term);
            let matches_filter = match filter() {
                "active" => !matches!(
                    order.status.as_str(),
                    "completed" | "delivered" | "canceled"
                ),
                "delivered" => matches!(order.status.as_str(), "delivered" | "completed"),
                "canceled" => order.status == "canceled",
                _ => true,
            };
            matches_search && matches_filter
        })
        .cloned()
        .collect::<Vec<_>>();
    rsx! {
        section { class: "orders-page",
            div { class: "page-title",
                div { p { class: "eyebrow", if messages_only { "KEEP IN TOUCH" } else { "YOUR MARKET JOURNEY" } }
                    h1 { if messages_only { "Messages" } else if rider { "Assigned deliveries" } else { "My orders" } }
                }
                if signed_in { button { class: "quiet refresh-link", disabled: loading(), onclick: move |_| refresh(), "Refresh ↻" } }
            }
            if messages_only { p { class: "page-description", "Choose an order to open its conversation with the people helping you." } }
            if !signed_in {
                div { class: "empty state-empty", span { class: "empty-icon", if messages_only { "✉" } else { "▤" } }
                    h2 { "Let’s get you signed in" }
                    p { if messages_only { "Your order conversations will appear here." } else { "Sign in to see your orders and follow their progress." } }
                    button { class: "primary", onclick: move |_| state.page.set(crate::Page::Account), "Sign in" }
                }
            } else if loading() {
                p { class: "empty", "Loading orders…" }
            } else if !(error)().is_empty() {
                p { class: "notice", role: "alert", "{error}" }
            } else {
                if !messages_only {
                    div { class: "search-row order-search", span { class: "search-icon", aria_hidden: "true", "⌕" }
                        input { r#type: "search", aria_label: "Search orders", placeholder: "Search orders", value: "{search}", oninput: move |e| search.set(e.value()) }
                    }
                    div { class: "order-tabs", aria_label: "Filter orders",
                        for (value, title) in [("all", "All"), ("active", "In progress"), ("delivered", "Delivered"), ("canceled", "Canceled")] {
                            button { key: "{value}", aria_pressed: filter() == value, onclick: move |_| filter.set(value), "{title}" }
                        }
                    }
                }
                if visible.is_empty() {
                    div { class: "empty state-empty", span { class: "empty-icon", if messages_only { "✉" } else { "▤" } }
                        h2 { if messages_only { "No conversations yet" } else { "Nothing here yet" } }
                        p { if messages_only { "When you place an order, you can chat with your market team here." } else { "Your orders will appear here as they make their way to you." } }
                        if !rider && !messages_only { button { class: "primary", onclick: move |_| state.page.set(crate::Page::Market), "Explore the market" } }
                    }
                } else {
                    div { class: "order-cards",
                        for order in visible {
                            button { class: "order-link", key: "{order.id}",
                                onclick: move |_| state.active_order.set(Some(order.id.clone())),
                                span { class: "order-mark", if messages_only { "✉" } else { "▤" } }
                                span { class: "order-copy", strong { "Order {short(&order.id)}" } small { "{label(&order.status)}" } }
                                if !messages_only { strong { class: "order-total", "{peso(order.estimated_total_centavos)}" } }
                                span { aria_hidden: "true", "›" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn OrderScreen(id: String) -> Element {
    let api = use_context::<Api>();
    let mut state = use_context::<State>();
    let detail = use_signal(|| None::<OrderDetail>);
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut location = use_signal(|| None::<CourierLocation>);
    let mut connection = use_signal(|| "Connecting…");
    let problem = use_signal(String::new);
    let path = format!("/orders/{id}");
    let stream_path = path.clone();
    let stream_api = api.clone();
    use_future(move || {
        let api = stream_api.clone();
        let path = stream_path.clone();
        let id = id.clone();
        async move {
            let mut cursor = 0;
            let mut backoff = 1000;
            loop {
                connection.set("Connecting…");
                let mut events = api.events(&id);
                match snapshot(&api, &path, detail, location, problem).await {
                    Access::Denied => {
                        messages.write().clear();
                        connection.set("Session unavailable");
                        return;
                    }
                    Access::Retry | Access::Ready => {}
                }
                catch_up(&api, &path, &mut cursor, messages, problem).await;
                let mut tick = Box::pin(delay(10_000).fuse());
                loop {
                    let next = events.next().fuse();
                    futures_util::pin_mut!(next);
                    let event = futures_util::select! {
                        item = next => item,
                        () = tick => Some(Live::Event(OrderEvent::Resync)),
                    };
                    let Some(event) = event else {
                        break;
                    };
                    match event {
                        Live::Connected => {
                            connection.set("Live");
                            backoff = 1000;
                            catch_up(&api, &path, &mut cursor, messages, problem).await;
                        }
                        Live::Event(event) => {
                            if !matches!(event, OrderEvent::Resync) {
                                connection.set("Live");
                                backoff = 1000;
                            }
                            match event {
                                OrderEvent::Message { .. } => {
                                    catch_up(&api, &path, &mut cursor, messages, problem).await
                                }
                                OrderEvent::Location(position) => {
                                    if detail
                                        .peek()
                                        .as_ref()
                                        .is_some_and(|d| d.order.status == "out_for_delivery")
                                    {
                                        location.set(Some(position));
                                    } else {
                                        snapshot(&api, &path, detail, location, problem).await;
                                    }
                                }
                                OrderEvent::Status { .. } | OrderEvent::Resync => {
                                    if let Access::Denied =
                                        snapshot(&api, &path, detail, location, problem).await
                                    {
                                        messages.write().clear();
                                        connection.set("Session unavailable");
                                        return;
                                    }
                                    catch_up(&api, &path, &mut cursor, messages, problem).await;
                                    tick = Box::pin(delay(10_000).fuse());
                                }
                            }
                        }
                    }
                }
                connection.set("Reconnecting…");
                delay(backoff).await;
                backoff = (backoff * 2).min(15_000);
            }
        }
    });
    let data = (detail)();
    let position = (location)();
    let user = (state.account)();
    rsx! {
        button { class: "quiet", onclick: move |_| state.active_order.set(None), "← All orders" }
        if !(problem)().is_empty() { p { class: "notice", role: "alert", "{problem}" } }
        if let Some(data) = data {
            section { class: "panel",
                p { class: "eyebrow", "MARKET ORDER" }
                h1 { "Order {short(&data.order.id)}" }
                strong { class: "badge", "{label(&data.order.status)}" }
                if data.snapshot.is_none() {
                p { "{suki_shared::addresses::display_address_text(&data.order.address)}" }
                for item in data.items.iter() {
                    div { class: "line", key: "{item.id}",
                        span { "{item.name} × {quantity(item.quantity_milli)} {item.unit}" }
                        strong { "{peso(item.estimated_cost_centavos)}" }
                    }
                }
                div { class: "line", span { "Estimated goods" } strong { "{peso(data.order.goods_subtotal_centavos)}" } }
                div { class: "line", span { "Delivery" } strong { "{peso(data.order.delivery_fee_centavos)}" } }
                div { class: "line totals", span { "Estimated total" } strong { "{peso(data.order.estimated_total_centavos)}" } }
                }
                if let Some(final_goods) = data.order.final_goods_centavos {
                    p { "Final goods: {peso(final_goods)}" }
                }
            }
            if let Some(snapshot) = &data.snapshot {
                CheckoutReceipt { snapshot: snapshot.clone(), order_id: data.order.id.clone(), payment_status: data.order.payment_status.clone() }
            }
            Payment { data: data.clone(), path: path.clone(), detail }
            if let Some(account) = user.clone().filter(|u| u.has_role(Role::Admin) || u.has_role(Role::Rider)) {
                crate::staff::StaffOrder { data: data.clone(), account, detail }
            }
            if user.as_ref().is_some_and(|u| u.has_role(Role::Customer))
                && matches!(data.order.status.as_str(), "awaiting_payment" | "payment_verification" | "paid" | "confirmed")
                && !data.payments.iter().any(|p| p.status == "under_verification")
            {
                CancelOrder { path: path.clone(), detail }
            }
            if user.as_ref().is_some_and(|u| u.has_role(Role::Customer)) {
                for row in data.adjustments.iter().filter(|row| row.3 == "pending") {
                    Approval { key: "{row.0}", detail, path: path.clone(),
                        endpoint: format!("{path}/adjustments/{}", row.0),
                        title: format!("Spending approval: {}", peso(row.1)), description: row.2.clone() }
                }
                for row in data.substitutions.iter().filter(|row| row.5 == "pending") {
                    Approval { key: "{row.0}", detail, path: path.clone(),
                        endpoint: format!("{path}/substitutions/{}", row.0),
                        title: format!("Substitute: {}", row.2),
                        description: format!("{} at {} per unit", quantity(row.3), peso(row.4)) }
                }
            }
            section { class: "panel",
                div { class: "line", h2 { "Delivery timeline" } small { "{connection}" } }
                ol { class: "timeline",
                    for (index, (status, _, timestamp)) in data.history.iter().enumerate() {
                        li { key: "{index}", strong { "{label(status)}" } " · {davao_date(*timestamp)}" }
                    }
                }
                if data.order.status == "out_for_delivery" {
                    if let Some(position) = position {
                        p { "Courier location: {position.lat:.5}, {position.lng:.5}" }
                        p { class: "muted", "Reported {davao_date(position.reported_at)}"
                            if now() - position.reported_at > 15 { " · Last report is stale" }
                        }
                    } else {
                        p { "Waiting for a courier location report." }
                    }
                    if user.as_ref().is_some_and(|u| data.order.courier_id.as_deref() == Some(&u.id) && (u.has_role(Role::Rider) || u.has_role(Role::Admin))) {
                        CourierReport { path: path.clone() }
                    }
                }
            }
            if user.as_ref().is_some_and(|u| u.has_role(Role::Customer)) && data.order.status == "out_for_delivery" {
                crate::tracking::LiveMap { order: data.order.clone(), location }
            }
            if user.as_ref().is_some_and(|u| u.has_role(Role::Customer)) && matches!(data.order.status.as_str(), "delivered" | "completed") && data.order.delivered_at.is_some() {
                crate::tracking::Reviews { order: data.order.clone() }
            }
            Chat { path, messages }
        } else if (problem)().is_empty() {
            p { "Loading order…" }
        }
    }
}

enum Access {
    Ready,
    Retry,
    Denied,
}

async fn snapshot(
    api: &Api,
    path: &str,
    mut detail: Signal<Option<OrderDetail>>,
    mut location: Signal<Option<CourierLocation>>,
    mut problem: Signal<String>,
) -> Access {
    match api.get::<OrderDetail>(path).await {
        Ok(data) => {
            let active = data.order.status == "out_for_delivery";
            detail.set(Some(data));
            problem.set(String::new());
            if active {
                match api.get(&format!("{path}/location")).await {
                    Ok(LocationSnapshot::Report { position, .. }) => {
                        location.set(Some(position));
                    }
                    Ok(LocationSnapshot::Missing { .. }) => location.set(None),
                    Err(failure) if matches!(failure.status, Some(401 | 403 | 404)) => {
                        detail.set(None);
                        location.set(None);
                        problem.set(failure.to_string());
                        return Access::Denied;
                    }
                    Err(_) => location.set(None),
                }
            } else {
                location.set(None);
            }
            Access::Ready
        }
        Err(failure) => {
            problem.set(failure.to_string());
            if matches!(failure.status, Some(401 | 403 | 404)) {
                detail.set(None);
                location.set(None);
                Access::Denied
            } else {
                Access::Retry
            }
        }
    }
}

pub(crate) async fn catch_up(
    api: &Api,
    path: &str,
    cursor: &mut i64,
    mut messages: Signal<Vec<ChatMessage>>,
    mut problem: Signal<String>,
) {
    loop {
        match api
            .get::<Vec<ChatMessage>>(&format!("{path}/messages?after={cursor}"))
            .await
        {
            Ok(page) => {
                if page.iter().any(|m| !m.is_legacy() && !m.valid_digest()) {
                    problem.set("Chat message failed its integrity check.".into());
                    break;
                }
                let count = page.len();
                if let Some(last) = page.last() {
                    *cursor = last.id;
                    messages.write().extend(page);
                }
                if count < 200 {
                    break;
                }
            }
            Err(failure) => {
                if matches!(failure.status, Some(401 | 403 | 404)) {
                    messages.write().clear();
                }
                problem.set(failure.to_string());
                break;
            }
        }
    }
}

#[component]
pub(crate) fn Chat(path: String, messages: Signal<Vec<ChatMessage>>) -> Element {
    let api = use_context::<Api>();
    let mut text = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    rsx! {
        section { class: "panel",
            h2 { "Order chat" }
            div { class: "chat", role: "log",
                if messages.read().is_empty() { p { "No messages yet." } }
                for message in messages.read().iter() {
                    article { key: "{message.id}",
                        strong { "{message.sender_name} · " if message.sender_role == "courier" { "rider" } else { "{message.sender_role}" } } small { "{davao_date(message.created_at)}" }
                        p { "{message.content}" }
                        if message.is_legacy() { small { "Historical message · no integrity hash" } }
                    }
                }
            }
            form { onsubmit: move |event| {
                event.prevent_default();
                if busy() || text().trim().is_empty() { return; }
                busy.set(true);
                error.set(String::new());
                let body = MessageInput { content: text() };
                let api = api.clone();
                let endpoint = format!("{path}/messages");
                spawn(async move {
                    match api.send::<OrderEvent, _>(Method::POST, &endpoint, &body).await {
                        Ok(_) => text.set(String::new()),
                        Err(failure) => error.set(failure.to_string()),
                    }
                    busy.set(false);
                });
            },
                label { "Message" textarea { disabled: busy(), maxlength: "2000", value: "{text}", oninput: move |e| text.set(e.value()) } }
                button { class: "primary", disabled: busy() || text().trim().is_empty(), "Send message" }
            }
            if !(error)().is_empty() { p { role: "alert", "{error}" } }
        }
    }
}

#[component]
fn CheckoutReceipt(snapshot: OrderSnapshot, order_id: String, payment_status: String) -> Element {
    let valid = snapshot.valid_digest_for(&order_id);
    let receipt = &snapshot.payload;
    let delivery = &receipt.delivery;
    rsx! {
        section { class: "panel",
            h2 { "Checkout receipt" }
            if !valid {
                p { role: "alert", "This receipt failed its integrity check." }
            } else {
                strong { class: "badge", "Verified receipt" }
                p { "{suki_shared::addresses::display_address_text(&receipt.address)} · {davao_date(receipt.created_at)}" }
                for item in &receipt.items {
                    div { class: "line", key: "{item.item_id}",
                        span { "{item.name} · {item.vendor_name} · {quantity(item.quantity_milli)} {item.unit} × {peso(item.unit_price_centavos)}" }
                        strong { "{peso(item.line_total_centavos)}" }
                    }
                    small { "{item.unit_weight_grams} g per {item.unit}" }
                }
                div { class: "line", span { "Goods at checkout" } strong { "{peso(receipt.goods_subtotal_centavos)}" } }
                div { class: "line", span { "Road distance ({delivery.distance_meters} m)" } strong { "{peso(delivery.distance_charge_centavos())}" } }
                if let Some(weight) = delivery.weight_grams {
                    div { class: "line", span { "Weight charge ({weight} g)" } strong { "{peso(delivery.weight_charge_centavos())}" } }
                    if let Some(floor) = delivery.floor_centavos {
                        div { class: "line", span { "Bankerohan minimum ({peso(floor)}) top-up" } strong { "{peso(delivery.floor_top_up_centavos)}" } }
                    }
                } else {
                    div { class: "line", span { "Legacy delivery base fee" } strong { "{peso(delivery.legacy_base_centavos)}" } }
                }
                div { class: "line", span { "Delivery fee" } strong { "{peso(delivery.total_centavos)}" } }
                div { class: "line totals", span { "Locked checkout total" } strong { "{peso(receipt.total_centavos)}" } }
                p { if receipt.payment_method == suki_shared::payments::PaymentMethod::CashOnDelivery { "Cash on delivery" } else { "GCash" }
                    " · Current payment status: {label(&payment_status)}"
                }
                details { summary { "Receipt fingerprint" } code { "{snapshot.snapshot_hash}" } }
            }
        }
    }
}

#[component]
fn Payment(data: OrderDetail, path: String, mut detail: Signal<Option<OrderDetail>>) -> Element {
    let api = use_context::<Api>();
    let state = use_context::<State>();
    let mut reference = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let paid: i64 = data
        .payments
        .iter()
        .filter(|p| p.status == "paid")
        .filter_map(|p| p.received_amount_centavos)
        .sum();
    let due = data.order.authorized_goods_centavos + data.order.delivery_fee_centavos - paid;
    let can_pay = !data.order.is_cod()
        && due > 0
        && state
            .account
            .read()
            .as_ref()
            .is_some_and(|u| u.has_role(Role::Customer))
        && matches!(
            data.order.status.as_str(),
            "awaiting_payment" | "payment_verification" | "price_adjustment"
        )
        && !data
            .payments
            .iter()
            .any(|p| p.status == "under_verification");
    let qr = state
        .settings
        .read()
        .as_ref()
        .and_then(|settings| api.image(&settings.gcash_qr_url));
    rsx! {
        section { class: "panel",
            h2 { if data.order.is_cod() { "Cash on delivery" } else { "GCash payment" } }
            p { "Payment status: {label(&data.order.payment_status)}" }
            for payment in data.payments.iter() {
                div { class: "line", key: "{payment.id}", span { "{payment.reference} · {label(&payment.status)}" } strong { "{peso(payment.claimed_amount_centavos)}" } }
            }
            for (id, amount, status) in data.refunds.iter() {
                div { class: "line", key: "{id}", span { "Refund · {label(status)}" } strong { "{peso(*amount)}" } }
            }
            if data.order.is_cod() && data.order.payment_status == "cod_pending" {
                p { "Pay the assigned rider {peso(data.order.final_goods_centavos.unwrap_or(data.order.goods_subtotal_centavos) + data.order.delivery_fee_centavos)} at handoff." }
            }
            if can_pay {
                if let Some(number) = &data.order.gcash_number {
                    p { "Transfer to this vendor's GCash number: " strong { "{number}" } }
                } else if let Some(qr) = qr {
                    img { class: "qr", src: "{qr}", alt: "Operator GCash payment QR" }
                }
                p { "Outstanding: {peso(due)}. Submit the reference after paying. The rider or an administrator will review it." }
                form {
                    onsubmit: move |event| {
                        event.prevent_default();
                        if busy() { return; }
                        busy.set(true);
                        error.set(String::new());
                        let api = api.clone();
                        let path = path.clone();
                        let evidence = Evidence { reference: reference(), amount_centavos: due };
                        spawn(async move {
                            match api.send::<EvidenceResult, _>(Method::POST, &format!("{path}/payments"), &evidence).await {
                                Ok(_) => {
                                    reference.set(String::new());
                                    match api.get(&path).await {
                                        Ok(data) => detail.set(Some(data)),
                                        Err(failure) => error.set(failure.to_string()),
                                    }
                                }
                                Err(failure) => error.set(failure.to_string()),
                            }
                            busy.set(false);
                        });
                    },
                    label { "GCash reference" input { required: true, disabled: busy(), minlength: "6", maxlength: "100", value: "{reference}", oninput: move |e| reference.set(e.value()) } }
                    button { class: "primary", disabled: busy(), "Submit GCash reference" }
                }
            }
            if !(error)().is_empty() { p { role: "alert", "{error}" } }
        }
    }
}

#[component]
fn CancelOrder(path: String, mut detail: Signal<Option<OrderDetail>>) -> Element {
    let api = use_context::<Api>();
    let mut confirm = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    rsx! {
        section { class: "panel",
            if confirm() {
                p { "Cancel this order? If a payment was verified, the operator will record a refund." }
                div { class: "actions",
                    button { disabled: busy(), onclick: move |_| {
                        if busy() { return; }
                        busy.set(true);
                        let api = api.clone();
                        let path = path.clone();
                        spawn(async move {
                            let result = async {
                                api.send::<Acknowledged, _>(
                                    Method::PATCH, &format!("{path}/status"), &Status { status: "canceled".into() },
                                ).await?;
                                let updated = api.get(&path).await?;
                                detail.set(Some(updated));
                                Ok::<_, crate::api::Failure>(())
                            }.await;
                            if let Err(failure) = result { error.set(failure.to_string()); }
                            busy.set(false);
                        });
                    }, "Confirm cancellation" }
                    button { disabled: busy(), onclick: move |_| confirm.set(false), "Keep order" }
                }
            } else {
                button { onclick: move |_| confirm.set(true), "Cancel order" }
            }
            if !(error)().is_empty() { p { role: "alert", "{error}" } }
        }
    }
}

#[component]
fn Approval(
    mut detail: Signal<Option<OrderDetail>>,
    path: String,
    endpoint: String,
    title: String,
    description: String,
) -> Element {
    let api = use_context::<Api>();
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let mut decide = move |approved| {
        if busy() {
            return;
        }
        busy.set(true);
        let (api, path, endpoint) = (api.clone(), path.clone(), endpoint.clone());
        spawn(async move {
            match api
                .send::<Acknowledged, _>(Method::POST, &endpoint, &Decision { approved })
                .await
            {
                Ok(_) => match api.get(&path).await {
                    Ok(data) => detail.set(Some(data)),
                    Err(failure) => error.set(failure.to_string()),
                },
                Err(failure) => error.set(failure.to_string()),
            }
            busy.set(false);
        });
    };
    let mut decline = decide.clone();
    rsx! {
        section { class: "panel", h2 { "{title}" } p { "{description}" }
            div { class: "actions",
                button { class: "primary", disabled: busy(), onclick: move |_| decide(true), "Approve" }
                button { disabled: busy(), onclick: move |_| decline(false), "Decline" }
            }
            if !(error)().is_empty() { p { role: "alert", "{error}" } }
        }
    }
}

#[component]
fn CourierReport(path: String) -> Element {
    let api = use_context::<Api>();
    let mut lat = use_signal(String::new);
    let mut lng = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut sharing = use_signal(|| false);
    let location_api = api.clone();
    let order_id = path.trim_start_matches("/orders/").to_owned();
    use_resource(move || {
        let active = sharing();
        let (api, order_id) = (location_api.clone(), order_id.clone());
        async move {
            if !active {
                return;
            }
            let source = match crate::gps::Source::new() {
                Ok(source) => source,
                Err(error) => {
                    status.set(error);
                    sharing.set(false);
                    return;
                }
            };
            let sender = api.location_sender(&order_id);
            loop {
                match source.position().await {
                    Ok(position) => {
                        let body = serde_json::to_string(&position).expect("serializable position");
                        if sender.unbounded_send(body).is_err() {
                            status.set("Tracking disconnected. Please share again.".into());
                            sharing.set(false);
                            break;
                        }
                        status.set("Sharing rider location.".into());
                    }
                    Err(error) => status.set(error),
                }
                delay(5_000).await;
            }
        }
    });
    rsx! {
        button { r#type: "button", onclick: move |_| {
            let enabled = !sharing();
            sharing.set(enabled);
            status.set(if enabled { "Requesting device location…" } else { "Live location paused." }.into());
        }, if sharing() { "Stop live GPS" } else { "Share live GPS" } }
        form { onsubmit: move |event| {
            event.prevent_default();
            if busy() || sharing() { return; }
            let (Ok(lat), Ok(lng)) = (lat().parse::<f64>(), lng().parse::<f64>()) else {
                status.set("Enter valid coordinates.".into());
                return;
            };
            if !lat.is_finite() || !lng.is_finite() {
                status.set("Enter valid coordinates.".into());
                return;
            }
            let api = api.clone();
            let endpoint = format!("{path}/location");
            busy.set(true);
            spawn(async move {
                let response = api.send::<PositionResult, _>(Method::PUT, &endpoint, &Position { lat, lng, ..Default::default() }).await;
                status.set(response.map_or_else(|failure| failure.to_string(), |_| "Location reported.".into()));
                busy.set(false);
            });
        },
            div { class: "field-pair",
                label { "Courier latitude" input { required: true, inputmode: "decimal", value: "{lat}", oninput: move |e| lat.set(e.value()) } }
                label { "Courier longitude" input { required: true, inputmode: "decimal", value: "{lng}", oninput: move |e| lng.set(e.value()) } }
            }
            button { disabled: busy() || sharing(), "Report coordinates manually" }
            p { role: "status", "{status}" }
        }
    }
}

fn short(id: &str) -> String {
    id.chars().take(8).collect()
}

fn label(status: &str) -> String {
    status.replace('_', " ")
}

fn quantity(milli: i64) -> String {
    if milli % 1000 == 0 {
        (milli / 1000).to_string()
    } else {
        format!("{}.{:03}", milli / 1000, milli % 1000)
            .trim_end_matches('0')
            .into()
    }
}

pub(crate) fn davao_date(timestamp: i64) -> String {
    // Davao observes UTC+8 year-round.
    time::OffsetDateTime::from_unix_timestamp(timestamp)
        .map(|utc| {
            let local = utc.to_offset(time::macros::offset!(+8));
            local
                .format(time::macros::format_description!(
                    "[month repr:short] [day], [year] · [hour]:[minute]"
                ))
                .unwrap_or_default()
        })
        .unwrap_or_default()
}
