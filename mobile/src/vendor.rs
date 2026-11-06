use dioxus::prelude::*;
use crate::images;
use crate::UserRole;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VendorTab {
    Catalog,
    Orders,
    Profile,
}

#[derive(Clone, PartialEq, Debug)]
pub struct VendorItem {
    pub id: &'static str,
    pub name: &'static str,
    pub local_name: &'static str,
    pub price: u32,
    pub unit: &'static str,
    pub in_stock: bool,
    pub stock_qty: &'static str,
    pub img: &'static str,
}

#[component]
pub fn VendorPortalScreen(
    stall_name: String,
    stall_number: String,
    on_switch_role: EventHandler<UserRole>,
    on_logout: EventHandler<()>,
) -> Element {
    let mut current_tab = use_signal(|| VendorTab::Catalog);
    let mut toast_msg = use_signal(|| Option::<String>::None);
    let mut search_query = use_signal(|| String::new());
    let mut show_add_modal = use_signal(|| false);

    // Initial stall catalog
    let mut items = use_signal(|| vec![
        VendorItem {
            id: "tomatoes",
            name: "Fresh tomatoes",
            local_name: "Kamatis",
            price: 80,
            unit: "/ kg",
            in_stock: true,
            stock_qty: "18 kg",
            img: images::TOMATOES_IMG,
        },
        VendorItem {
            id: "pechay",
            name: "Native pechay",
            local_name: "Pechay",
            price: 60,
            unit: "/ kg",
            in_stock: false,
            stock_qty: "0 kg",
            img: images::CABBAGE_IMG,
        },
        VendorItem {
            id: "carrots",
            name: "Benguet carrots",
            local_name: "Karot",
            price: 120,
            unit: "/ kg",
            in_stock: true,
            stock_qty: "12 kg",
            img: images::CARROTS_IMG,
        },
        VendorItem {
            id: "eggplant",
            name: "Native talong",
            local_name: "Talong",
            price: 70,
            unit: "/ kg",
            in_stock: true,
            stock_qty: "15 kg",
            img: images::EGGPLANT_IMG,
        },
    ]);

    // Simple fields for new product modal
    let mut new_name = use_signal(|| String::new());
    let mut new_price = use_signal(|| "90".to_string());

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
            div { class: "portal-role-banner", style: "background: #115e59;",
                span { style: "font-size: 11px; font-weight: 800; color: #99f6e4; text-transform: uppercase; letter-spacing: 0.5px;",
                    "🏪 VENDOR PORTAL · NENA'S GULAYAN (V-24)"
                }
                div { class: "portal-quick-chips",
                    button {
                        class: "quick-chip-btn",
                        onclick: move |_| on_switch_role.call(UserRole::Customer),
                        "👤 Customer"
                    }
                    button {
                        class: "quick-chip-btn",
                        onclick: move |_| on_switch_role.call(UserRole::Delivery),
                        "🛵 Shopper"
                    }
                    button {
                        class: "quick-chip-btn active",
                        "🏪 Vendor"
                    }
                }
            }

            // Screen V08 Header
            div { class: "market-top-header",
                div { class: "market-header-left",
                    span { class: "market-screen-heading", "Your stall · Mobile web" }
                }
                button {
                    class: "market-help-icon",
                    onclick: move |_| toast_msg.set(Some("Bankerohan Wholesale Stall V-24 active.".into())),
                    "?"
                }
            }

            // Stall Identity Banner
            div { style: "padding: 0 16px;",
                p { style: "font-size: 11px; color: var(--text-muted); margin-bottom: 4px;",
                    "🔒 vendor.sukidavao.ph / catalog · 412 px web layout"
                }
                div { style: "display: flex; align-items: center; gap: 8px; margin-top: 4px;",
                    span { style: "font-size: 20px;", "🌿" }
                    strong { style: "font-size: 14px; color: var(--primary-dark); letter-spacing: 0.5px;", "SUKI DAVAO" }
                }
                h1 { style: "font-size: 26px; font-weight: 800; color: var(--primary-dark); margin: 6px 0 4px 0;",
                    "{stall_name}"
                }
                span { class: "tag-pill-green", "Accredited · {stall_number}" }
            }

            // Stats Metrics Grid (Today's Sales & Orders to prepare)
            div { class: "vendor-stat-grid",
                div { class: "vendor-stat-box",
                    span { class: "vendor-stat-label", "Today's stall sales" }
                    h2 { class: "vendor-stat-val", "₱4,820" }
                }
                div { class: "vendor-stat-box",
                    span { class: "vendor-stat-label", "To prepare" }
                    h2 { class: "vendor-stat-val", "6 orders" }
                }
            }

            // Subview Filter Pills
            div { class: "history-filter-bar", style: "padding: 0 16px 8px 16px;",
                button {
                    class: if current_tab() == VendorTab::Catalog { "stage-chip-btn active" } else { "stage-chip-btn" },
                    onclick: move |_| current_tab.set(VendorTab::Catalog),
                    "Catalog · 22"
                }
                button {
                    class: if current_tab() == VendorTab::Orders { "stage-chip-btn active" } else { "stage-chip-btn" },
                    onclick: move |_| current_tab.set(VendorTab::Orders),
                    "Orders · 6"
                }
                button {
                    class: if current_tab() == VendorTab::Profile { "stage-chip-btn active" } else { "stage-chip-btn" },
                    onclick: move |_| current_tab.set(VendorTab::Profile),
                    "Profile"
                }
            }

            match current_tab() {
                // =============================================================
                // Tab: Catalog (V08 Product & Stock Management)
                // =============================================================
                VendorTab::Catalog => rsx! {
                    div { style: "padding: 0 16px 10px 16px;",
                        label { class: "form-label", "Find a product" }
                        input {
                            class: "form-input",
                            placeholder: "Enter English or Bisaya name",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value()),
                        }
                    }

                    div { style: "padding: 0 16px 12px 16px;",
                        button {
                            class: "primary-btn",
                            style: "width: 100%;",
                            onclick: move |_| show_add_modal.set(true),
                            "+ Add product"
                        }
                    }

                    // Add Product Modal
                    if show_add_modal() {
                        div { class: "cant-find-callout", style: "margin: 0 16px 14px 16px; border: 2px solid var(--primary);",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark);", "Add produce to stall" }
                            div { class: "form-group", style: "margin-top: 8px;",
                                label { class: "form-label", "Produce Name" }
                                input {
                                    class: "form-input",
                                    placeholder: "e.g. Kalabasa / Squash",
                                    value: "{new_name}",
                                    oninput: move |e| new_name.set(e.value()),
                                }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Price (₱/kg)" }
                                input {
                                    class: "form-input",
                                    value: "{new_price}",
                                    oninput: move |e| new_price.set(e.value()),
                                }
                            }
                            div { style: "display: flex; gap: 8px; margin-top: 10px;",
                                button {
                                    class: "primary-btn",
                                    style: "flex: 1; padding: 8px;",
                                    onclick: move |_| {
                                        toast_msg.set(Some("Produce added to stall catalog!".into()));
                                        show_add_modal.set(false);
                                    },
                                    "Save Produce"
                                }
                                button {
                                    class: "btn-outline-danger",
                                    style: "flex: 1; padding: 8px;",
                                    onclick: move |_| show_add_modal.set(false),
                                    "Cancel"
                                }
                            }
                        }
                    }

                    // Product Cards
                    div { style: "padding: 0 16px;",
                        for (idx, p) in items().iter().enumerate() {
                            {
                                let is_in_stock = p.in_stock;
                                let p_price = p.price;
                                let p_name = p.name;
                                let p_local = p.local_name;
                                let p_stock = p.stock_qty;
                                let p_img = p.img;

                                rsx! {
                                    div { key: "{idx}", class: "vendor-product-card",
                                        div { style: "display: flex; gap: 12px; align-items: center;",
                                            img {
                                                src: "{p_img}",
                                                style: "width: 60px; height: 60px; border-radius: 10px; object-fit: cover;"
                                            }
                                            div { style: "flex: 1;",
                                                h4 { style: "font-size: 15px; font-weight: 800; color: var(--primary-dark); margin: 0;",
                                                    "{p_name} · {p_local}"
                                                }
                                                strong { style: "font-size: 14px; color: var(--text-main); display: block; margin: 2px 0;",
                                                    "₱{p_price} / kg"
                                                }
                                                if is_in_stock {
                                                    span { class: "stock-badge-pill in-stock", "In stock · {p_stock}" }
                                                } else {
                                                    span { class: "stock-badge-pill out-of-stock", "Out of stock" }
                                                }
                                            }
                                        }

                                        div { style: "display: flex; gap: 8px; margin-top: 12px;",
                                            button {
                                                class: "vendor-action-btn edit",
                                                onclick: move |_| {
                                                    toast_msg.set(Some(format!("Updated price for {} to ₱90/kg", p_name)));
                                                },
                                                "Edit product"
                                            }
                                            button {
                                                class: if is_in_stock { "vendor-action-btn stock toggle-out" } else { "vendor-action-btn stock toggle-in" },
                                                onclick: move |_| {
                                                    items.write()[idx].in_stock = !is_in_stock;
                                                    let status = if !is_in_stock { "IN STOCK" } else { "OUT OF STOCK" };
                                                    toast_msg.set(Some(format!("{} is now marked {}", p_name, status)));
                                                },
                                                if is_in_stock { "Mark Out of Stock" } else { "Mark In Stock" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    p { style: "font-size: 11px; color: var(--text-muted); text-align: center; margin: 16px 16px 24px 16px; line-height: 1.4;",
                        "Desktop table becomes stacked product cards; navigation moves to a compact bottom bar. All editing actions remain available."
                    }
                },

                // =============================================================
                // Tab: Orders (To Prepare for Couriers)
                // =============================================================
                VendorTab::Orders => rsx! {
                    div { style: "padding: 8px 16px;",
                        h3 { style: "font-size: 16px; font-weight: 800; color: var(--primary-dark);", "Incoming Shopper Pickups" }
                        p { style: "font-size: 12px; color: var(--text-muted);", "Orders being packed by personal shoppers at Bankerohan." }
                    }

                    // Order 1: Jun Mercado for Ana Santos
                    div { class: "history-card",
                        div { style: "display: flex; justify-content: space-between; align-items: center;",
                            strong { style: "font-size: 14px; color: var(--primary-dark);", "Order SKD-241108" }
                            span { class: "tag-pill-green", "Shopper Jun Mercado" }
                        }
                        p { style: "font-size: 12px; color: var(--text-muted); margin-top: 4px;", "Pickup stall: V-24 · Expected 9:50 AM" }

                        div { style: "border-top: 1px dashed var(--border-color); margin-top: 8px; padding-top: 8px; font-size: 13px;",
                            div { style: "display: flex; justify-content: space-between; padding: 2px 0;",
                                span { "1.00 kg Fresh tomatoes (Kamatis)" }
                                strong { "₱90.00" }
                            }
                            div { style: "display: flex; justify-content: space-between; padding: 2px 0;",
                                span { "0.50 kg Benguet carrots (Karot)" }
                                strong { "₱60.00" }
                            }
                        }

                        button {
                            class: "primary-btn",
                            style: "width: 100%; margin-top: 10px; font-size: 12px; padding: 8px;",
                            onclick: move |_| toast_msg.set(Some("Order SKD-241108 packed & ready for Jun!".into())),
                            "✓ Mark Prepared for Jun"
                        }
                    }

                    // Order 2: Next Shopper
                    div { class: "history-card",
                        div { style: "display: flex; justify-content: space-between; align-items: center;",
                            strong { style: "font-size: 14px; color: var(--primary-dark);", "Order SKD-241109" }
                            span { class: "tag-pill-green", style: "background: #f1f5f9; color: var(--text-main);", "Awaiting Shopper" }
                        }
                        p { style: "font-size: 12px; color: var(--text-muted); margin-top: 4px;", "2 kg Kalabasa + 1 kg Talong · ₱220.00" }
                    }
                },

                // =============================================================
                // Tab: Profile & Settings
                // =============================================================
                VendorTab::Profile => rsx! {
                    div { style: "padding: 8px 16px;",
                        div { class: "vendor-stall-card", style: "margin: 0 0 14px 0;",
                            h3 { style: "font-size: 16px; font-weight: 800; color: var(--primary-dark); margin: 0;", "Aling Nena's Gulayan" }
                            p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "Stall V-24 · Bankerohan Central Wholesale Terminal" }
                            span { style: "font-size: 11px; color: #16a34a; font-weight: 700; display: block; margin-top: 4px;", "✓ Certified Suki Palengke Vendor" }
                        }

                        // Switch Portal Directly
                        div { class: "history-card", style: "margin: 0 0 14px 0; border: 2px solid var(--primary); background: #f0fdf4;",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 6px;", "Switch Portal Directly" }
                            p { style: "font-size: 12px; color: var(--text-muted); margin-bottom: 10px;", "One-click access to test customer and delivery courier perspectives." }
                            div { style: "display: flex; gap: 8px;",
                                button {
                                    class: "primary-btn",
                                    style: "flex: 1; padding: 10px; font-size: 12px;",
                                    onclick: move |_| on_switch_role.call(UserRole::Customer),
                                    "👤 Customer"
                                }
                                button {
                                    class: "primary-btn",
                                    style: "flex: 1; padding: 10px; font-size: 12px; background: #047857;",
                                    onclick: move |_| on_switch_role.call(UserRole::Delivery),
                                    "🛵 Shopper"
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

        // Bottom Navigation for Vendor
        div { class: "bottom-nav",
            button {
                class: if current_tab() == VendorTab::Catalog { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| current_tab.set(VendorTab::Catalog),
                span { class: "nav-tab-icon", "📦" }
                span { "Catalog" }
            }
            button {
                class: if current_tab() == VendorTab::Orders { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| current_tab.set(VendorTab::Orders),
                span { class: "nav-tab-icon", "📋" }
                span { "Orders" }
            }
            button {
                class: if current_tab() == VendorTab::Profile { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| current_tab.set(VendorTab::Profile),
                span { class: "nav-tab-icon", "🏪" }
                span { "Profile" }
            }
        }
    }
}

