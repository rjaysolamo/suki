use dioxus::prelude::*;
use crate::images;
use crate::UserRole;
use crate::street_map::{DavaoStreetMap, MapMode};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ShopperSubView {
    Dashboard,              // R01: Courier/Shopper dashboard
    AssignedOrder,          // R02: Assigned & paid orders
    ShoppingChecklist,      // R03: Stall shopping checklist
    RecordWeight,           // R04: Actual weight & price (Scale input)
    UnavailableReplacement, // R05: Unavailable / replacement
    PriceAdjustment,        // R06: Price adjustment request
    Reconciliation,         // R07: Shopping reconciliation
    DeliveryGpsRoute,       // R08: Delivery & GPS sharing
    CustomerCommunication,  // R09: Customer communication (Chat)
    DeliveryCompletion,     // R10: Delivery completion (Proof of delivery)
    Account,                // Shopper account & settings
}

#[component]
pub fn ShopperPortalScreen(
    shopper_name: String,
    on_switch_role: EventHandler<UserRole>,
    on_logout: EventHandler<()>,
) -> Element {
    let mut subview = use_signal(|| ShopperSubView::Dashboard);
    let mut toast_msg = use_signal(|| Option::<String>::None);

    // Interactive form states for shopper actions
    let mut actual_weight = use_signal(|| "1.40".to_string());
    let mut current_market_rate = use_signal(|| "250.00".to_string());
    let mut weight_reason = use_signal(|| "Stall rate increased; actual portion weighs 1.40 kg.".to_string());
    let mut scale_photo_attached = use_signal(|| true);

    let mut sub_item_name = use_signal(|| "Fresh kangkong · 1 bunch".to_string());
    let mut sub_price = use_signal(|| "25.00".to_string());
    let mut sub_msg = use_signal(|| "Fresh kangkong is available. Would you like 1 bunch instead?".to_string());

    let mut price_reason = use_signal(|| "Today's stall price is ₱90/kg; these ripe tomatoes weigh 1.00 kg.".to_string());

    let mut chat_input = use_signal(|| String::new());
    let mut chat_messages = use_signal(|| vec![
        ("Jun", "9:52 AM", "Maayong buntag, Ana! Kamatis is ₱90/kg today. I sent a price request for you to review."),
        ("Ana", "9:55 AM", "Salamat, Jun. I'll approve the tomatoes. Please skip the kangkong replacement."),
        ("Jun", "10:02 AM", "Ana, all set. Your groceries will arrive around 10:40–11:00 AM."),
    ]);

    let mut pod_received_by = use_signal(|| "Ana Santos - Customer".to_string());
    let mut pod_notes = use_signal(|| "4 purchased items handed over at green gate.".to_string());

    rsx! {
        // Floating Toast
        if let Some(msg) = toast_msg() {
            div { class: "toast-success",
                div { style: "display: flex; justify-content: space-between; align-items: center;",
                    span { "{msg}" }
                    button {
                        style: "background: none; border: none; font-size: 16px; font-weight: 700; cursor: pointer; color: #14532d;",
                        onclick: move |_| toast_msg.set(None),
                        "×"
                    }
                }
            }
        }

        div { class: "screen-scroll",
            // Role Quick-Switch Pill Bar for Developer / Demo Convenience ("Para daretso na lang")
            div { class: "portal-role-banner",
                span { style: "font-size: 11px; font-weight: 800; color: #a7f3d0; text-transform: uppercase; letter-spacing: 0.5px;",
                    "🛵 PERSONAL SHOPPER PORTAL · JUN MERCADO"
                }
                div { class: "portal-quick-chips",
                    button {
                        class: "quick-chip-btn",
                        onclick: move |_| on_switch_role.call(UserRole::Customer),
                        "👤 Customer"
                    }
                    button {
                        class: "quick-chip-btn active",
                        "🛵 Shopper"
                    }
                    button {
                        class: "quick-chip-btn",
                        onclick: move |_| on_switch_role.call(UserRole::Vendor),
                        "🏪 Vendor"
                    }
                }
            }

            match subview() {
                // =============================================================
                // R01: Courier / Shopper Dashboard
                // =============================================================
                ShopperSubView::Dashboard => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            span { class: "market-screen-heading", "Shopper dashboard" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("Davao Dispatch GPS active · Bankerohan Hub".into())),
                            "?"
                        }
                    }

                    div { class: "shopper-hero-card",
                        div { style: "display: flex; align-items: center; gap: 14px;",
                            div { class: "rider-avatar-circle", "🛵" }
                            div {
                                h2 { style: "font-size: 20px; font-weight: 800; color: #ffffff; margin: 0;", "Maayong buntag, Jun." }
                                p { style: "font-size: 12px; color: #a7f3d0; margin-top: 2px;", "Let's bring the market home." }
                                span { class: "shopper-status-badge", "● Online · Available for assignment" }
                            }
                        }
                    }

                    // Today at a glance
                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 10px;", "Today at a glance" }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 4px 0;",
                            span { "Completed market runs" }
                            strong { "3" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 4px 0;",
                            span { "Shopper earnings" }
                            strong { style: "color: #15803d;", "₱150.00" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 4px 0;",
                            span { "Operational expenses to review" }
                            strong { style: "color: #b45309;", "₱12.00" }
                        }
                    }

                    // Your Active Assignment
                    div { class: "assignment-active-card",
                        span { class: "tag-pill-green", style: "background: #fef3c7; color: #92400e;", "Paid & verified · 9:35 AM" }
                        h3 { style: "font-size: 17px; font-weight: 800; color: var(--primary-dark); margin-top: 8px;", "SKD-241108 · Ana Santos" }
                        p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "5 items · 3 stalls · Obrero, Davao City" }

                        div { style: "margin-top: 10px; font-size: 13px;",
                            div { style: "display: flex; justify-content: space-between; padding: 3px 0;",
                                span { "Verified grocery budget" }
                                strong { "₱650.00" }
                            }
                            div { style: "display: flex; justify-content: space-between; padding: 3px 0;",
                                span { "Fixed delivery · 4 km" }
                                strong { "₱74.00" }
                            }
                        }

                        button {
                            class: "primary-btn",
                            style: "width: 100%; margin-top: 12px;",
                            onclick: move |_| subview.set(ShopperSubView::AssignedOrder),
                            "Open assigned order →"
                        }
                    }

                    // Payment Guarantee Notice
                    div { class: "cant-find-callout", style: "margin: 0 16px 14px 16px; border-left: 4px solid var(--primary); background: #f0fdf4;",
                        strong { style: "color: var(--primary-dark);", "✓ Verified payment comes first" }
                        p { style: "margin-top: 4px; font-size: 11px; line-height: 1.3;",
                            "Only manually approved payments unlock assignments and shopping. Pending and rejected submissions are never shoppable."
                        }
                    }

                    button {
                        class: "btn-outline-danger",
                        style: "width: calc(100% - 32px); margin: 0 16px 24px 16px; border-color: var(--border-color); color: var(--text-main);",
                        onclick: move |_| subview.set(ShopperSubView::AssignedOrder),
                        "View today's assignments"
                    }
                },

                // =============================================================
                // R02: Assigned & Paid Orders
                // =============================================================
                ShopperSubView::AssignedOrder => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            button {
                                class: "market-back-btn",
                                onclick: move |_| subview.set(ShopperSubView::Dashboard),
                                "←"
                            }
                            span { class: "market-screen-heading", "Assigned & paid orders" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("SKD-241108 verified payment unlocked for shopping.".into())),
                            "?"
                        }
                    }

                    span { class: "order-stage-banner", "SKD-241108 · STAGE: ASSIGNED · 9:38 AM" }

                    div { style: "padding: 8px 16px 12px 16px;",
                        h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark);", "Your next market run" }
                        span { class: "status-pill-success", style: "margin-top: 4px;", "Payment manually verified" }
                    }

                    // Customer & Destination Card
                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        h4 { style: "font-size: 15px; font-weight: 800; color: var(--primary-dark);", "SKD-241108 · Ana Santos" }
                        p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "0917 ••• 4826 · Delivery today" }
                        p { style: "font-size: 12px; font-weight: 600; color: var(--text-main); margin-top: 6px;", "18 N. Torres St., Brgy. Obrero, Davao City" }
                        span { style: "font-size: 11px; color: #16a34a; font-weight: 700;", "Green gate near the sari-sari store" }

                        div { style: "border-top: 1px dashed var(--border-color); margin-top: 10px; padding-top: 8px; font-size: 12px;",
                            div { style: "display: flex; justify-content: space-between; padding: 2px 0;",
                                span { "Payment reference" }
                                strong { "1234567890123" }
                            }
                            div { style: "display: flex; justify-content: space-between; padding: 2px 0;",
                                span { "Verified total" }
                                strong { "₱724.00" }
                            }
                            div { style: "display: flex; justify-content: space-between; padding: 2px 0;",
                                span { "Available grocery funds" }
                                span { "₱650.00" }
                            }
                            div { style: "display: flex; justify-content: space-between; padding: 2px 0;",
                                span { "Reserved fixed delivery" }
                                span { "₱74.00" }
                            }
                        }
                    }

                    // Stall Route
                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 8px;", "Your stall route" }
                        div { class: "route-step-item",
                            span { class: "route-num-badge", "1" }
                            div {
                                strong { "Aling Nena's Gulayan · V-24" }
                                span { style: "font-size: 11px; color: var(--text-muted); display: block;", "3 items · Kamatis, Karot, Pechay" }
                            }
                        }
                        div { class: "route-step-item",
                            span { class: "route-num-badge", "2" }
                            div {
                                strong { "Dading's Poultry · M-08" }
                                span { style: "font-size: 11px; color: var(--text-muted); display: block;", "Chicken breast (needs weighing)" }
                            }
                        }
                        div { class: "route-step-item",
                            span { class: "route-num-badge", "3" }
                            div {
                                strong { "Apo Fruit Stand · F-12" }
                                span { style: "font-size: 11px; color: var(--text-muted); display: block;", "Lakatan bananas" }
                            }
                        }
                        p { style: "font-size: 11px; color: #16a34a; font-weight: 700; margin-top: 8px;", "All stalls are SUKI accredited." }
                    }

                    div { class: "cant-find-callout", style: "margin: 0 16px 12px 16px;",
                        strong { "ⓘ Ask before you buy" }
                        p { style: "margin-top: 4px; line-height: 1.3;",
                            "Record actual weight and price. Every price increase and replacement requires customer approval plus enough verified funds."
                        }
                    }

                    button {
                        class: "primary-btn",
                        style: "width: calc(100% - 32px); margin: 0 16px 8px 16px;",
                        onclick: move |_| subview.set(ShopperSubView::ShoppingChecklist),
                        "Start market shopping →"
                    }

                    button {
                        class: "btn-outline-danger",
                        style: "width: calc(100% - 32px); margin: 0 16px 24px 16px; border-color: var(--border-color); color: var(--text-main);",
                        onclick: move |_| toast_msg.set(Some("Connected to Davao Central Dispatch: 0917-555-SUKI".into())),
                        "Contact dispatch"
                    }
                },

                // =============================================================
                // R03: Stall Shopping Checklist
                // =============================================================
                ShopperSubView::ShoppingChecklist => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            button {
                                class: "market-back-btn",
                                onclick: move |_| subview.set(ShopperSubView::AssignedOrder),
                                "←"
                            }
                            span { class: "market-screen-heading", "Market shopping checklist" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("Check off items as stalls package them.".into())),
                            "?"
                        }
                    }

                    span { class: "order-stage-banner", "SKD-241108 · STAGE: SHOPPING · 10:04 AM" }

                    div { style: "padding: 8px 16px 4px 16px;",
                        h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark);", "Stall by stall." }
                        p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "4 purchased · 1 unavailable · 0 unresolved decisions" }
                    }

                    // Stall 1: Aling Nena's Gulayan · V-24
                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark);", "Aling Nena's Gulayan · V-24" }
                            span { class: "tag-pill-green", "Accredited · Vegetables" }
                        }

                        div { class: "checklist-row",
                            div {
                                strong { "✓ Kamatis · 1.00 kg = ₱90" }
                                span { style: "font-size: 11px; color: #16a34a; display: block;", "Purchased · Increase approved 9:56" }
                            }
                            strong { "₱90" }
                        }

                        div { class: "checklist-row",
                            div {
                                strong { "✓ Karot · 0.50 kg = ₱120" }
                                span { style: "font-size: 11px; color: var(--text-muted); display: block;", "Purchased · Unchanged" }
                            }
                            strong { "₱60" }
                        }

                        div { class: "checklist-row", style: "opacity: 0.7; background: #fff1f2; padding: 6px; border-radius: 8px;",
                            div {
                                span { style: "text-decoration: line-through;", "✕ Native pechay · 0.50 kg" }
                                span { style: "font-size: 11px; color: #e11d48; display: block;", "Unavailable · Kangkong rejected 9:58" }
                            }
                            strong { "₱0" }
                        }

                        div { style: "display: flex; gap: 8px; margin-top: 10px;",
                            button {
                                style: "flex: 1; background: #f1f5f9; border: 1px solid var(--border-color); border-radius: 8px; padding: 6px 8px; font-size: 11px; font-weight: 700; cursor: pointer;",
                                onclick: move |_| subview.set(ShopperSubView::PriceAdjustment),
                                "Request Price Adjust (R06)"
                            }
                            button {
                                style: "flex: 1; background: #f1f5f9; border: 1px solid var(--border-color); border-radius: 8px; padding: 6px 8px; font-size: 11px; font-weight: 700; cursor: pointer;",
                                onclick: move |_| subview.set(ShopperSubView::UnavailableReplacement),
                                "Offer Substitute (R05)"
                            }
                        }
                    }

                    // Stall 2: Dading's Poultry · M-08
                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark);", "Dading's Poultry · M-08" }
                            span { class: "tag-pill-green", "Accredited · Meat" }
                        }

                        div { class: "checklist-row",
                            div {
                                strong { "✓ Manok · 1.40 kg = ₱250" }
                                span { style: "font-size: 11px; color: #16a34a; display: block;", "Purchased · Actual weight recorded 10:02" }
                            }
                            strong { "₱350" }
                        }

                        button {
                            style: "width: 100%; margin-top: 8px; background: #f8fafc; border: 1px dashed var(--primary); color: var(--primary); border-radius: 8px; padding: 7px; font-size: 11px; font-weight: 700; cursor: pointer;",
                            onclick: move |_| subview.set(ShopperSubView::RecordWeight),
                            "⚖️ Weigh Scale Input (R04) →"
                        }
                    }

                    // Stall 3: Apo Fruit Stand · F-12
                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark);", "Apo Fruit Stand · F-12" }
                            span { class: "tag-pill-green", "Accredited · Fruits" }
                        }

                        div { class: "checklist-row",
                            div {
                                strong { "✓ Lakatan · 0.75 kg = ₱120" }
                                span { style: "font-size: 11px; color: #16a34a; display: block;", "Purchased · Actual weight recorded" }
                            }
                            strong { "₱90" }
                        }
                    }

                    div { class: "cant-find-callout", style: "margin: 0 16px 12px 16px;",
                        strong { "ⓘ Nothing pending, nothing extra" }
                        p { style: "margin-top: 4px; font-size: 11px; line-height: 1.3;",
                            "Kangkong was rejected and is not in the basket. Actual groceries: ₱590. No unapproved purchase or top-up."
                        }
                    }

                    button {
                        class: "primary-btn",
                        style: "width: calc(100% - 32px); margin: 0 16px 24px 16px;",
                        onclick: move |_| subview.set(ShopperSubView::Reconciliation),
                        "Review reconciliation →"
                    }
                },

                // =============================================================
                // R04: Actual Weight & Price (Scale Input)
                // =============================================================
                ShopperSubView::RecordWeight => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            button {
                                class: "market-back-btn",
                                onclick: move |_| subview.set(ShopperSubView::ShoppingChecklist),
                                "←"
                            }
                            span { class: "market-screen-heading", "Record actual weight & price" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("Enter precise scale reading at vendor stall.".into())),
                            "?"
                        }
                    }

                    span { class: "order-stage-banner", "SKD-241108 · STAGE: WEIGHING · 10:00 AM" }

                    div { style: "padding: 12px 16px;",
                        div { style: "display: flex; align-items: center; gap: 12px;",
                            img {
                                src: "{images::CHICKEN_IMG}",
                                style: "width: 56px; height: 56px; border-radius: 12px; object-fit: cover;"
                            }
                            div {
                                h2 { style: "font-size: 18px; font-weight: 800; color: var(--primary-dark); margin: 0;", "Chicken breast Manok" }
                                span { style: "font-size: 12px; color: var(--text-muted);", "Dading's Poultry · Stall M-08 · Accredited" }
                            }
                        }
                    }

                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        h4 { style: "font-size: 13px; font-weight: 700; color: var(--text-muted); text-transform: uppercase;", "Original estimate" }
                        div { style: "display: flex; justify-content: space-between; align-items: center; margin-top: 4px;",
                            span { "1.50 kg × ₱240.00/kg" }
                            strong { style: "font-size: 16px;", "₱360.00" }
                        }
                    }

                    // Interactive Inputs
                    div { style: "padding: 0 16px;",
                        div { class: "form-group",
                            label { class: "form-label", "Actual measured weight (kg)" }
                            input {
                                class: "form-input",
                                value: "{actual_weight}",
                                oninput: move |e| actual_weight.set(e.value()),
                            }
                            span { style: "font-size: 11px; color: var(--text-muted); display: block; margin-top: 2px;", "Enter the scale reading; up to 3 decimal places" }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Current market price (₱ / kg)" }
                            input {
                                class: "form-input",
                                value: "{current_market_rate}",
                                oninput: move |e| current_market_rate.set(e.value()),
                            }
                        }

                        // Calculated Total
                        div { class: "vendor-stall-card", style: "margin: 12px 0; background: #f0fdf4; border-color: #bbf7d0;",
                            div { style: "display: flex; justify-content: space-between; align-items: center;",
                                strong { style: "font-size: 16px; color: var(--primary-dark);", "1.40 × ₱250.00" }
                                strong { style: "font-size: 20px; color: var(--primary-dark);", "₱350.00" }
                            }
                            span { class: "tag-pill-green", style: "margin-top: 6px;", "Unit price +₱10/kg · Approval required" }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Reason / weighing note" }
                            input {
                                class: "form-input",
                                value: "{weight_reason}",
                                oninput: move |e| weight_reason.set(e.value()),
                            }
                        }

                        button {
                            style: "width: 100%; background: #ffffff; border: 1px dashed var(--border-color); border-radius: 12px; padding: 12px; font-weight: 700; color: var(--text-main); cursor: pointer; margin-bottom: 12px;",
                            onclick: move |_| {
                                scale_photo_attached.set(!scale_photo_attached());
                                toast_msg.set(Some("Scale photo toggled.".into()));
                            },
                            if scale_photo_attached() { "✓ Scale / stall receipt photo attached" } else { "+ Attach scale / stall receipt photo" }
                        }

                        div { class: "cant-find-callout", style: "border-left: 4px solid var(--primary); background: #f0fdf4;",
                            strong { "ⓘ Lower total still needs price approval" }
                            p { style: "margin-top: 4px; font-size: 11px; line-height: 1.3;",
                                "The unit price increased even though actual weight reduced the line total. Send the change to Ana before purchasing."
                            }
                        }

                        button {
                            class: "primary-btn",
                            style: "width: 100%; margin: 16px 0 24px 0;",
                            onclick: move |_| {
                                toast_msg.set(Some("Weight & price adjustment sent to Ana for approval!".into()));
                                subview.set(ShopperSubView::ShoppingChecklist);
                            },
                            "Send price & weight for approval →"
                        }
                    }
                },

                // =============================================================
                // R05: Unavailable & Replacement
                // =============================================================
                ShopperSubView::UnavailableReplacement => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            button {
                                class: "market-back-btn",
                                onclick: move |_| subview.set(ShopperSubView::ShoppingChecklist),
                                "←"
                            }
                            span { class: "market-screen-heading", "Unavailable & replacement" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("Shoppers offer substitutions from accredited stalls only.".into())),
                            "?"
                        }
                    }

                    span { class: "order-stage-banner", "SKD-241108 · STAGE: SUBSTITUTION REQUEST · 9:57 AM" }

                    div { style: "padding: 8px 16px;",
                        span { style: "font-size: 11px; color: #e11d48; font-weight: 700; text-transform: uppercase;", "Native pechay · Unavailable" }
                        h1 { style: "font-size: 22px; font-weight: 800; color: var(--primary-dark); margin-top: 4px;",
                            "Offer an option. Wait for their choice."
                        }
                    }

                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        h4 { style: "font-size: 13px; color: var(--text-muted);", "Original · Native pechay" }
                        div { style: "display: flex; justify-content: space-between; margin-top: 4px;",
                            span { "0.50 kg × ₱60/kg" }
                            strong { "₱30.00" }
                        }
                        p { style: "font-size: 11px; color: #b45309; margin-top: 4px;", "No stock at accredited vegetable stalls." }
                    }

                    div { style: "padding: 0 16px;",
                        div { class: "form-group",
                            label { class: "form-label", "Suggested replacement" }
                            input {
                                class: "form-input",
                                value: "{sub_item_name}",
                                oninput: move |e| sub_item_name.set(e.value()),
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Accredited vendor" }
                            input {
                                class: "form-input",
                                readonly: true,
                                value: "Aling Nena's Gulayan · V-24"
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Replacement line price (₱)" }
                            input {
                                class: "form-input",
                                value: "{sub_price}",
                                oninput: move |e| sub_price.set(e.value()),
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Message to customer" }
                            input {
                                class: "form-input",
                                value: "{sub_msg}",
                                oninput: move |e| sub_msg.set(e.value()),
                            }
                        }

                        button {
                            class: "primary-btn",
                            style: "width: 100%; margin-top: 8px;",
                            onclick: move |_| {
                                toast_msg.set(Some("Substitution proposal sent to Ana Santos!".into()));
                                subview.set(ShopperSubView::ShoppingChecklist);
                            },
                            "Send substitution request →"
                        }

                        div { class: "outcome-row-card", style: "margin-top: 14px; margin-bottom: 24px;",
                            strong { style: "font-size: 12px; color: var(--primary-dark); display: block; margin-bottom: 4px;", "Request decision states" }
                            span { style: "font-size: 11px; color: #b45309; font-weight: 700;", "9:57 · Waiting for customer" }
                            p { style: "font-size: 11px; color: var(--text-muted); margin-top: 2px;", "Do not purchase. Timeout at 10:12 → skip." }
                        }
                    }
                },

                // =============================================================
                // R06: Price Adjustment Request
                // =============================================================
                ShopperSubView::PriceAdjustment => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            button {
                                class: "market-back-btn",
                                onclick: move |_| subview.set(ShopperSubView::ShoppingChecklist),
                                "←"
                            }
                            span { class: "market-screen-heading", "Price adjustment request" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("Price changes require customer authorization.".into())),
                            "?"
                        }
                    }

                    span { class: "order-stage-banner", "SKD-241108 · STAGE: PRICE REVIEW · 9:52 AM" }

                    div { style: "padding: 8px 16px;",
                        h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark);", "Make the change clear." }
                    }

                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark);", "Tomatoes · Nena's Gulayan V-24" }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 3px 0; margin-top: 6px;",
                            span { "Estimated · 1 kg × ₱80" }
                            strong { "₱80.00" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 3px 0;",
                            span { "Proposed · 1 kg × ₱90" }
                            strong { "₱90.00" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 4px 0; border-top: 1px dashed var(--border-color); margin-top: 4px;",
                            strong { "Difference" }
                            strong { style: "color: #b45309;", "+₱10.00" }
                        }
                    }

                    div { style: "padding: 0 16px;",
                        div { class: "form-group",
                            label { class: "form-label", "Reason visible to Ana" }
                            input {
                                class: "form-input",
                                value: "{price_reason}",
                                oninput: move |e| price_reason.set(e.value()),
                            }
                        }

                        button {
                            class: "primary-btn",
                            style: "width: 100%; margin-top: 8px;",
                            onclick: move |_| {
                                toast_msg.set(Some("Price request sent to Ana (+₱10.00)!".into()));
                                subview.set(ShopperSubView::ShoppingChecklist);
                            },
                            "Send request to customer →"
                        }

                        div { class: "outcome-row-card", style: "margin-top: 14px; margin-bottom: 24px;",
                            strong { style: "font-size: 13px; color: var(--primary-dark); display: block; margin-bottom: 4px;", "Decision & funding gate" }
                            span { class: "tag-pill-green", style: "background: #fef3c7; color: #92400e;", "9:52 · Waiting for approval" }
                            p { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;",
                                "Purchase disabled while waiting. After approval, verify funds before buying."
                            }
                        }
                    }
                },

                // =============================================================
                // R07: Shopping Reconciliation
                // =============================================================
                ShopperSubView::Reconciliation => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            button {
                                class: "market-back-btn",
                                onclick: move |_| subview.set(ShopperSubView::ShoppingChecklist),
                                "←"
                            }
                            span { class: "market-screen-heading", "Shopping reconciliation" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("Every peso reconciled before motorcycle departure.".into())),
                            "?"
                        }
                    }

                    span { class: "order-stage-banner", "SKD-241108 · STAGE: READY FOR DELIVERY · 10:12 AM" }

                    div { style: "padding: 8px 16px;",
                        h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark);", "Every peso, recorded." }
                    }

                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        h4 { style: "font-size: 13px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; margin-bottom: 8px;", "Purchased / excluded" }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                            span { "Kamatis · 1.00 kg" }
                            strong { "₱90.00" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                            span { "Karot · 0.50 kg" }
                            strong { "₱60.00" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                            span { "Native pechay (excluded)" }
                            strong { "₱0.00" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                            span { "Manok · 1.40 kg" }
                            strong { "₱350.00" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                            span { "Lakatan bananas · 0.75 kg" }
                            strong { "₱90.00" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 15px; padding: 6px 0; border-top: 1px solid var(--border-color); margin-top: 6px; color: var(--primary-dark);",
                            strong { "Actual grocery cost" }
                            strong { "₱590.00" }
                        }
                    }

                    // Settlement breakdown
                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        h4 { style: "font-size: 13px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; margin-bottom: 8px;", "Customer settlement" }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                            span { "Verified funds" }
                            strong { "₱724.00" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                            span { "Groceries + fixed delivery" }
                            strong { "₱664.00" }
                        }
                        div { style: "display: flex; justify-content: space-between; font-size: 14px; padding: 4px 0; color: #15803d; border-top: 1px solid #f1f5f9; margin-top: 4px;",
                            strong { "Refundable excess" }
                            strong { "₱60.00" }
                        }
                        span { style: "font-size: 11px; color: var(--text-muted); display: block; margin-top: 4px;", "Automatically calculated · Pending processing, not yet sent" }
                    }

                    // Operational Expense
                    div { class: "vendor-stall-card", style: "margin: 0 16px 14px 16px;",
                        h4 { style: "font-size: 13px; font-weight: 700; color: var(--text-muted); text-transform: uppercase;", "Operational expense · Separate ledger" }
                        div { style: "display: flex; justify-content: space-between; margin-top: 6px; font-size: 13px;",
                            span { "Reusable carry bag · receipt attached" }
                            strong { "₱12.00" }
                        }
                        p { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;",
                            "Platform operating expense. Never added to Ana's bill. Shopper payout ₱50 is also recorded separately."
                        }
                    }

                    button {
                        class: "primary-btn",
                        style: "width: calc(100% - 32px); margin: 0 16px 24px 16px;",
                        onclick: move |_| subview.set(ShopperSubView::DeliveryGpsRoute),
                        "Confirm packing & begin delivery →"
                    }
                },

                // =============================================================
                // R08: Delivery & GPS Sharing
                // =============================================================
                ShopperSubView::DeliveryGpsRoute => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            button {
                                class: "market-back-btn",
                                onclick: move |_| subview.set(ShopperSubView::Reconciliation),
                                "←"
                            }
                            span { class: "market-screen-heading", "Delivery & GPS sharing" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("GPS route active: Bankerohan to Obrero.".into())),
                            "?"
                        }
                    }

                    span { class: "order-stage-banner", "SKD-241108 · STAGE: DELIVERY · 10:31 AM" }
                    span { class: "status-pill-success", "● GPS sharing on · Last sample 10:31" }

                    div { style: "padding: 8px 16px;",
                        h1 { style: "font-size: 26px; font-weight: 800; color: var(--primary-dark);", "Next stop: Ana's home." }
                    }

                    // Authentic Davao Street Map Route & Live Navigation
                    DavaoStreetMap {
                        mode: MapMode::Tracking,
                        shopper_name: shopper_name.clone(),
                        destination_label: "18 N. Torres St., Obrero".to_string(),
                        distance: "4.00 km".to_string(),
                        eta: "10:40–11:00 AM".to_string(),
                        is_shopper_view: true,
                    }

                    div { class: "vendor-stall-card", style: "margin: 8px 16px 12px 16px;",
                        h4 { style: "font-size: 15px; font-weight: 800; color: var(--primary-dark);", "18 N. Torres St., Obrero" }
                        p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "Brgy. Obrero, Davao City · Green gate near corner sari-sari store" }
                        div { style: "display: flex; justify-content: space-between; font-size: 12px; margin-top: 8px; border-top: 1px solid #f1f5f9; padding-top: 6px;",
                            span { "Driving distance: 4.00 km" }
                            strong { "Arrival: 10:40–11:00 AM" }
                        }
                    }

                    div { style: "display: flex; gap: 8px; padding: 0 16px 12px 16px;",
                        button {
                            style: "flex: 1; background: #ffffff; border: 1px solid var(--border-color); border-radius: 12px; padding: 10px; font-weight: 700; color: var(--primary-dark); cursor: pointer; font-size: 12px;",
                            onclick: move |_| subview.set(ShopperSubView::CustomerCommunication),
                            "💬 Chat with Ana"
                        }
                        button {
                            style: "flex: 1; background: #ffffff; border: 1px solid var(--border-color); border-radius: 12px; padding: 10px; font-weight: 700; color: var(--primary-dark); cursor: pointer; font-size: 12px;",
                            onclick: move |_| toast_msg.set(Some("Calling Ana Santos (0917-555-4826)...".into())),
                            "📞 Call Ana"
                        }
                    }

                    button {
                        class: "primary-btn",
                        style: "width: calc(100% - 32px); margin: 0 16px 24px 16px;",
                        onclick: move |_| subview.set(ShopperSubView::DeliveryCompletion),
                        "I've arrived at the doorstep →"
                    }
                },

                // =============================================================
                // R09: Customer Communication (Chat)
                // =============================================================
                ShopperSubView::CustomerCommunication => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            button {
                                class: "market-back-btn",
                                onclick: move |_| subview.set(ShopperSubView::DeliveryGpsRoute),
                                "←"
                            }
                            span { class: "market-screen-heading", "Customer communication" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("Encrypted courier messaging".into())),
                            "?"
                        }
                    }

                    span { class: "order-stage-banner", "ANA SANTOS · SKD-241108 · 0917 ••• 4826" }

                    div { class: "chat-container",
                        for (sender, time, text) in chat_messages().iter() {
                            div {
                                class: if *sender == "Jun" { "chat-bubble-customer" } else { "chat-bubble-shopper" },
                                p { "{text}" }
                                span {
                                    class: if *sender == "Jun" { "chat-timestamp-customer" } else { "chat-timestamp-shopper" },
                                    "{sender} · {time}"
                                }
                            }
                        }
                    }

                    // Chat Input Bar
                    div { class: "chat-input-bar",
                        input {
                            class: "chat-input-field",
                            placeholder: "✏️ Message Ana Santos...",
                            value: "{chat_input}",
                            oninput: move |e| chat_input.set(e.value()),
                        }
                        button {
                            class: "btn-chat-send",
                            onclick: move |_| {
                                let txt = chat_input().trim().to_string();
                                if !txt.is_empty() {
                                    chat_messages.write().push(("Jun", "Just now", Box::leak(txt.into_boxed_str())));
                                    chat_input.set(String::new());
                                }
                            },
                            "➤"
                        }
                    }
                },

                // =============================================================
                // R10: Delivery Completion (Proof of Delivery)
                // =============================================================
                ShopperSubView::DeliveryCompletion => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            button {
                                class: "market-back-btn",
                                onclick: move |_| subview.set(ShopperSubView::DeliveryGpsRoute),
                                "←"
                            }
                            span { class: "market-screen-heading", "Delivery completion" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("Order completion signs off customer settlement.".into())),
                            "?"
                        }
                    }

                    span { class: "order-stage-banner", "SKD-241108 · STAGE: DELIVERED · 10:48 AM" }
                    span { class: "status-pill-success", "Delivered successfully" }

                    div { style: "padding: 8px 16px;",
                        h1 { style: "font-size: 26px; font-weight: 800; color: var(--primary-dark);", "Fresh finds, safely home." }
                    }

                    div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                        h4 { style: "font-size: 13px; font-weight: 700; color: var(--text-muted); text-transform: uppercase;", "Proof of delivery" }
                        div { style: "display: flex; align-items: center; gap: 10px; margin-top: 8px; padding: 10px; background: #f0fdf4; border-radius: 10px;",
                            span { style: "font-size: 24px;", "📷" }
                            div {
                                strong { style: "font-size: 12px; color: var(--primary-dark); display: block;", "POD-241108.jpg Doorstep photo · 10:48 AM" }
                                span { style: "font-size: 11px; color: #16a34a;", "Upload complete · 1.2 MB" }
                            }
                        }
                    }

                    div { style: "padding: 0 16px;",
                        div { class: "form-group",
                            label { class: "form-label", "Received by" }
                            input {
                                class: "form-input",
                                value: "{pod_received_by}",
                                oninput: move |e| pod_received_by.set(e.value()),
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Delivery notes" }
                            input {
                                class: "form-input",
                                value: "{pod_notes}",
                                oninput: move |e| pod_notes.set(e.value()),
                            }
                        }

                        // Order Closed Card
                        div { class: "vendor-stall-card", style: "margin: 12px 0;",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark);", "Order closed" }
                            div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                                span { "Final charged" }
                                strong { "₱664.00" }
                            }
                            div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                                span { "Refundable excess" }
                                strong { style: "color: #15803d;", "₱60.00" }
                            }
                            div { style: "border-top: 1px dashed var(--border-color); margin-top: 6px; padding-top: 6px; display: flex; justify-content: space-between; font-size: 13px;",
                                span { "Shopper payout" }
                                strong { style: "color: #15803d;", "₱50.00" }
                            }
                            div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                                span { "Expense claim · under review" }
                                strong { "₱12.00" }
                            }
                        }

                        button {
                            class: "primary-btn",
                            style: "width: 100%; margin: 8px 0 24px 0;",
                            onclick: move |_| {
                                toast_msg.set(Some("Order SKD-241108 signed off! Great run, Jun.".into()));
                                subview.set(ShopperSubView::Dashboard);
                            },
                            "Back to shopper dashboard"
                        }
                    }
                },

                // =============================================================
                // Account Tab for Shopper
                // =============================================================
                ShopperSubView::Account => rsx! {
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            span { class: "market-screen-heading", "Courier account" }
                        }
                    }

                    div { style: "padding: 8px 16px;",
                        div { class: "vendor-stall-card", style: "margin: 0 0 14px 0;",
                            div { style: "display: flex; align-items: center; gap: 14px;",
                                div { class: "rider-avatar-circle", "🛵" }
                                div {
                                    h3 { style: "font-size: 17px; font-weight: 800; color: var(--primary-dark); margin: 0;", "Jun Mercado" }
                                    p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "★ 4.9 · Accredited Suki Courier" }
                                    span { style: "font-size: 11px; color: var(--primary); font-weight: 700;", "Motorcycle Plate: 1108-DX" }
                                }
                            }
                        }

                        // Switch Portal Directly
                        div { class: "history-card", style: "margin: 0 0 14px 0; border: 2px solid var(--primary); background: #f0fdf4;",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 6px;", "Switch Portal Directly" }
                            p { style: "font-size: 12px; color: var(--text-muted); margin-bottom: 10px;", "One-click access to test customer and vendor perspectives." }
                            div { style: "display: flex; gap: 8px;",
                                button {
                                    class: "primary-btn",
                                    style: "flex: 1; padding: 10px; font-size: 12px;",
                                    onclick: move |_| on_switch_role.call(UserRole::Customer),
                                    "👤 Customer"
                                }
                                button {
                                    class: "primary-btn",
                                    style: "flex: 1; padding: 10px; font-size: 12px; background: #0f766e;",
                                    onclick: move |_| on_switch_role.call(UserRole::Vendor),
                                    "🏪 Vendor"
                                }
                            }
                        }

                        button {
                            class: "btn-logout",
                            style: "width: 100%; margin-top: 8px;",
                            onclick: move |_| on_logout.call(()),
                            "Log out"
                        }
                    }
                },
            }
        }

        // Bottom Navigation Bar for Personal Shopper
        div { class: "bottom-nav",
            button {
                class: if subview() == ShopperSubView::Dashboard { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| subview.set(ShopperSubView::Dashboard),
                span { class: "nav-tab-icon", "🏠" }
                span { "Home" }
            }
            button {
                class: if subview() == ShopperSubView::ShoppingChecklist || subview() == ShopperSubView::AssignedOrder { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| subview.set(ShopperSubView::ShoppingChecklist),
                span { class: "nav-tab-icon", "📋" }
                span { "Checklist" }
            }
            button {
                class: if subview() == ShopperSubView::CustomerCommunication { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| subview.set(ShopperSubView::CustomerCommunication),
                span { class: "nav-tab-icon", "💬" }
                span { "Chat" }
            }
            button {
                class: if subview() == ShopperSubView::Account { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| subview.set(ShopperSubView::Account),
                span { class: "nav-tab-icon", "👤" }
                span { "Account" }
            }
        }
    }
}
