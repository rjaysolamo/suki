use dioxus::prelude::*;

mod images;
mod style;
mod shopper;
mod vendor;
mod street_map;

pub use shopper::*;
pub use vendor::*;
pub use street_map::*;

#[cfg(target_os = "android")]
mod platform;

fn main() {
    dioxus::launch(App);
}

// -----------------------------------------------------------------------------
// App State & Lifecycle Models
// -----------------------------------------------------------------------------
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UserRole {
    Customer,
    Delivery,
    Vendor,
}

#[derive(Clone, PartialEq, Debug)]
pub struct UserSession {
    pub role: UserRole,
    pub name: String,
    pub phone: String,
    pub address: String,
    pub barangay: String,
    pub suki_points: u32,
    pub stall_name: Option<String>,
    pub stall_number: Option<String>,
    pub vehicle_info: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AppScreen {
    Welcome,       // Initial Home / Welcome showcase with Login & Register entry
    Auth,          // Login & Register form (gated account creation)
    Marketplace,   // Customer Marketplace App (C06 - C25)
    ShopperPortal, // Personal Shopper / Delivery App (R01 - R10)
    VendorPortal,  // Vendor Stall Mobile Web / App (V08)
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MarketSubView {
    Catalog,             // C06: The market catalog
    BilingualSearch,     // C07: Bilingual search
    ProductDetail,       // C08: Fresh from the stall
    Basket,              // C09 (min met) / C10 (min unmet)
    SavedAddresses,      // C11: Saved delivery addresses
    AddressMapPicker,    // C12: Pin your doorstep
    ReviewCheckout,      // C13: Review & checkout
    GCashPayment,        // C14: Pay with GCash
    PaymentVerification, // C15: Payment verification
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum OrderScreenStage {
    DeliveryTracking,     // C19: Track your delivery
    ShopperChat,          // C21: Chat with Jun (encrypted order chat)
    PriceApproval,        // C16: Price approval (+₱10 tomatoes)
    SubstitutionApproval, // C17: Substitution approval (Pechay -> Kangkong)
    FundingTopUp,         // C18: Funding before purchase (+₱30 top-up)
    DigitalReceipt,       // C22: Your digital receipt (estimate vs actual)
    RefundNotification,   // C20: Your refund update (₱60.00 accounted for)
    OrderHistory,         // C23: Order history (Good food. Good memories)
    RecoveryStates,       // C25: First-time & recovery states
}

#[derive(Clone, PartialEq, Debug)]
pub struct CartItem {
    pub id: &'static str,
    pub name: &'static str,
    pub local_name: &'static str,
    pub vendor: &'static str,
    pub stall: &'static str,
    pub price: u32,
    pub quarters: u32, // in 0.25 kg steps
    pub img: &'static str,
}

impl CartItem {
    pub fn weight_str(&self) -> String {
        format!("{:.2} kg", self.quarters as f64 * 0.25)
    }

    pub fn item_total(&self) -> u32 {
        ((self.price as f64 * (self.quarters as f64 * 0.25)).round()) as u32
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Product {
    pub id: &'static str,
    pub name: &'static str,
    pub local_name: &'static str,
    pub vendor: &'static str,
    pub stall: &'static str,
    pub price: u32,
    pub unit: &'static str,
    pub category: &'static str,
    pub in_stock: bool,
    pub description: &'static str,
    pub img: &'static str,
}

const PRODUCTS: &[Product] = &[
    Product {
        id: "tomatoes",
        name: "Fresh tomatoes",
        local_name: "Kamatis",
        vendor: "Aling Nena's Gulayan",
        stall: "Stall V-24 · Bankerohan Public Market",
        price: 80,
        unit: "/ kg",
        category: "Vegetables",
        in_stock: true,
        description: "Kamatis · Ripe and juicy, perfect for gisa and sinigang. Picked with care, not pre-packed.",
        img: images::TOMATOES_IMG,
    },
    Product {
        id: "salad_tomatoes",
        name: "Salad tomatoes",
        local_name: "Kamatis",
        vendor: "Aling Nena's Gulayan",
        stall: "Stall V-24 · Bankerohan Public Market",
        price: 95,
        unit: "/ kg",
        category: "Vegetables",
        in_stock: true,
        description: "Kamatis · Firm, sweet salad tomatoes selected for ensalada and sandwiches.",
        img: images::TOMATOES_IMG,
    },
    Product {
        id: "carrots",
        name: "Benguet carrots",
        local_name: "Karot",
        vendor: "Aling Nena's Gulayan",
        stall: "Stall V-24 · Bankerohan Public Market",
        price: 120,
        unit: "/ kg",
        category: "Vegetables",
        in_stock: true,
        description: "Karot · Crisp highland carrots from Marilog, sweet and vibrant for chop suey.",
        img: images::CARROTS_IMG,
    },
    Product {
        id: "pechay",
        name: "Native pechay",
        local_name: "Pechay",
        vendor: "Aling Nena's Gulayan",
        stall: "Stall V-24 · Bankerohan Public Market",
        price: 60,
        unit: "/ kg",
        category: "Vegetables",
        in_stock: true,
        description: "Pechay · Fresh leafy greens harvested at dawn from Bankerohan greens section.",
        img: images::CABBAGE_IMG,
    },
    Product {
        id: "eggplant",
        name: "Long eggplant",
        local_name: "Talong",
        vendor: "Aling Nena's Gulayan",
        stall: "Stall V-24 · Bankerohan Public Market",
        price: 85,
        unit: "/ kg",
        category: "Vegetables",
        in_stock: true,
        description: "Talong · Tender native purple eggplants for tortang talong and pinakbet.",
        img: images::EGGPLANT_IMG,
    },
    Product {
        id: "chicken",
        name: "Fresh dressed chicken",
        local_name: "Manok",
        vendor: "Davao Poultry Stalls",
        stall: "Stall M-08 · Bankerohan Public Market",
        price: 240,
        unit: "/ kg",
        category: "Meat",
        in_stock: true,
        description: "Manok · Freshly dressed native poultry prepared under hygienic Davao veterinary standards.",
        img: images::CHICKEN_IMG,
    },
    Product {
        id: "tuna",
        name: "Yellowfin Tuna Steak",
        local_name: "Isda / Bariles",
        vendor: "Agdao Fish Port Dock",
        stall: "Stall S-12 · Agdao Public Market",
        price: 380,
        unit: "/ kg",
        category: "Seafood",
        in_stock: true,
        description: "Isda · First-class sashimi-grade yellowfin tuna from Davao Gulf morning catch.",
        img: images::FISH_IMG,
    },
    Product {
        id: "bananas",
        name: "Lakatan bananas",
        local_name: "Saging",
        vendor: "Toril Fruit Terminal",
        stall: "Stall F-15 · Toril Public Market",
        price: 60,
        unit: "/ kg",
        category: "Fruits",
        in_stock: true,
        description: "Saging · Sweet fragrant Lakatan bananas from Davao orchards.",
        img: images::BANANAS_IMG,
    },
    Product {
        id: "durian",
        name: "Davao Puyat Durian",
        local_name: "Durian",
        vendor: "Toril Fruit Stand",
        stall: "Stall F-02 · Toril Public Market",
        price: 160,
        unit: "/ kg",
        category: "Fruits",
        in_stock: true,
        description: "Durian · Creamy, rich Puyat variety with thick golden meat and small seeds.",
        img: images::HERO_IMG,
    },
    Product {
        id: "mangoes",
        name: "Sweet Davao Carabao Mangoes",
        local_name: "Mangga",
        vendor: "Toril Fruit Stand",
        stall: "Stall F-08 · Toril Public Market",
        price: 140,
        unit: "/ kg",
        category: "Fruits",
        in_stock: true,
        description: "Mangga · Golden sweet Davao carabao mangoes freshly harvested from Southern Mindanao orchards.",
        img: images::MANGOES_IMG,
    },
];

const CATEGORIES: &[&'static str] = &["Vegetables", "Fruits", "Meat", "Seafood"];

const DAVAO_BARANGAYS: &[&'static str] = &[
    "Obrero",
    "Matina",
    "Buhangin",
    "Agdao",
    "Toril",
    "Bajada",
    "Poblacion",
    "Lanang",
];

// -----------------------------------------------------------------------------
// Root Application
// -----------------------------------------------------------------------------
#[component]
fn App() -> Element {
    let mut current_screen = use_signal(|| AppScreen::Welcome);
    let mut user_session = use_signal(|| Option::<UserSession>::None);
    let mut auth_tab = use_signal(|| "login");
    let mut cart = use_signal(|| Vec::<CartItem>::new());
    let mut toast_msg = use_signal(|| Option::<String>::None);

    let is_welcome = current_screen() == AppScreen::Welcome;

    let on_quick_demo_login = move |role: UserRole| {
        match role {
            UserRole::Customer => {
                user_session.set(Some(UserSession {
                    role: UserRole::Customer,
                    name: "Ana Santos".into(),
                    phone: "0917 555 4826".into(),
                    address: "18 N. Torres St.".into(),
                    barangay: "Obrero".into(),
                    suki_points: 150,
                    stall_name: None,
                    stall_number: None,
                    vehicle_info: None,
                }));
                cart.set(vec![
                    CartItem {
                        id: "tomatoes",
                        name: "Tomatoes",
                        local_name: "Kamatis",
                        vendor: "Aling Nena's Gulayan",
                        stall: "Stall V-24 · Bankerohan Public Market",
                        price: 80,
                        quarters: 4, // 1.00 kg = ₱80.00
                        img: images::TOMATOES_IMG,
                    },
                    CartItem {
                        id: "carrots",
                        name: "Carrots",
                        local_name: "Karot",
                        vendor: "Aling Nena's Gulayan",
                        stall: "Stall V-24 · Bankerohan Public Market",
                        price: 120,
                        quarters: 2, // 0.50 kg = ₱60.00
                        img: images::CARROTS_IMG,
                    },
                    CartItem {
                        id: "pechay",
                        name: "Native pechay",
                        local_name: "Pechay",
                        vendor: "Aling Nena's Gulayan",
                        stall: "Stall V-24 · Bankerohan Public Market",
                        price: 60,
                        quarters: 2, // 0.50 kg = ₱30.00
                        img: images::CABBAGE_IMG,
                    },
                    CartItem {
                        id: "chicken",
                        name: "Chicken",
                        local_name: "Manok",
                        vendor: "Davao Poultry Stalls",
                        stall: "Stall M-08 · Bankerohan Public Market",
                        price: 240,
                        quarters: 6, // 1.50 kg = ₱360.00
                        img: images::CHICKEN_IMG,
                    },
                    CartItem {
                        id: "bananas",
                        name: "Lakatan bananas",
                        local_name: "Saging",
                        vendor: "Toril Fruit Terminal",
                        stall: "Stall F-15 · Toril Public Market",
                        price: 60,
                        quarters: 8, // 2.00 kg = ₱120.00
                        img: images::BANANAS_IMG,
                    },
                ]);
                toast_msg.set(Some("Maayong pag-abot, Ana Santos! Connected to Bankerohan 🌿".into()));
                current_screen.set(AppScreen::Marketplace);
            }
            UserRole::Delivery => {
                user_session.set(Some(UserSession {
                    role: UserRole::Delivery,
                    name: "Jun Mercado".into(),
                    phone: "0918 777 1108".into(),
                    address: "Bankerohan Dispatch Hub".into(),
                    barangay: "Poblacion".into(),
                    suki_points: 0,
                    stall_name: None,
                    stall_number: None,
                    vehicle_info: Some("Motorcycle 1108-DX".into()),
                }));
                toast_msg.set(Some("Maayong adlaw, Jun! Shopper Dispatch ready 🛵".into()));
                current_screen.set(AppScreen::ShopperPortal);
            }
            UserRole::Vendor => {
                user_session.set(Some(UserSession {
                    role: UserRole::Vendor,
                    name: "Aling Nena".into(),
                    phone: "0919 888 2424".into(),
                    address: "Bankerohan Central Wholesale".into(),
                    barangay: "Poblacion".into(),
                    suki_points: 0,
                    stall_name: Some("Nena's Gulayan".into()),
                    stall_number: Some("Stall V-24".into()),
                    vehicle_info: None,
                }));
                toast_msg.set(Some("Maayong adlaw, Aling Nena! Stall V-24 active 🏪".into()));
                current_screen.set(AppScreen::VendorPortal);
            }
        }
    };

    rsx! {
        document::Style { {style::render_css()} }

        div { class: if is_welcome { "device-wrapper splash-theme" } else { "device-wrapper" },

            // Active Screen Content
            match current_screen() {
                AppScreen::Welcome => rsx! {
                    WelcomeScreen {
                        on_login_click: move |_| {
                            auth_tab.set("login");
                            current_screen.set(AppScreen::Auth);
                        },
                        on_register_click: move |_| {
                            auth_tab.set("register");
                            current_screen.set(AppScreen::Auth);
                        },
                        on_quick_demo_login: on_quick_demo_login,
                    }
                },
                AppScreen::Auth => rsx! {
                    AuthScreen {
                        auth_tab: auth_tab,
                        on_back: move |_| current_screen.set(AppScreen::Welcome),
                        on_quick_demo_login: on_quick_demo_login,
                        on_auth_success: move |session: UserSession| {
                            let role = session.role;
                            user_session.set(Some(session));
                            match role {
                                UserRole::Customer => {
                                    toast_msg.set(Some("Welcome to Suki Davao 🌿".into()));
                                    current_screen.set(AppScreen::Marketplace);
                                }
                                UserRole::Delivery => {
                                    toast_msg.set(Some("Welcome, Jun! Shopper Dispatch active 🛵".into()));
                                    current_screen.set(AppScreen::ShopperPortal);
                                }
                                UserRole::Vendor => {
                                    toast_msg.set(Some("Welcome, Aling Nena! Stall V-24 active 🏪".into()));
                                    current_screen.set(AppScreen::VendorPortal);
                                }
                            }
                        },
                    }
                },
                AppScreen::Marketplace => rsx! {
                    MarketplaceScreen {
                        session: user_session,
                        cart: cart,
                        toast_msg: toast_msg,
                        on_switch_role: on_quick_demo_login,
                        on_logout: move |_| {
                            user_session.set(None);
                            cart.set(Vec::new());
                            current_screen.set(AppScreen::Welcome);
                        },
                    }
                },
                AppScreen::ShopperPortal => rsx! {
                    ShopperPortalScreen {
                        shopper_name: "Jun Mercado".to_string(),
                        on_switch_role: on_quick_demo_login,
                        on_logout: move |_| {
                            user_session.set(None);
                            current_screen.set(AppScreen::Welcome);
                        },
                    }
                },
                AppScreen::VendorPortal => rsx! {
                    VendorPortalScreen {
                        stall_name: "Nena's Gulayan".to_string(),
                        stall_number: "Stall V-24".to_string(),
                        on_switch_role: on_quick_demo_login,
                        on_logout: move |_| {
                            user_session.set(None);
                            current_screen.set(AppScreen::Welcome);
                        },
                    }
                },
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Component: Brand Logo
// -----------------------------------------------------------------------------
#[component]
fn BrandLogo(is_dark: bool) -> Element {
    rsx! {
        div { class: "brand-logo-wrap",
            div { class: if is_dark { "brand-leaf-icon dark-bg" } else { "brand-leaf-icon" },
                "🌿"
            }
            span { class: if is_dark { "brand-text light" } else { "brand-text" },
                "SUKI DAVAO"
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Screen: Welcome Screen (Home Preview with Gated Account Registration/Login)
// -----------------------------------------------------------------------------
#[component]
fn WelcomeScreen(
    on_login_click: EventHandler<()>,
    on_register_click: EventHandler<()>,
    on_quick_demo_login: EventHandler<UserRole>,
) -> Element {
    rsx! {
        div { class: "top-nav",
            div { class: "nav-back-group",
                span { style: "color: #dafa9c; font-weight: 700;", "Davao City Palengke Hub" }
            }
            div { style: "font-size: 11px; color: rgba(255,255,255,0.7); font-weight: 600;",
                "Bankerohan · Agdao · Toril"
            }
        }

        div { class: "screen-scroll",
            div { class: "splash-container",
                div {
                    BrandLogo { is_dark: true }
                    h1 { class: "splash-hero-title",
                        "Fresh from Bankerohan, Delivered to Your Doorstep."
                    }
                    div { class: "splash-img-wrap",
                        img { src: "{images::HERO_IMG}", alt: "Bankerohan Market Produce" }
                    }
                    p { class: "splash-sub", "Direct bagsakan wholesale prices. Handpicked by personal shoppers." }

                    div { class: "dots-indicator",
                        div { class: "dot active" }
                        div { class: "dot" }
                        div { class: "dot" }
                    }
                    p { class: "splash-caption", "Davao City's Accredited Market Dispatch" }
                }

                div { style: "display: flex; flex-direction: column; gap: 10px; width: 100%; margin-top: 16px;",
                    button {
                        class: "splash-cta-btn",
                        onclick: move |_| on_register_click.call(()),
                        "Create Suki Account (Register) →"
                    }
                    button {
                        style: "background: rgba(255,255,255,0.15); border: 1px solid rgba(255,255,255,0.3); color: #ffffff; padding: 13px; border-radius: 14px; font-weight: 700; cursor: pointer; font-size: 15px;",
                        onclick: move |_| on_login_click.call(()),
                        "Log in to Existing Account"
                    }

                    // Direct Role Access ("Para daretso na lang")
                    div { style: "border-top: 1px solid rgba(255,255,255,0.2); padding-top: 12px; margin-top: 4px; text-align: center;",
                        p { style: "font-size: 11px; color: #dafa9c; margin-bottom: 8px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.5px;",
                            "⚡ Direct Role Access (Para daretso na lang):"
                        }
                        div { style: "display: flex; gap: 6px; justify-content: center;",
                            button {
                                style: "background: rgba(255,255,255,0.2); border: 1px solid rgba(255,255,255,0.4); color: #ffffff; padding: 7px 10px; border-radius: 10px; font-size: 11px; font-weight: 700; cursor: pointer; flex: 1;",
                                onclick: move |_| on_quick_demo_login.call(UserRole::Customer),
                                "👤 Customer"
                            }
                            button {
                                style: "background: rgba(255,255,255,0.2); border: 1px solid rgba(255,255,255,0.4); color: #ffffff; padding: 7px 10px; border-radius: 10px; font-size: 11px; font-weight: 700; cursor: pointer; flex: 1;",
                                onclick: move |_| on_quick_demo_login.call(UserRole::Delivery),
                                "🛵 Shopper"
                            }
                            button {
                                style: "background: rgba(255,255,255,0.2); border: 1px solid rgba(255,255,255,0.4); color: #ffffff; padding: 7px 10px; border-radius: 10px; font-size: 11px; font-weight: 700; cursor: pointer; flex: 1;",
                                onclick: move |_| on_quick_demo_login.call(UserRole::Vendor),
                                "🏪 Vendor"
                            }
                        }
                    }
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Screen: Auth Screen (Unified Log In & Register Lifecycle with Multi-Role)
// -----------------------------------------------------------------------------
#[component]
fn AuthScreen(
    mut auth_tab: Signal<&'static str>,
    on_back: EventHandler<()>,
    on_quick_demo_login: EventHandler<UserRole>,
    on_auth_success: EventHandler<UserSession>,
) -> Element {
    let mut selected_role = use_signal(|| UserRole::Customer);

    // Form inputs
    let mut reg_name = use_signal(|| "Ana Santos".to_string());
    let mut reg_phone = use_signal(|| "0917 555 4826".to_string());
    let mut reg_barangay = use_signal(|| "Obrero".to_string());
    let mut reg_street = use_signal(|| "18 N. Torres St.".to_string());
    let mut reg_pass = use_signal(|| "suki123".to_string());

    // Role-specific fields
    let mut stall_name = use_signal(|| "Nena's Gulayan".to_string());
    let mut stall_num = use_signal(|| "Stall V-24".to_string());
    let mut vehicle_plate = use_signal(|| "Motorcycle 1108-DX".to_string());

    let mut login_phone = use_signal(|| "0917 555 4826".to_string());
    let mut login_pass = use_signal(|| "suki123".to_string());

    let is_login = auth_tab() == "login";

    rsx! {
        div { class: "top-nav",
            button {
                class: "nav-back-group",
                onclick: move |_| on_back.call(()),
                span { "←" }
                span { "Back to Home" }
            }
            span { style: "font-size: 12px; color: var(--text-muted); font-weight: 600;", "Suki Davao Access" }
        }

        div { class: "screen-scroll",
            // Role Selection Tabs (Customer, Delivery, Vendor)
            div { style: "padding: 12px 16px 4px 16px;",
                span { style: "font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; display: block; margin-bottom: 6px;",
                    "Select Account Type:"
                }
                div { style: "display: flex; gap: 6px;",
                    button {
                        class: if selected_role() == UserRole::Customer { "stage-chip-btn active" } else { "stage-chip-btn" },
                        onclick: move |_| {
                            selected_role.set(UserRole::Customer);
                            reg_name.set("Ana Santos".to_string());
                            login_phone.set("0917 555 4826".to_string());
                        },
                        "👤 Customer"
                    }
                    button {
                        class: if selected_role() == UserRole::Delivery { "stage-chip-btn active" } else { "stage-chip-btn" },
                        onclick: move |_| {
                            selected_role.set(UserRole::Delivery);
                            reg_name.set("Jun Mercado".to_string());
                            login_phone.set("0918 777 1108".to_string());
                        },
                        "🛵 Delivery Shopper"
                    }
                    button {
                        class: if selected_role() == UserRole::Vendor { "stage-chip-btn active" } else { "stage-chip-btn" },
                        onclick: move |_| {
                            selected_role.set(UserRole::Vendor);
                            reg_name.set("Aling Nena".to_string());
                            login_phone.set("0919 888 2424".to_string());
                        },
                        "🏪 Vendor Stall"
                    }
                }
            }

            // Quick 1-Click Demo Launcher ("Para daretso na lang")
            div { style: "padding: 8px 16px;",
                button {
                    style: "width: 100%; background: #f0fdf4; border: 1.5px dashed var(--primary); border-radius: 10px; padding: 10px; font-size: 12px; font-weight: 700; color: var(--primary); cursor: pointer;",
                    onclick: move |_| on_quick_demo_login.call(selected_role()),
                    match selected_role() {
                        UserRole::Customer => "⚡ Quick Launch as Ana Santos (Customer) →",
                        UserRole::Delivery => "⚡ Quick Launch as Jun Mercado (Shopper) →",
                        UserRole::Vendor => "⚡ Quick Launch as Aling Nena (Stall V-24) →",
                    }
                }
            }

            // Log In vs Register Segmented Nav
            div { class: "auth-segmented-nav",
                button {
                    class: if is_login { "auth-segment-tab active" } else { "auth-segment-tab" },
                    onclick: move |_| auth_tab.set("login"),
                    "Log In"
                }
                button {
                    class: if !is_login { "auth-segment-tab active" } else { "auth-segment-tab" },
                    onclick: move |_| auth_tab.set("register"),
                    "Register"
                }
            }

            if is_login {
                div {
                    h2 { class: "auth-title",
                        match selected_role() {
                            UserRole::Customer => "Log in to Customer Account",
                            UserRole::Delivery => "Shopper Dispatch Login",
                            UserRole::Vendor => "Vendor Stall Portal Login",
                        }
                    }
                    p { class: "auth-subtitle",
                        match selected_role() {
                            UserRole::Customer => "Enter your Davao mobile number to access daily fresh palengke wholesale.",
                            UserRole::Delivery => "Authorized personal shoppers and motorcycle couriers at Bankerohan, Agdao & Toril.",
                            UserRole::Vendor => "Stall owners and accredited wholesalers managing morning catalog and orders.",
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Davao Mobile Number" }
                        input {
                            class: "form-input",
                            r#type: "tel",
                            placeholder: "0917 555 4826",
                            value: "{login_phone}",
                            oninput: move |e| login_phone.set(e.value())
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Passcode / PIN" }
                        input {
                            class: "form-input",
                            r#type: "password",
                            placeholder: "••••",
                            value: "{login_pass}",
                            oninput: move |e| login_pass.set(e.value())
                        }
                    }

                    button {
                        class: "primary-btn",
                        style: "margin-top: 10px;",
                        onclick: move |_| {
                            match selected_role() {
                                UserRole::Customer => {
                                    on_auth_success.call(UserSession {
                                        role: UserRole::Customer,
                                        name: "Ana Santos".into(),
                                        phone: login_phone(),
                                        address: "18 N. Torres St.".into(),
                                        barangay: "Obrero".into(),
                                        suki_points: 150,
                                        stall_name: None,
                                        stall_number: None,
                                        vehicle_info: None,
                                    });
                                }
                                UserRole::Delivery => {
                                    on_auth_success.call(UserSession {
                                        role: UserRole::Delivery,
                                        name: "Jun Mercado".into(),
                                        phone: login_phone(),
                                        address: "Bankerohan Dispatch".into(),
                                        barangay: "Poblacion".into(),
                                        suki_points: 0,
                                        stall_name: None,
                                        stall_number: None,
                                        vehicle_info: Some("Motorcycle 1108-DX".into()),
                                    });
                                }
                                UserRole::Vendor => {
                                    on_auth_success.call(UserSession {
                                        role: UserRole::Vendor,
                                        name: "Aling Nena".into(),
                                        phone: login_phone(),
                                        address: "Bankerohan Central Wholesale".into(),
                                        barangay: "Poblacion".into(),
                                        suki_points: 0,
                                        stall_name: Some("Nena's Gulayan".into()),
                                        stall_number: Some("Stall V-24".into()),
                                        vehicle_info: None,
                                    });
                                }
                            }
                        },
                        match selected_role() {
                            UserRole::Customer => "Log In & Start Shopping →",
                            UserRole::Delivery => "Log In to Shopper Dispatch →",
                            UserRole::Vendor => "Log In to Stall Management →",
                        }
                    }
                }
            } else {
                div {
                    h2 { class: "auth-title",
                        match selected_role() {
                            UserRole::Customer => "Join Suki Davao Customer",
                            UserRole::Delivery => "Register as Personal Shopper",
                            UserRole::Vendor => "Register Accredited Market Stall",
                        }
                    }
                    p { class: "auth-subtitle",
                        match selected_role() {
                            UserRole::Customer => "Create your customer profile for dawn delivery directly from Bankerohan, Agdao & Toril.",
                            UserRole::Delivery => "Join Davao's trusted courier fleet for fair earnings and verified payment runs.",
                            UserRole::Vendor => "Connect your market stall directly to thousands of daily households.",
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label",
                            match selected_role() {
                                UserRole::Customer => "Full Name",
                                UserRole::Delivery => "Shopper / Courier Name",
                                UserRole::Vendor => "Vendor Contact Name",
                            }
                        }
                        input {
                            class: "form-input",
                            placeholder: "e.g. Maria Clara Santos",
                            value: "{reg_name}",
                            oninput: move |e| reg_name.set(e.value())
                        }
                    }

                    if selected_role() == UserRole::Vendor {
                        div { class: "form-group",
                            label { class: "form-label", "Stall Name" }
                            input {
                                class: "form-input",
                                placeholder: "e.g. Nena's Gulayan",
                                value: "{stall_name}",
                                oninput: move |e| stall_name.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Stall Number & Terminal" }
                            input {
                                class: "form-input",
                                placeholder: "e.g. Stall V-24 · Bankerohan Wholesale",
                                value: "{stall_num}",
                                oninput: move |e| stall_num.set(e.value())
                            }
                        }
                    }

                    if selected_role() == UserRole::Delivery {
                        div { class: "form-group",
                            label { class: "form-label", "Vehicle / Motorcycle Plate" }
                            input {
                                class: "form-input",
                                placeholder: "e.g. Motorcycle 1108-DX",
                                value: "{vehicle_plate}",
                                oninput: move |e| vehicle_plate.set(e.value())
                            }
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Mobile Number" }
                        input {
                            class: "form-input",
                            r#type: "tel",
                            placeholder: "0917 XXX XXXX",
                            value: "{reg_phone}",
                            oninput: move |e| reg_phone.set(e.value())
                        }
                    }

                    if selected_role() == UserRole::Customer {
                        div { class: "form-group",
                            label { class: "form-label", "Davao Barangay" }
                            div { style: "display: flex; flex-wrap: wrap; gap: 6px; margin-top: 4px;",
                                for &bgy in DAVAO_BARANGAYS.iter() {
                                    button {
                                        class: if reg_barangay() == bgy { "category-item active" } else { "category-item" },
                                        style: "padding: 6px 10px; border-radius: 8px; font-size: 12px; font-weight: 600;",
                                        onclick: move |_| reg_barangay.set(bgy.to_string()),
                                        "{bgy}"
                                    }
                                }
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Street Address & Landmark" }
                            input {
                                class: "form-input",
                                placeholder: "e.g. 18 N. Torres St., near church",
                                value: "{reg_street}",
                                oninput: move |e| reg_street.set(e.value())
                            }
                        }
                    }

                    div { class: "form-group",
                        label { class: "form-label", "Set 4-Digit Passcode" }
                        input {
                            class: "form-input",
                            r#type: "password",
                            value: "{reg_pass}",
                            oninput: move |e| reg_pass.set(e.value())
                        }
                    }

                    button {
                        class: "primary-btn",
                        style: "margin-top: 14px;",
                        onclick: move |_| {
                            let entered_name = if reg_name().trim().is_empty() {
                                "Suki User".to_string()
                            } else {
                                reg_name().trim().to_string()
                            };
                            let role = selected_role();
                            match role {
                                UserRole::Customer => {
                                    on_auth_success.call(UserSession {
                                        role: UserRole::Customer,
                                        name: entered_name,
                                        phone: reg_phone(),
                                        address: reg_street(),
                                        barangay: reg_barangay(),
                                        suki_points: 100,
                                        stall_name: None,
                                        stall_number: None,
                                        vehicle_info: None,
                                    });
                                }
                                UserRole::Delivery => {
                                    on_auth_success.call(UserSession {
                                        role: UserRole::Delivery,
                                        name: entered_name,
                                        phone: reg_phone(),
                                        address: "Bankerohan Hub".into(),
                                        barangay: "Poblacion".into(),
                                        suki_points: 0,
                                        stall_name: None,
                                        stall_number: None,
                                        vehicle_info: Some(vehicle_plate()),
                                    });
                                }
                                UserRole::Vendor => {
                                    on_auth_success.call(UserSession {
                                        role: UserRole::Vendor,
                                        name: entered_name,
                                        phone: reg_phone(),
                                        address: "Bankerohan Wholesale".into(),
                                        barangay: "Poblacion".into(),
                                        suki_points: 0,
                                        stall_name: Some(stall_name()),
                                        stall_number: Some(stall_num()),
                                        vehicle_info: None,
                                    });
                                }
                            }
                        },
                        "Create Account & Enter Portal →"
                    }
                }
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Screen: Marketplace Screen (Hosts C06 - C15 Flows)
// -----------------------------------------------------------------------------
#[component]
fn MarketplaceScreen(
    session: Signal<Option<UserSession>>,
    mut cart: Signal<Vec<CartItem>>,
    mut toast_msg: Signal<Option<String>>,
    on_logout: EventHandler<()>,
    on_switch_role: EventHandler<UserRole>,
) -> Element {
    let sess = session().unwrap_or(UserSession {
        role: UserRole::Customer,
        name: "Ana Santos".into(),
        phone: "0917 555 4826".into(),
        address: "18 N. Torres St.".into(),
        barangay: "Obrero".into(),
        suki_points: 150,
        stall_name: None,
        stall_number: None,
        vehicle_info: None,
    });

    let mut active_tab = use_signal(|| "market");
    let mut subview = use_signal(|| MarketSubView::Catalog);
    let mut selected_category = use_signal(|| "Vegetables");
    let mut search_query = use_signal(|| String::new());
    let mut selected_product_id = use_signal(|| "tomatoes");
    let mut detail_quarters = use_signal(|| 2u32); // Default 0.50 kg
    let mut order_stage = use_signal(|| OrderScreenStage::DeliveryTracking);
    let mut chat_input = use_signal(|| String::new());
    let mut extra_chat_msgs = use_signal(|| Vec::<String>::new());
    let mut order_history_filter = use_signal(|| "All orders");
    let mut star_rating = use_signal(|| 5u32);

    // C11 / C12 / C13 / C14 checkout state
    let mut selected_address_type = use_signal(|| "home");
    let mut delivery_instructions = use_signal(|| "Please call when you reach the green gate.".to_string());
    let mut ask_replacement = use_signal(|| true);
    let mut gcash_ref = use_signal(|| "1234567890123".to_string());
    let map_search = use_signal(|| "18 N. Torres St., Obrero".to_string());
    let map_landmark = use_signal(|| "Green gate near the sari-sari store".to_string());

    let cart_items = cart();
    let total_subtotal: u32 = cart_items.iter().map(|i| i.item_total()).sum();
    let total_items_count: u32 = cart_items.len() as u32;
    let delivery_fee = 74u32;
    let grand_total = total_subtotal + delivery_fee;

    rsx! {
        // Floating Toast Message
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

        // Top Portal Role Banner with 1-click Quick Launch
        div { class: "portal-role-banner",
            div { style: "display: flex; justify-content: space-between; align-items: center;",
                div {
                    strong { style: "font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: #ffffff;", "Role: Customer" }
                    span { style: "display: block; font-size: 10px; color: #bbf7d0;", "Direct portal switch" }
                }
                div { class: "portal-quick-chips",
                    button {
                        class: "quick-chip-btn active",
                        "👤 Customer"
                    }
                    button {
                        class: "quick-chip-btn",
                        onclick: move |_| on_switch_role.call(UserRole::Delivery),
                        "🛵 Shopper"
                    }
                    button {
                        class: "quick-chip-btn",
                        onclick: move |_| on_switch_role.call(UserRole::Vendor),
                        "🏪 Vendor"
                    }
                }
            }
        }

        div { class: "screen-scroll",
            match active_tab() {
                "market" => rsx! {
                    match subview() {
                        MarketSubView::Catalog => rsx! {
                            // C06: Product categories & The Market
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    span { class: "market-screen-heading", "← The market" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Davao Market Guide: Bankerohan dawn wholesale pricing.".into())),
                                    "?"
                                }
                            }

                            // English or Bisaya Search Trigger
                            div {
                                class: "bilingual-search-box",
                                onclick: move |_| subview.set(MarketSubView::BilingualSearch),
                                span { class: "search-kicker-label", "English or Bisaya" }
                                input {
                                    class: "search-input-clean",
                                    placeholder: "Enter produce, meat or pantry item",
                                    readonly: true,
                                    value: "{search_query}"
                                }
                            }

                            // Category Filter Pills
                            div { class: "cat-chips-bar",
                                for &cat in CATEGORIES.iter() {
                                    button {
                                        class: if selected_category() == cat { "cat-filter-btn active" } else { "cat-filter-btn" },
                                        onclick: move |_| selected_category.set(cat),
                                        "{cat}"
                                    }
                                }
                            }

                            // Big Section Title & Subtitle
                            div { class: "section-intro-c06",
                                h2 { class: "big-section-title", "Fresh {selected_category().to_lowercase()}" }
                                p { class: "section-sub-copy", "Handpicked from trusted stalls, ready for your kusina." }
                                div { class: "filter-row-secondary",
                                    span { class: "tag-pill-green", "✓ Accredited only" }
                                    span { class: "tag-pill-green", style: "background: #f1f5f9; color: var(--text-main);", "Popular ▾" }
                                }
                            }

                            // Products Grid (2 columns)
                            div { class: "products-grid-c06",
                                for p in PRODUCTS.iter().filter(|p| p.category == selected_category()) {
                                    {
                                        let p_id = p.id;
                                        let p_name = p.name;
                                        let p_local = p.local_name;
                                        let p_vendor = p.vendor;
                                        let p_stall = p.stall;
                                        let p_price = p.price;
                                        let p_in_stock = p.in_stock;
                                        let p_img = p.img;

                                        rsx! {
                                            div {
                                                class: "product-card-modern",
                                                key: "{p_id}",
                                                onclick: move |_| {
                                                    selected_product_id.set(p_id);
                                                    detail_quarters.set(2);
                                                    subview.set(MarketSubView::ProductDetail);
                                                },
                                                img { class: "product-card-modern-img", src: "{p_img}", alt: "{p_name}" }
                                                div { class: "product-card-modern-body",
                                                    div {
                                                        h5 { class: "modern-prod-name", "{p_name}" }
                                                        p { class: "modern-prod-sub", "{p_local} · {p_vendor}" }
                                                    }
                                                    div { class: "modern-prod-price-row",
                                                        span { class: "modern-price-val", "₱{p_price} / kg" }
                                                        if p_in_stock {
                                                            button {
                                                                class: "btn-add-square",
                                                                onclick: move |e| {
                                                                    e.stop_propagation();
                                                                    let mut c = cart();
                                                                    if let Some(item) = c.iter_mut().find(|i| i.id == p_id) {
                                                                        item.quarters += 1;
                                                                    } else {
                                                                        c.push(CartItem {
                                                                            id: p_id,
                                                                            name: p_name,
                                                                            local_name: p_local,
                                                                            vendor: p_vendor,
                                                                            stall: p_stall,
                                                                            price: p_price,
                                                                            quarters: 1, // 0.25 kg
                                                                            img: p_img,
                                                                        });
                                                                    }
                                                                    cart.set(c);
                                                                    toast_msg.set(Some(format!("Added 0.25 kg of {}!", p_name)));
                                                                },
                                                                "+"
                                                            }
                                                        } else {
                                                            span { class: "badge-out-of-stock", "Out of stock" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            p { class: "guide-price-footer-note",
                                "Guide prices · Updated 6:30 AM · You approve any increase before purchase."
                            }

                            // Sticky Floating Basket Button
                            if total_items_count > 0 {
                                button {
                                    class: "floating-basket-pill",
                                    onclick: move |_| subview.set(MarketSubView::Basket),
                                    "View basket · {total_items_count} items · ₱{total_subtotal}.00"
                                }
                            }
                        },

                        MarketSubView::BilingualSearch => rsx! {
                            // C07: Bilingual Search Screen
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| subview.set(MarketSubView::Catalog),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Find your market favorites" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Search using Bisaya terms like kamatis, manok, isda, or talong!".into())),
                                    "?"
                                }
                            }

                            div { class: "bilingual-search-box",
                                span { class: "search-kicker-label", "Search in English or Bisaya" }
                                div { style: "display: flex; align-items: center; justify-content: space-between;",
                                    input {
                                        class: "search-input-clean",
                                        placeholder: "kamatis",
                                        value: "{search_query}",
                                        oninput: move |e| search_query.set(e.value())
                                    }
                                    if !search_query().is_empty() {
                                        button {
                                            style: "background: none; border: none; font-size: 16px; color: var(--text-muted); cursor: pointer;",
                                            onclick: move |_| search_query.set(String::new()),
                                            "×"
                                        }
                                    }
                                }
                            }

                            div { style: "display: flex; gap: 8px; padding: 0 16px 8px 16px;",
                                span { class: "tag-pill-green", "English + Bisaya" }
                                span { class: "tag-pill-green", style: "background: #f1f5f9; color: var(--text-main);", "In stock ▾" }
                            }

                            // Dynamic Bilingual Headline
                            div { class: "bilingual-result-card",
                                {
                                    let q = search_query().trim().to_lowercase();
                                    if q.contains("kamatis") || q.contains("tomato") {
                                        rsx! {
                                            h3 { class: "bilingual-match-headline", "You say kamatis. We find tomatoes." }
                                            p { class: "bilingual-match-sub", "2 matches · Vegetables · Accredited vendors" }
                                        }
                                    } else if q.contains("manok") || q.contains("chicken") {
                                        rsx! {
                                            h3 { class: "bilingual-match-headline", "You say manok. We find chicken." }
                                            p { class: "bilingual-match-sub", "1 match · Meat · Accredited vendors" }
                                        }
                                    } else if q.contains("isda") || q.contains("tuna") {
                                        rsx! {
                                            h3 { class: "bilingual-match-headline", "You say isda. We find yellowfin tuna." }
                                            p { class: "bilingual-match-sub", "1 match · Seafood · Accredited vendors" }
                                        }
                                    } else if q.contains("karot") || q.contains("carrot") {
                                        rsx! {
                                            h3 { class: "bilingual-match-headline", "You say karot. We find carrots." }
                                            p { class: "bilingual-match-sub", "1 match · Vegetables · Accredited vendors" }
                                        }
                                    } else if q.contains("talong") || q.contains("eggplant") {
                                        rsx! {
                                            h3 { class: "bilingual-match-headline", "You say talong. We find eggplant." }
                                            p { class: "bilingual-match-sub", "1 match · Vegetables · Accredited vendors" }
                                        }
                                    } else {
                                        rsx! {
                                            h3 { class: "bilingual-match-headline", "You say kamatis. We find tomatoes." }
                                            p { class: "bilingual-match-sub", "Try searching Davao market staples in Bisaya or English" }
                                        }
                                    }
                                }
                            }

                            // Search Results Grid
                            div { class: "products-grid-c06",
                                for p in PRODUCTS.iter().filter(|p| {
                                    let q = search_query().trim().to_lowercase();
                                    if q.is_empty() {
                                        p.id == "tomatoes" || p.id == "salad_tomatoes"
                                    } else {
                                        p.name.to_lowercase().contains(&q)
                                            || p.local_name.to_lowercase().contains(&q)
                                            || p.category.to_lowercase().contains(&q)
                                    }
                                }) {
                                    {
                                        let p_id = p.id;
                                        let p_name = p.name;
                                        let p_local = p.local_name;
                                        let p_vendor = p.vendor;
                                        let p_stall = p.stall;
                                        let p_price = p.price;
                                        let p_img = p.img;
                                        rsx! {
                                            div {
                                                class: "product-card-modern",
                                                key: "{p_id}",
                                                onclick: move |_| {
                                                    selected_product_id.set(p_id);
                                                    detail_quarters.set(2);
                                                    subview.set(MarketSubView::ProductDetail);
                                                },
                                                img { class: "product-card-modern-img", src: "{p_img}", alt: "{p_name}" }
                                                div { class: "product-card-modern-body",
                                                    div {
                                                        h5 { class: "modern-prod-name", "{p_name}" }
                                                        p { class: "modern-prod-sub", "{p_local} · {p_vendor}" }
                                                    }
                                                    div { class: "modern-prod-price-row",
                                                        span { class: "modern-price-val", "₱{p_price} / kg" }
                                                        button {
                                                            class: "btn-add-square",
                                                            onclick: move |e| {
                                                                e.stop_propagation();
                                                                let mut c = cart();
                                                                if let Some(item) = c.iter_mut().find(|i| i.id == p_id) {
                                                                    item.quarters += 1;
                                                                } else {
                                                                    c.push(CartItem {
                                                                        id: p_id,
                                                                        name: p_name,
                                                                        local_name: p_local,
                                                                        vendor: p_vendor,
                                                                        stall: p_stall,
                                                                        price: p_price,
                                                                        quarters: 1,
                                                                        img: p_img,
                                                                    });
                                                                }
                                                                cart.set(c);
                                                                toast_msg.set(Some(format!("Added {} to basket!", p_name)));
                                                            },
                                                            "+"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Try it in either language Section
                            div { class: "either-lang-card",
                                h4 { class: "either-lang-heading", "Try it in either language" }
                                div {
                                    class: "either-lang-pair",
                                    onclick: move |_| search_query.set("manok".into()),
                                    span { "Chicken ↔ Manok" }
                                    span { "→" }
                                }
                                div {
                                    class: "either-lang-pair",
                                    onclick: move |_| search_query.set("talong".into()),
                                    span { "Eggplant ↔ Talong" }
                                    span { "→" }
                                }
                                div {
                                    class: "either-lang-pair",
                                    onclick: move |_| search_query.set("karot".into()),
                                    span { "Carrots ↔ Karot" }
                                    span { "→" }
                                }
                                div {
                                    class: "either-lang-pair",
                                    onclick: move |_| search_query.set("isda".into()),
                                    span { "Fish ↔ Isda" }
                                    span { "→" }
                                }
                                div {
                                    class: "either-lang-pair",
                                    onclick: move |_| search_query.set("kamatis".into()),
                                    span { "Tomatoes ↔ Kamatis" }
                                    span { "→" }
                                }
                            }

                            // Can't Find Ingredient Callout
                            div { class: "cant-find-callout",
                                strong { "Can't find your ingredient?" }
                                p { style: "margin-top: 4px;",
                                    "Try a local name or browse the category. Prices and availability are refreshed by our accredited market vendors."
                                }
                            }

                            if total_items_count > 0 {
                                button {
                                    class: "floating-basket-pill",
                                    onclick: move |_| subview.set(MarketSubView::Basket),
                                    "View basket · {total_items_count} items · ₱{total_subtotal}.00"
                                }
                            }
                        },

                        MarketSubView::ProductDetail => rsx! {
                            // C08: Product Details Screen (Fresh from the stall)
                            {
                                let product = PRODUCTS.iter().find(|p| p.id == selected_product_id()).unwrap_or(&PRODUCTS[0]);
                                let p_id = product.id;
                                let p_name = product.name;
                                let p_local = product.local_name;
                                let p_vendor = product.vendor;
                                let p_stall = product.stall;
                                let p_price = product.price;
                                let p_img = product.img;
                                let p_desc = product.description;
                                let weight_kg = detail_quarters() as f64 * 0.25;
                                let total_for_weight = ((p_price as f64 * weight_kg).round()) as u32;

                                rsx! {
                                    div { class: "market-top-header",
                                        div { class: "market-header-left",
                                            button {
                                                class: "market-back-btn",
                                                onclick: move |_| subview.set(MarketSubView::Catalog),
                                                "←"
                                            }
                                            span { class: "market-screen-heading", "Fresh from the stall" }
                                        }
                                        button {
                                            class: "market-help-icon",
                                            onclick: move |_| toast_msg.set(Some("Handpicked fresh upon order confirmation.".into())),
                                            "?"
                                        }
                                    }

                                    img { class: "detail-hero-frame", src: "{p_img}", alt: "{p_name}" }

                                    span { class: "badge-fresh-stock", "Fresh today · In stock" }
                                    h1 { class: "detail-name-big", "{p_name}" }
                                    p { class: "detail-desc-text", "{p_desc}" }

                                    div { class: "detail-price-panel",
                                        div {
                                            span { style: "font-size: 11px; color: var(--text-muted); font-weight: 700; text-transform: uppercase;", "Guide price · per kilogram" }
                                            p { style: "font-size: 11px; color: var(--text-muted); margin-top: 2px;", "Updated today, 6:30 AM · Final cost uses actual weight" }
                                        }
                                        span { style: "font-size: 22px; font-weight: 800; color: var(--primary-dark);", "₱{p_price}.00" }
                                    }

                                    div { class: "vendor-stall-card",
                                        h4 { style: "font-size: 15px; font-weight: 700; color: var(--primary-dark);", "{p_vendor}" }
                                        p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "{p_stall}" }
                                        span { class: "vendor-tag-badge", "✓ SUKI accredited · Vendor checked" }
                                    }

                                    div { class: "weight-stepper-card",
                                        h4 { style: "font-size: 14px; font-weight: 700; color: var(--text-main);", "How much do you need?" }
                                        div { class: "stepper-flex-row",
                                            button {
                                                class: "stepper-round-btn",
                                                onclick: move |_| {
                                                    if detail_quarters() > 1 {
                                                        detail_quarters.set(detail_quarters() - 1);
                                                    }
                                                },
                                                "—"
                                            }
                                            span { class: "stepper-weight-text", "{weight_kg:.2} kg" }
                                            button {
                                                class: "stepper-round-btn",
                                                onclick: move |_| detail_quarters.set(detail_quarters() + 1),
                                                "+"
                                            }
                                        }
                                        p { style: "font-size: 11px; color: var(--text-muted);", "¼ kg steps · 0.25 / 0.50 / 0.75 / 1.00 kg" }
                                    }

                                    div { class: "fresh-weigh-pledge",
                                        h5 { class: "pledge-heading", "⚖ We weigh it fresh. You stay in control." }
                                        p { class: "pledge-sub",
                                            "Actual weight × market price = final cost. Any price increase or substitution needs your approval before purchase."
                                        }
                                    }

                                    button {
                                        class: "floating-basket-pill",
                                        onclick: move |_| {
                                            let mut c = cart();
                                            if let Some(item) = c.iter_mut().find(|i| i.id == p_id) {
                                                item.quarters = detail_quarters();
                                            } else {
                                                c.push(CartItem {
                                                    id: p_id,
                                                    name: p_name,
                                                    local_name: p_local,
                                                    vendor: p_vendor,
                                                    stall: p_stall,
                                                    price: p_price,
                                                    quarters: detail_quarters(),
                                                    img: p_img,
                                                });
                                            }
                                            cart.set(c);
                                            toast_msg.set(Some(format!("Added {:.2} kg of {} to basket!", weight_kg, p_name)));
                                            subview.set(MarketSubView::Basket);
                                        },
                                        "Add to basket — ₱{total_for_weight}.00"
                                    }
                                }
                            }
                        },

                        MarketSubView::Basket => rsx! {
                            // C09 / C10: Shopping Basket Experience
                            if total_subtotal < 450 {
                                // C10: Minimum-Order Validation Screen
                                div { class: "market-top-header",
                                    div { class: "market-header-left",
                                        button {
                                            class: "market-back-btn",
                                            onclick: move |_| subview.set(MarketSubView::Catalog),
                                            "←"
                                        }
                                        span { class: "market-screen-heading", "Basket · minimum not met" }
                                    }
                                    button {
                                        class: "market-help-icon",
                                        onclick: move |_| toast_msg.set(Some("Davao direct couriers require ₱450 grocery minimum.".into())),
                                        "?"
                                    }
                                }

                                div { style: "padding: 16px;",
                                    h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark);", "A little more freshness." }
                                    p { style: "font-size: 13px; color: var(--text-muted); margin-top: 4px;",
                                        "Your basket is saved. Add groceries before checking out."
                                    }
                                }

                                div { class: "basket-items-card",
                                    for (idx, item) in cart_items.iter().enumerate() {
                                        div { class: "basket-item-single", key: "{item.id}",
                                            div { style: "display: flex; align-items: center; gap: 10px;",
                                                img { src: "{item.img}", alt: "{item.name}", style: "width: 48px; height: 48px; border-radius: 8px; object-fit: cover;" }
                                                div {
                                                    h5 { style: "font-size: 14px; font-weight: 700;", "{item.name} · {item.local_name}" }
                                                    p { style: "font-size: 11px; color: var(--text-muted);", "{item.weight_str()} × ₱{item.price}/kg" }
                                                }
                                            }
                                            div { style: "display: flex; align-items: center; gap: 8px;",
                                                button {
                                                    class: "stepper-round-btn",
                                                    style: "width: 28px; height: 28px; font-size: 14px;",
                                                    onclick: move |_| {
                                                        let mut c = cart();
                                                        if c[idx].quarters > 1 {
                                                            c[idx].quarters -= 1;
                                                        } else {
                                                            c.remove(idx);
                                                        }
                                                        cart.set(c);
                                                    },
                                                    "—"
                                                }
                                                span { style: "font-weight: 700; font-size: 13px; min-width: 50px; text-align: center;", "{item.weight_str()}" }
                                                button {
                                                    class: "stepper-round-btn",
                                                    style: "width: 28px; height: 28px; font-size: 14px;",
                                                    onclick: move |_| {
                                                        let mut c = cart();
                                                        c[idx].quarters += 1;
                                                        cart.set(c);
                                                    },
                                                    "+"
                                                }
                                                span { style: "font-size: 13px; font-weight: 800; min-width: 60px; text-align: right; color: var(--primary-dark);",
                                                    "₱{item.item_total()}.00"
                                                }
                                            }
                                        }
                                    }
                                    div { class: "subtotal-sum-row",
                                        span { "Grocery subtotal" }
                                        span { "₱{total_subtotal}.00" }
                                    }
                                }

                                div { class: "min-order-callout-deficit",
                                    strong { "ℹ Add ₱{450 - total_subtotal}.00 to reach ₱450" }
                                    p { style: "margin-top: 4px;",
                                        "The minimum applies to groceries only. Delivery fees do not count toward the ₱450 grocery minimum."
                                    }
                                }

                                button {
                                    class: "btn-checkout-disabled",
                                    "Checkout unavailable · below minimum"
                                }

                                button {
                                    class: "primary-btn",
                                    style: "width: calc(100% - 32px); margin: 0 16px 80px 16px;",
                                    onclick: move |_| subview.set(MarketSubView::Catalog),
                                    "Browse more groceries"
                                }
                            } else {
                                // C09: Your Shopping Basket (Minimum Met)
                                div { class: "market-top-header",
                                    div { class: "market-header-left",
                                        button {
                                            class: "market-back-btn",
                                            onclick: move |_| subview.set(MarketSubView::Catalog),
                                            "←"
                                        }
                                        span { class: "market-screen-heading", "Your shopping basket" }
                                    }
                                    button {
                                        class: "market-help-icon",
                                        onclick: move |_| toast_msg.set(Some("Order dispatched from Bankerohan Central Wholesale.".into())),
                                        "?"
                                    }
                                }

                                div { style: "padding: 16px;",
                                    h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark);", "Your market list" }
                                    p { style: "font-size: 13px; color: var(--text-muted); margin-top: 4px;",
                                        "{total_items_count} items · 3 accredited Bankerohan stalls"
                                    }
                                }

                                div { class: "basket-items-card",
                                    for (idx, item) in cart_items.iter().enumerate() {
                                        div { class: "basket-item-single", key: "{item.id}",
                                            div { style: "display: flex; align-items: center; gap: 10px;",
                                                img { src: "{item.img}", alt: "{item.name}", style: "width: 44px; height: 44px; border-radius: 8px; object-fit: cover;" }
                                                div {
                                                    h5 { style: "font-size: 13px; font-weight: 700;", "{item.name} · {item.local_name}" }
                                                    p { style: "font-size: 11px; color: var(--text-muted);", "{item.weight_str()} · ₱{item.price}/kg" }
                                                }
                                            }
                                            div { style: "display: flex; align-items: center; gap: 6px;",
                                                button {
                                                    class: "stepper-round-btn",
                                                    style: "width: 26px; height: 26px; font-size: 13px;",
                                                    onclick: move |_| {
                                                        let mut c = cart();
                                                        if c[idx].quarters > 1 {
                                                            c[idx].quarters -= 1;
                                                        } else {
                                                            c.remove(idx);
                                                        }
                                                        cart.set(c);
                                                    },
                                                    "—"
                                                }
                                                span { style: "font-weight: 700; font-size: 12px; min-width: 45px; text-align: center;", "{item.weight_str()}" }
                                                button {
                                                    class: "stepper-round-btn",
                                                    style: "width: 26px; height: 26px; font-size: 13px;",
                                                    onclick: move |_| {
                                                        let mut c = cart();
                                                        c[idx].quarters += 1;
                                                        cart.set(c);
                                                    },
                                                    "+"
                                                }
                                                span { style: "font-size: 13px; font-weight: 800; min-width: 55px; text-align: right; color: var(--primary-dark);",
                                                    "₱{item.item_total()}.00"
                                                }
                                            }
                                        }
                                    }
                                }

                                p { style: "font-size: 11px; color: var(--text-muted); padding: 0 16px 8px 16px;", "Edit quantities · ¼ kg steps available" }

                                div { class: "min-order-pill-met",
                                    span { "✓" }
                                    span { "₱450 grocery minimum met · ₱{total_subtotal} subtotal" }
                                }

                                div { class: "estimate-card-c09",
                                    h4 { style: "font-size: 15px; font-weight: 700; margin-bottom: 12px;", "Your estimate" }
                                    div { style: "display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 6px;",
                                        span { "Groceries · {total_items_count} items" }
                                        span { "₱{total_subtotal}.00" }
                                    }
                                    div { style: "display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 2px;",
                                        span { "Delivery · 4.00 km" }
                                        span { "₱{delivery_fee}.00" }
                                    }
                                    p { style: "font-size: 11px; color: var(--text-muted); margin-bottom: 12px;",
                                        "₱50 base + ₱6.00 × 4 km · No extra shopper fee"
                                    }
                                    div { style: "display: flex; justify-content: space-between; font-size: 16px; font-weight: 800; border-top: 1px solid var(--border-color); padding-top: 10px; color: var(--primary-dark);",
                                        span { "Total to pay" }
                                        span { "₱{grand_total}.00" }
                                    }
                                }

                                div { class: "fresh-weigh-pledge", style: "margin-bottom: 20px;",
                                    h5 { class: "pledge-heading", "🛡️ Changes are always your choice" }
                                    p { class: "pledge-sub",
                                        "We ask before price increases or substitutions. Rejected and unavailable items are not bought or billed."
                                    }
                                }

                                button {
                                    class: "primary-btn",
                                    style: "width: calc(100% - 32px); margin: 0 16px 10px 16px;",
                                    onclick: move |_| subview.set(MarketSubView::SavedAddresses),
                                    "Choose delivery address →"
                                }

                                button {
                                    style: "background: none; border: none; width: 100%; text-align: center; color: var(--primary); font-size: 14px; font-weight: 700; cursor: pointer; padding: 10px; margin-bottom: 80px;",
                                    onclick: move |_| subview.set(MarketSubView::Catalog),
                                    "Keep shopping"
                                }
                            }
                        },

                        MarketSubView::SavedAddresses => rsx! {
                            // C11: Saved Delivery Addresses
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| subview.set(MarketSubView::Basket),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Saved delivery addresses" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Davao coverage covers District 1, 2, and 3.".into())),
                                    "?"
                                }
                            }

                            div { style: "padding: 16px 16px 8px 16px;",
                                h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark);", "Where's home today?" }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 4px;",
                                    "Choose a saved address or drop a new pin."
                                }
                            }

                            // Address Card 1: Home (Default, Selected)
                            div {
                                class: if selected_address_type() == "home" { "address-card-item selected" } else { "address-card-item" },
                                onclick: move |_| selected_address_type.set("home"),
                                div { style: "display: flex; justify-content: space-between; align-items: center;",
                                    div { style: "display: flex; align-items: center; gap: 8px;",
                                        span { style: "font-size: 14px; color: var(--primary);", if selected_address_type() == "home" { "●" } else { "○" } }
                                        strong { style: "font-size: 15px; color: var(--text-main);", "Home · default" }
                                        span { class: "address-tag-within", "Selected · Within delivery area" }
                                    }
                                }
                                p { style: "font-size: 13px; font-weight: 600; margin-top: 8px;", "Ana Santos · 0917 555 4826" }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 2px;", "18 N. Torres St., Brgy. Obrero, Davao City" }
                                p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "Green gate, near the corner sari-sari store" }
                                div { style: "display: flex; justify-content: space-between; font-size: 12px; border-top: 1px solid #f1f5f9; padding-top: 8px; margin-top: 8px; color: var(--primary);",
                                    span { style: "text-decoration: underline; cursor: pointer;", "Edit address" }
                                    span { style: "font-weight: 700;", "4.00 km · ₱74 delivery" }
                                }
                            }

                            // Address Card 2: Work (Unselected)
                            div {
                                class: if selected_address_type() == "work" { "address-card-item selected" } else { "address-card-item" },
                                onclick: move |_| selected_address_type.set("work"),
                                div { style: "display: flex; align-items: center; gap: 8px;",
                                    span { style: "font-size: 14px; color: var(--text-muted);", if selected_address_type() == "work" { "●" } else { "○" } }
                                    strong { style: "font-size: 15px; color: var(--text-main);", "Work" }
                                }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 6px;", "Unit 3, Palma Gil St., Brgy. 4-A, Davao City" }
                                p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "Reception desk, second floor · 2.20 km" }
                                div { style: "display: flex; justify-content: space-between; font-size: 12px; border-top: 1px solid #f1f5f9; padding-top: 8px; margin-top: 8px; color: var(--text-muted);",
                                    span { style: "text-decoration: underline; cursor: pointer;", "Edit address" }
                                    span { style: "font-weight: 700;", "₱61.70 delivery" }
                                }
                            }

                            // Add Address On Map Button
                            button {
                                class: "btn-add-map",
                                onclick: move |_| subview.set(MarketSubView::AddressMapPicker),
                                span { "+" }
                                span { "Add address on map" }
                            }

                            // Delivery Instructions Card
                            div { class: "delivery-instruct-card",
                                label { style: "display: block; font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; margin-bottom: 4px;", "Delivery instructions" }
                                input {
                                    class: "search-input-clean",
                                    style: "font-size: 13px; width: 100%;",
                                    value: "{delivery_instructions}",
                                    oninput: move |e| delivery_instructions.set(e.value())
                                }
                            }

                            button {
                                class: "primary-btn",
                                style: "width: calc(100% - 32px); margin: 0 16px 80px 16px;",
                                onclick: move |_| subview.set(MarketSubView::ReviewCheckout),
                                "Use Home · ₱74.00 delivery"
                            }
                        },

                        MarketSubView::AddressMapPicker => rsx! {
                            // C12: Address Map Picker
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| subview.set(MarketSubView::SavedAddresses),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Pin your doorstep" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Davao GPS Map coordinates active.".into())),
                                    "?"
                                }
                            }

                            div { class: "bilingual-search-box",
                                span { class: "search-kicker-label", "Find an address in Davao City" }
                                input {
                                    class: "search-input-clean",
                                    value: "{map_search}",
                                    readonly: true
                                }
                            }

                            // Authentic Davao Street Map Canvas with Doorstep Pin
                            DavaoStreetMap {
                                mode: MapMode::Picker,
                                shopper_name: "Jun Mercado".to_string(),
                                destination_label: "18 N. Torres St., Brgy. Obrero".to_string(),
                                distance: "4.00 km".to_string(),
                                eta: "10:40–11:00 AM".to_string(),
                                is_shopper_view: false,
                            }

                            // Selected Location Details
                            div { class: "vendor-stall-card",
                                h4 { style: "font-size: 15px; font-weight: 800; color: var(--primary-dark);", "Home · Obrero" }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 2px;", "18 N. Torres St., Brgy. Obrero, Davao City" }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; margin-top: 8px; border-top: 1px solid #f1f5f9; padding-top: 8px;",
                                    span { "Distance from Bankerohan" }
                                    strong { "4.00 km" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; margin-top: 2px; color: var(--primary);",
                                    span { "₱50 + (₱6.00 × 4 km)" }
                                    strong { "₱74.00" }
                                }
                            }

                            // Landmark Input
                            div { class: "delivery-instruct-card",
                                label { style: "display: block; font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; margin-bottom: 4px;", "Landmark / unit / gate" }
                                input {
                                    class: "search-input-clean",
                                    style: "font-size: 13px; width: 100%;",
                                    value: "{map_landmark}",
                                    readonly: true
                                }
                            }

                            div { class: "cant-find-callout", style: "margin-bottom: 20px;",
                                strong { "🛡️ Location permission is optional" }
                                p { style: "margin-top: 4px;",
                                    "Allow location to find yourself, or search and enter your address manually. Confirm your pin and landmark before saving."
                                }
                            }

                            button {
                                class: "primary-btn",
                                style: "width: calc(100% - 32px); margin: 0 16px 80px 16px;",
                                onclick: move |_| subview.set(MarketSubView::ReviewCheckout),
                                "Confirm pin & save address"
                            }
                        },

                        MarketSubView::ReviewCheckout => rsx! {
                            // C13: Review & Checkout
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| subview.set(MarketSubView::SavedAddresses),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Review & checkout" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Order ID: SKD-241108 created.".into())),
                                    "?"
                                }
                            }

                            span { class: "order-stage-banner", "SKD-241108 · STAGE: ORDER CREATED" }

                            // Deliver to Home Summary Card
                            div { class: "deliver-summary-card",
                                div { style: "display: flex; justify-content: space-between; align-items: center;",
                                    strong { style: "font-size: 15px; color: var(--text-main);", "Deliver to Home" }
                                    button {
                                        style: "background: none; border: none; color: var(--primary); font-weight: 700; font-size: 13px; cursor: pointer;",
                                        onclick: move |_| subview.set(MarketSubView::SavedAddresses),
                                        "Change"
                                    }
                                }
                                p { style: "font-size: 13px; margin-top: 4px;", "Ana Santos · 0917 •••• 4826" }
                                p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "18 N. Torres St., Brgy. Obrero, Davao City · Green gate" }
                                span { class: "badge-fresh-stock", style: "margin: 8px 0 0 0;", "Today · 10:40–11:00 AM estimated" }
                            }

                            // Items List Card
                            div { class: "basket-items-card",
                                div { style: "padding: 12px 14px 6px 14px; font-size: 14px; font-weight: 800;",
                                    "Your market list · {total_items_count} items"
                                }
                                for item in cart_items.iter() {
                                    div { class: "basket-item-single", key: "{item.id}",
                                        div { style: "display: flex; align-items: center; gap: 10px;",
                                            img { src: "{item.img}", alt: "{item.name}", style: "width: 40px; height: 40px; border-radius: 8px; object-fit: cover;" }
                                            div {
                                                h5 { style: "font-size: 13px; font-weight: 700;", "{item.name} · {item.local_name}" }
                                                p { style: "font-size: 11px; color: var(--text-muted);", "{item.weight_str()} · ₱{item.price}/kg" }
                                            }
                                        }
                                        span { style: "font-size: 13px; font-weight: 800; color: var(--primary-dark);", "₱{item.item_total()}.00" }
                                    }
                                }
                            }

                            // Replacement Choice Option Box
                            div { class: "replacement-opt-box",
                                input {
                                    r#type: "checkbox",
                                    checked: "{ask_replacement}",
                                    style: "margin-top: 3px; accent-color: var(--primary); cursor: pointer;",
                                    onchange: move |_| ask_replacement.set(!ask_replacement())
                                }
                                div {
                                    strong { style: "font-size: 13px; color: #854d0e;", "Ask me before any replacement" }
                                    p { style: "font-size: 11px; color: #713f12; margin-top: 2px;",
                                        "Price increases and substitutions require my explicit approval. Unavailable, rejected or timed-out items are excluded."
                                    }
                                }
                            }

                            // Estimate Breakdown Card
                            div { class: "estimate-card-c09",
                                h4 { style: "font-size: 15px; font-weight: 700; margin-bottom: 12px;", "Your estimate" }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 6px;",
                                    span { "Groceries · {total_items_count} items" }
                                    span { "₱{total_subtotal}.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 2px;",
                                    span { "Delivery · 4.00 km" }
                                    span { "₱{delivery_fee}.00" }
                                }
                                p { style: "font-size: 11px; color: var(--text-muted); margin-bottom: 12px;",
                                    "₱50 base + ₱6.00 × 4 km · No extra shopper fee"
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 16px; font-weight: 800; border-top: 1px solid var(--border-color); padding-top: 10px; color: var(--primary-dark);",
                                    span { "Total to pay" }
                                    span { "₱{grand_total}.00" }
                                }
                            }

                            // Payment Method Card
                            div { class: "vendor-stall-card",
                                div { style: "display: flex; align-items: center; gap: 8px;",
                                    span { style: "color: var(--primary); font-size: 15px;", "●" }
                                    strong { style: "font-size: 14px; color: var(--text-main);", "GCash QR only" }
                                }
                                p { style: "font-size: 12px; color: var(--text-muted); margin-top: 4px;",
                                    "Pay the estimate, submit your 13-digit reference, then wait for manual admin verification."
                                }
                            }

                            p { style: "font-size: 11px; color: var(--text-muted); line-height: 1.4; padding: 0 16px 12px 16px;",
                                "Shopping starts only after admin approves payment. Excess is calculated as refundable; refund processing has its own status. Any approved total above funds needs a verified top-up before purchase."
                            }

                            button {
                                class: "primary-btn",
                                style: "width: calc(100% - 32px); margin: 0 16px 80px 16px;",
                                onclick: move |_| subview.set(MarketSubView::GCashPayment),
                                "Continue to GCash · ₱{grand_total}.00"
                            }
                        },

                        MarketSubView::GCashPayment => rsx! {
                            // C14: GCash Payment Screen
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| subview.set(MarketSubView::ReviewCheckout),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Pay with GCash" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Davao Merchant Account: Suki Davao Dispatch".into())),
                                    "?"
                                }
                            }

                            span { class: "order-stage-banner", "SKD-241108 · STAGE: AWAITING PAYMENT" }
                            span { class: "status-pill-awaiting", "Payment not verified" }

                            div { style: "padding: 10px 16px 4px 16px;",
                                h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark); line-height: 1.2;",
                                    "One payment. Your market run, next."
                                }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 6px;",
                                    "Save this demo QR or open GCash. Submit the transaction reference after payment."
                                }
                            }

                            // GCash QR Display Box
                            div { class: "gcash-qr-card-box",
                                h4 { style: "font-size: 16px; font-weight: 800; color: #1e3a8a;", "GCash QR · SUKI DAVAO" }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 2px;",
                                    "₱{total_subtotal} groceries + ₱{delivery_fee} delivery"
                                }
                                h3 { style: "font-size: 26px; font-weight: 800; color: var(--primary-dark); margin: 4px 0 12px 0;",
                                    "₱{grand_total}.00"
                                }

                                // Stylized Scalable SVG QR Representation
                                svg {
                                    style: "width: 170px; height: 170px; margin: 0 auto; display: block;",
                                    view_box: "0 0 100 100",
                                    rect { width: "100", height: "100", fill: "#ffffff" }
                                    // Corners Top Left
                                    rect { x: "5", y: "5", width: "30", height: "30", fill: "#000000" }
                                    rect { x: "10", y: "10", width: "20", height: "20", fill: "#ffffff" }
                                    rect { x: "15", y: "15", width: "10", height: "10", fill: "#000000" }
                                    // Corners Top Right
                                    rect { x: "65", y: "5", width: "30", height: "30", fill: "#000000" }
                                    rect { x: "70", y: "10", width: "20", height: "20", fill: "#ffffff" }
                                    rect { x: "75", y: "15", width: "10", height: "10", fill: "#000000" }
                                    // Corners Bottom Left
                                    rect { x: "5", y: "65", width: "30", height: "30", fill: "#000000" }
                                    rect { x: "10", y: "70", width: "20", height: "20", fill: "#ffffff" }
                                    rect { x: "15", y: "75", width: "10", height: "10", fill: "#000000" }
                                    // Data Blocks
                                    rect { x: "45", y: "10", width: "10", height: "20", fill: "#000000" }
                                    rect { x: "40", y: "40", width: "20", height: "20", fill: "#000000" }
                                    rect { x: "65", y: "45", width: "15", height: "15", fill: "#000000" }
                                    rect { x: "10", y: "45", width: "20", height: "10", fill: "#000000" }
                                    rect { x: "45", y: "70", width: "20", height: "15", fill: "#000000" }
                                    rect { x: "75", y: "75", width: "15", height: "15", fill: "#000000" }
                                }

                                span { class: "qr-demo-badge", "DEMO QR · NON-PAYABLE" }
                                p { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;",
                                    "Demo only · Merchant QR not connected"
                                }

                                div { style: "margin-top: 12px; border-top: 1px solid #f1f5f9; padding-top: 10px;",
                                    a {
                                        style: "font-size: 13px; font-weight: 700; color: var(--primary); text-decoration: none; cursor: pointer;",
                                        onclick: move |_| toast_msg.set(Some("Demo GCash QR saved to clipboard.".into())),
                                        "Save QR / Open GCash ↗"
                                    }
                                }
                            }

                            // 13-Digit Reference Input Group
                            div { class: "ref-input-group",
                                label { style: "display: block; font-size: 11px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; margin-bottom: 4px;", "GCash transaction reference" }
                                input {
                                    class: "search-input-clean",
                                    style: "font-size: 16px; font-weight: 700; font-family: monospace; letter-spacing: 1px; width: 100%;",
                                    value: "{gcash_ref}",
                                    oninput: move |e| gcash_ref.set(e.value().chars().filter(|c| c.is_ascii_digit()).take(13).collect())
                                }
                                p { style: "font-size: 11px; color: var(--text-muted); margin-top: 4px;",
                                    "Enter exactly 13 digits. This reference has {gcash_ref().len()}."
                                }
                            }

                            // Correct Format Example Box
                            div { class: "ref-example-box",
                                span { style: "display: block; font-size: 11px; font-weight: 700; color: var(--text-muted);", "Correct format example" }
                                strong { style: "font-family: monospace; font-size: 14px; color: var(--primary-dark); display: block; margin: 2px 0;", "1234567890123" }
                                span { style: "font-size: 11px; color: var(--text-muted);", "13/13 digits · Numbers only · No spaces or letters" }
                            }

                            // Submission Warning Callout
                            div { class: "cant-find-callout", style: "margin: 0 16px 20px 16px;",
                                strong { "🛡️ Submission is not payment approval" }
                                p { style: "margin-top: 4px;",
                                    "An admin checks the reference and amount manually. Your order cannot be assigned or shopped until approved."
                                }
                            }

                            button {
                                class: if gcash_ref().len() == 13 { "primary-btn" } else { "btn-checkout-disabled" },
                                style: "width: calc(100% - 32px); margin: 0 16px 80px 16px;",
                                onclick: move |_| {
                                    if gcash_ref().len() == 13 {
                                        subview.set(MarketSubView::PaymentVerification);
                                    } else {
                                        toast_msg.set(Some("Please enter the full 13-digit GCash reference.".into()));
                                    }
                                },
                                "Submit payment reference"
                            }
                        },

                        MarketSubView::PaymentVerification => rsx! {
                            // C15: Payment Verification Screen
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| subview.set(MarketSubView::Catalog),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Payment verification" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Direct Bankerohan Dispatch Audit: 0917-555-SUKI".into())),
                                    "?"
                                }
                            }

                            span { class: "order-stage-banner", "SKD-241108 · STAGE: MANUAL REVIEW · 9:34 AM" }
                            span { class: "status-pill-awaiting", "Under admin review" }

                            div { class: "clock-circle-icon", "🕒" }

                            div { style: "padding: 4px 16px 12px 16px;",
                                h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark); line-height: 1.2;",
                                    "We're checking your payment."
                                }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 6px;",
                                    "Your reference was submitted. Please don't pay again while we check it."
                                }
                            }

                            // Payment Submission Summary Card
                            div { class: "verification-card",
                                h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 8px;", "Payment submission" }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 4px;",
                                    span { "Amount submitted" }
                                    strong { "₱{grand_total}.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 4px;",
                                    span { "13-digit reference" }
                                    strong { style: "font-family: monospace;", "{gcash_ref}" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px;",
                                    span { "Submitted" }
                                    span { "Today · 9:34 AM" }
                                }
                                span { class: "locked-status-pill", "🔒 Assignment & shopping locked" }
                            }

                            // What happens next? Card
                            div { class: "cant-find-callout", style: "margin: 0 16px 12px 16px;",
                                strong { "🛡️ What happens next?" }
                                p { style: "margin-top: 4px;",
                                    "An admin verifies the GCash transfer. Once approved, we assign your shopper. You'll receive an update here."
                                }
                            }

                            // Other Verification Outcomes
                            div { class: "outcome-row-card",
                                strong { style: "color: #14532d; display: block; margin-bottom: 2px;", "Approved · 9:35 AM" }
                                p { style: "color: var(--text-muted);", "₱724 verified → eligible for assignment. This is the next lifecycle stage." }
                                div { style: "border-top: 1px solid #f1f5f9; margin-top: 8px; padding-top: 8px;",
                                    strong { style: "color: #b91c1c; display: block; margin-bottom: 2px;", "Rejected · correction needed" }
                                    p { style: "color: var(--text-muted);", "Alternate state: mismatched reference or amount. Correct and resubmit; shopping stays locked." }
                                }
                            }

                            button {
                                style: "background: none; border: 1px solid var(--border-color); border-radius: 12px; width: calc(100% - 32px); margin: 0 16px 80px 16px; padding: 12px; font-size: 13px; font-weight: 700; color: var(--text-main); cursor: pointer;",
                                onclick: move |_| active_tab.set("orders"),
                                "View live order tracking →"
                            }
                        }
                    }
                },

                "home" => rsx! {
                    div { class: "market-top-header",
                        span { class: "market-screen-heading", "Davao Market Hubs" }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("Davao City 24/7 Palengke Network".into())),
                            "?"
                        }
                    }
                    div { style: "padding: 16px;",
                        h3 { style: "font-weight: 800; font-size: 20px; color: var(--primary-dark);", "Suki Davao Hubs" }
                        p { style: "font-size: 13px; color: var(--text-muted); margin-top: 4px; margin-bottom: 16px;",
                            "Direct morning harvest across Davao's 3 primary trading post terminals."
                        }

                        div { class: "vendor-stall-card",
                            h4 { style: "font-size: 16px; font-weight: 800; color: var(--primary-dark);", "Bankerohan Central Wholesale" }
                            p { style: "font-size: 12px; color: var(--text-muted); margin-top: 4px;",
                                "24/7 highland vegetables from Marilog, fresh poultry, native eggs, and root crops."
                            }
                            span { class: "vendor-tag-badge", "148 Active Stalls · Open 24/7" }
                        }

                        div { class: "vendor-stall-card",
                            h4 { style: "font-size: 16px; font-weight: 800; color: var(--primary-dark);", "Agdao Fish Port Terminal" }
                            p { style: "font-size: 12px; color: var(--text-muted); margin-top: 4px;",
                                "Davao Gulf fresh yellowfin tuna, bangus, blue marlin, and live seafood arriving dawn."
                            }
                            span { class: "vendor-tag-badge", "86 Active Stalls · 4:00 AM – 7:30 PM" }
                        }

                        div { class: "vendor-stall-card",
                            h4 { style: "font-size: 16px; font-weight: 800; color: var(--primary-dark);", "Toril Fruit Gateway" }
                            p { style: "font-size: 12px; color: var(--text-muted); margin-top: 4px;",
                                "Foothills of Mt. Apo. Famous for Puyat durian, golden pomelo, and Lakatan bananas."
                            }
                            span { class: "vendor-tag-badge", "64 Active Stalls · 5:00 AM – 8:00 PM" }
                        }
                    }
                },

                "orders" => rsx! {
                    match order_stage() {
                        OrderScreenStage::DeliveryTracking => rsx! {
                            // C19: Delivery tracking
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| {
                                            active_tab.set("market");
                                            subview.set(MarketSubView::Catalog);
                                        },
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Track your delivery" }
                                }
                                div { style: "display: flex; gap: 8px; align-items: center;",
                                    button {
                                        style: "background: #f1f5f9; border: 1px solid var(--border-color); border-radius: 9999px; padding: 4px 10px; font-size: 11px; font-weight: 700; color: var(--primary-dark); cursor: pointer;",
                                        onclick: move |_| order_stage.set(OrderScreenStage::OrderHistory),
                                        "Order History"
                                    }
                                    button {
                                        class: "market-help-icon",
                                        onclick: move |_| toast_msg.set(Some("Davao GPS delivery route active.".into())),
                                        "?"
                                    }
                                }
                            }

                            span { class: "order-stage-banner", "SKD-241108 · STAGE: OUT FOR DELIVERY · 10:31 AM" }
                            span { class: "status-pill-success", "Jun is on the way" }

                            div { style: "padding: 4px 16px 12px 16px;",
                                h1 { style: "font-size: 28px; font-weight: 800; color: var(--primary-dark); line-height: 1.1;",
                                    "10:40–11:00 AM"
                                }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 4px;",
                                    "Estimated arrival · Your groceries are with Jun."
                                }
                            }

                            // Authentic Davao Street Map Route with Live Courier & Beacon
                            DavaoStreetMap {
                                mode: MapMode::Tracking,
                                shopper_name: "Jun Mercado".to_string(),
                                destination_label: "18 N. Torres St., Brgy. Obrero".to_string(),
                                distance: "4.00 km".to_string(),
                                eta: "10:40–11:00 AM".to_string(),
                                is_shopper_view: false,
                            }

                            // Personal Shopper Box
                            div { class: "rider-profile-box",
                                div { style: "display: flex; align-items: center; gap: 12px;",
                                    div { class: "rider-avatar-circle", "🛵" }
                                    div {
                                        span { style: "font-size: 10px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; display: block;", "Your Personal Shopper" }
                                        h4 { style: "font-size: 15px; font-weight: 800; color: var(--primary-dark); margin: 0;", "Jun Mercado" }
                                        span { style: "font-size: 11px; color: var(--text-muted);", "★ 4.9 · Verified · 320 market runs" }
                                    }
                                }
                                button {
                                    style: "background: none; border: none; font-size: 20px; cursor: pointer; padding: 6px;",
                                    onclick: move |_| order_stage.set(OrderScreenStage::ShopperChat),
                                    "💬"
                                }
                            }

                            button {
                                class: "btn-call-shopper",
                                onclick: move |_| order_stage.set(OrderScreenStage::ShopperChat),
                                "Call Jun / Send a message"
                            }

                            // Active Shopper Market Decisions (Placed where the interaction belongs)
                            div { class: "cant-find-callout", style: "margin: 0 16px 8px 16px; border-left: 4px solid #b45309; background: #fffbeb;",
                                div { style: "display: flex; justify-content: space-between; align-items: center;",
                                    div {
                                        strong { style: "color: #b45309; font-size: 13px; display: block;", "⚠️ Price adjustment pending" }
                                        p { style: "font-size: 11px; margin-top: 2px;", "Kamatis +₱10.00 at Aling Nena's stall" }
                                    }
                                    button {
                                        style: "background: #b45309; color: #ffffff; border: none; border-radius: 8px; padding: 6px 12px; font-size: 11px; font-weight: 700; cursor: pointer; white-space: nowrap;",
                                        onclick: move |_| order_stage.set(OrderScreenStage::PriceApproval),
                                        "Review →"
                                    }
                                }
                            }

                            div { class: "cant-find-callout", style: "margin: 0 16px 12px 16px; border-left: 4px solid var(--primary); background: #f0fdf4;",
                                div { style: "display: flex; justify-content: space-between; align-items: center;",
                                    div {
                                        strong { style: "color: var(--primary-dark); font-size: 13px; display: block;", "🔄 Substitution suggested" }
                                        p { style: "font-size: 11px; margin-top: 2px;", "Pechay out of stock · Jun suggested Kangkong (₱25.00)" }
                                    }
                                    button {
                                        style: "background: var(--primary); color: #ffffff; border: none; border-radius: 8px; padding: 6px 12px; font-size: 11px; font-weight: 700; cursor: pointer; white-space: nowrap;",
                                        onclick: move |_| order_stage.set(OrderScreenStage::SubstitutionApproval),
                                        "Review →"
                                    }
                                }
                            }

                            // 5-Step Order Progress Timeline
                            div { class: "timeline-card-box",
                                h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 8px;", "Your market run" }
                                div { class: "timeline-row-item",
                                    span { "✓ Payment manually verified" }
                                    span { style: "color: var(--text-muted);", "9:35" }
                                }
                                div { class: "timeline-row-item",
                                    span { "✓ Shopper assigned" }
                                    span { style: "color: var(--text-muted);", "9:38" }
                                }
                                div { class: "timeline-row-item",
                                    span { "✓ Shopping & decisions complete" }
                                    span { style: "color: var(--text-muted);", "10:12" }
                                }
                                div { class: "timeline-row-item", style: "font-weight: 700; color: #16a34a;",
                                    span { "● Out for delivery" }
                                    span { "10:31" }
                                }
                                div { class: "timeline-row-item", style: "color: var(--text-muted);",
                                    span { "○ Delivered" }
                                    span { "Next" }
                                }
                            }

                            // Delivery Address Card
                            div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                                h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark);", "Ana · Home" }
                                p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;",
                                    "18 N. Torres St., Brgy. Obrero, Davao City · Green gate"
                                }
                            }

                            // Final Charged Card (Clickable to jump to C20 Refund Notification)
                            div {
                                class: "final-charge-card",
                                style: "cursor: pointer;",
                                onclick: move |_| order_stage.set(OrderScreenStage::RefundNotification),
                                div { style: "display: flex; justify-content: space-between; align-items: center;",
                                    strong { "✓ Final charged · ₱664.00" }
                                    span { style: "font-size: 11px; font-weight: 700; text-decoration: underline;", "View Refund →" }
                                }
                                p { style: "margin-top: 4px; font-size: 11px;",
                                    "₱590 actual groceries + ₱74 fixed delivery. Refundable excess: ₱60. Refund processing is separate from this calculation."
                                }
                            }
                        },

                        OrderScreenStage::PriceApproval => rsx! {
                            // C16: Price approval (Dark green background)
                            div { class: "decision-dark-screen",
                                div { class: "market-top-header", style: "background: transparent; color: #ffffff; padding: 0 0 12px 0;",
                                    div { class: "market-header-left",
                                        button {
                                            class: "market-back-btn",
                                            style: "color: #ffffff; border-color: rgba(255,255,255,0.3);",
                                            onclick: move |_| order_stage.set(OrderScreenStage::DeliveryTracking),
                                            "←"
                                        }
                                        span { class: "market-screen-heading", style: "color: #ffffff;", "Your market run" }
                                    }
                                    button {
                                        class: "market-help-icon",
                                        style: "color: #ffffff; border-color: rgba(255,255,255,0.3);",
                                        onclick: move |_| toast_msg.set(Some("Price Guarantee: Shoppers never purchase price increases without approval.".into())),
                                        "?"
                                    }
                                }

                                span { style: "font-size: 11px; font-weight: 700; color: #a7f3d0; letter-spacing: 0.5px; text-transform: uppercase;",
                                    "SKD-241108 · STAGE: SHOPPING / APPROVAL · 9:52 AM"
                                }

                                div { class: "decision-card-overlay",
                                    span { class: "status-pill-awaiting", "PRICE ADJUSTMENT · WAITING" }
                                    h2 { style: "font-size: 20px; font-weight: 800; color: var(--primary-dark); margin-top: 10px; line-height: 1.2;",
                                        "Tomatoes cost a little more today."
                                    }

                                    div { style: "display: flex; align-items: center; gap: 10px; margin-top: 12px; padding: 10px; background: #f8fafc; border-radius: 10px;",
                                        img {
                                            src: "{images::TOMATOES_IMG}",
                                            style: "width: 44px; height: 44px; border-radius: 8px; object-fit: cover;"
                                        }
                                        div {
                                            strong { style: "font-size: 13px; color: var(--text-main); display: block;", "Fresh tomatoes · Kamatis" }
                                            span { style: "font-size: 11px; color: var(--text-muted);", "Nena's Gulayan · V-24" }
                                        }
                                    }

                                    div { style: "margin-top: 14px; font-size: 13px;",
                                        div { style: "display: flex; justify-content: space-between; padding: 4px 0;",
                                            span { "Estimate · 1 kg × ₱80" }
                                            strong { "₱80.00" }
                                        }
                                        div { style: "display: flex; justify-content: space-between; padding: 4px 0;",
                                            span { "Proposed · 1 kg × ₱90" }
                                            strong { "₱90.00" }
                                        }
                                        div { style: "display: flex; justify-content: space-between; padding: 4px 0; border-top: 1px dashed var(--border-color); margin-top: 4px; padding-top: 6px;",
                                            strong { "Increase" }
                                            strong { style: "color: #b45309;", "+₱10.00" }
                                        }
                                    }

                                    div { class: "shopper-bubble-quote",
                                        strong { "Jun: " }
                                        "\"Today's stall price is ₱90/kg. May I buy these ripe tomatoes?\""
                                    }

                                    p { style: "font-size: 11px; color: var(--text-muted); margin-top: 6px; line-height: 1.3;",
                                        "Covered by verified grocery funds. No top-up needed for this item."
                                    }

                                    button {
                                        class: "primary-btn",
                                        style: "width: 100%; margin-top: 14px;",
                                        onclick: move |_| {
                                            toast_msg.set(Some("Approved ₱90.00 purchase for tomatoes! Jun is checking out.".into()));
                                            order_stage.set(OrderScreenStage::DeliveryTracking);
                                        },
                                        "Approve ₱90.00 purchase"
                                    }
                                    button {
                                        class: "btn-outline-danger",
                                        onclick: move |_| {
                                            toast_msg.set(Some("Rejected price adjustment. Tomatoes removed from market run.".into()));
                                            order_stage.set(OrderScreenStage::DeliveryTracking);
                                        },
                                        "Reject · remove item"
                                    }
                                }

                                p { style: "font-size: 11px; color: rgba(255,255,255,0.7); text-align: center; margin: 8px 16px 24px 16px; line-height: 1.3;",
                                    "Response by 10:07 AM. Rejected or timed-out requests are not purchased automatically."
                                }
                            }
                        },

                        OrderScreenStage::SubstitutionApproval => rsx! {
                            // C17: Substitution approval (Dark green background)
                            div { class: "decision-dark-screen",
                                div { class: "market-top-header", style: "background: transparent; color: #ffffff; padding: 0 0 12px 0;",
                                    div { class: "market-header-left",
                                        button {
                                            class: "market-back-btn",
                                            style: "color: #ffffff; border-color: rgba(255,255,255,0.3);",
                                            onclick: move |_| order_stage.set(OrderScreenStage::DeliveryTracking),
                                            "←"
                                        }
                                        span { class: "market-screen-heading", style: "color: #ffffff;", "Your market run" }
                                    }
                                    button {
                                        class: "market-help-icon",
                                        style: "color: #ffffff; border-color: rgba(255,255,255,0.3);",
                                        onclick: move |_| toast_msg.set(Some("Substitutions are strictly approved by you first.".into())),
                                        "?"
                                    }
                                }

                                span { style: "font-size: 11px; font-weight: 700; color: #a7f3d0; letter-spacing: 0.5px; text-transform: uppercase;",
                                    "SKD-241108 · STAGE: SUBSTITUTION REVIEW · 9:57 AM"
                                }

                                div { class: "decision-card-overlay",
                                    span { class: "status-pill-awaiting", "REPLACEMENT · WAITING" }
                                    h2 { style: "font-size: 20px; font-weight: 800; color: var(--primary-dark); margin-top: 10px; line-height: 1.2;",
                                        "Pechay is unavailable."
                                    }
                                    p { style: "font-size: 13px; color: var(--text-muted); margin-top: 6px; line-height: 1.3;",
                                        "Jun found fresh kangkong at the same accredited stall. Would you like it instead?"
                                    }

                                    div { class: "sub-compare-box",
                                        strong { style: "font-size: 14px; color: var(--primary-dark); display: block; margin-bottom: 8px;",
                                            "Native pechay → Kangkong"
                                        }
                                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                                            span { "Original · 0.50 kg" }
                                            strong { "₱30.00" }
                                        }
                                        div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                                            span { "Replacement · 1 bunch" }
                                            strong { "₱25.00" }
                                        }
                                        div { style: "margin-top: 6px;",
                                            span { class: "tag-pill-green", "₱5 less · Approval still required" }
                                        }
                                    }

                                    p { style: "font-size: 11px; color: var(--text-muted); line-height: 1.3;",
                                        "Aling Nena's Gulayan · V-24. No purchase is made until you approve."
                                    }

                                    button {
                                        class: "primary-btn",
                                        style: "width: 100%; margin-top: 14px;",
                                        onclick: move |_| {
                                            toast_msg.set(Some("Approved kangkong substitution (₱25.00)!".into()));
                                            order_stage.set(OrderScreenStage::DeliveryTracking);
                                        },
                                        "Approve kangkong · ₱25.00"
                                    }
                                    button {
                                        class: "btn-outline-danger",
                                        onclick: move |_| {
                                            toast_msg.set(Some("Substitution declined. Pechay removed from list.".into()));
                                            order_stage.set(OrderScreenStage::DeliveryTracking);
                                        },
                                        "No thanks · remove pechay"
                                    }
                                }

                                p { style: "font-size: 11px; color: rgba(255,255,255,0.7); text-align: center; margin: 8px 16px 24px 16px; line-height: 1.3;",
                                    "Rejecting removes both items from your bill. No answer by 10:12? Skip, don't buy."
                                }
                            }
                        },

                        OrderScreenStage::FundingTopUp => rsx! {
                            // C18: Funding before purchase (Top-up)
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| order_stage.set(OrderScreenStage::DeliveryTracking),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Funding before purchase" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Additional funds required for basket increase.".into())),
                                    "?"
                                }
                            }

                            span { class: "order-stage-banner", "SKD-241108 · ALTERNATE BRANCH · NOT THE FINAL RECEIPT" }
                            span { class: "status-pill-awaiting", "Approved change · Purchase on hold" }

                            div { style: "padding: 4px 16px 12px 16px;",
                                h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark); line-height: 1.2;",
                                    "Approval isn't a top-up."
                                }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 6px; line-height: 1.4;",
                                    "If your revised basket exceeds paid funds, we pause the affected purchases until extra payment is manually verified."
                                }
                            }

                            // Alternate higher-cost calculation card
                            div { class: "vendor-stall-card", style: "margin: 0 16px 12px 16px;",
                                h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 8px;", "Alternate higher-cost basket" }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 3px 0;",
                                    span { "Proposed approved groceries" }
                                    strong { "₱680.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 3px 0;",
                                    span { "Fixed delivery" }
                                    strong { "₱74.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 3px 0; border-top: 1px solid #f1f5f9; margin-top: 4px; padding-top: 6px;",
                                    strong { "Revised total" }
                                    strong { "₱754.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 3px 0;",
                                    span { "Verified funds" }
                                    span { "₱724.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 14px; padding: 6px 0; border-top: 2px solid var(--border-color); margin-top: 6px; color: var(--primary);",
                                    strong { "Top-up required" }
                                    strong { "₱30.00" }
                                }
                            }

                            div { class: "cant-find-callout", style: "margin: 0 16px 12px 16px;",
                                strong { "ⓘ Do not purchase yet" }
                                p { style: "margin-top: 4px; line-height: 1.3;",
                                    "Customer approval records permission, not funds. Top-up uses official GCash QR and a new 13-digit reference. Admin approval is required before purchase."
                                }
                            }

                            button {
                                class: "primary-btn",
                                style: "width: calc(100% - 32px); margin: 0 16px 8px 16px;",
                                onclick: move |_| {
                                    toast_msg.set(Some("Top-up reference submitted! ₱30.00 under review.".into()));
                                    order_stage.set(OrderScreenStage::DeliveryTracking);
                                },
                                "Continue to demo GCash · ₱30.00"
                            }

                            button {
                                class: "btn-outline-danger",
                                style: "width: calc(100% - 32px); margin: 0 16px 14px 16px;",
                                onclick: move |_| {
                                    toast_msg.set(Some("Increase declined. Reverting to original verified items.".into()));
                                    order_stage.set(OrderScreenStage::DeliveryTracking);
                                },
                                "Decline increase & remove affected item"
                            }

                            // Funding States Reference Card
                            div { class: "outcome-row-card",
                                strong { style: "color: var(--primary-dark); display: block; margin-bottom: 4px;", "Funding states" }
                                div { style: "padding: 4px 0;",
                                    span { class: "tag-pill-green", style: "background: #fef3c7; color: #92400e;", "Awaiting top-up / Under review" }
                                    p { style: "margin-top: 4px; color: var(--text-muted);", "Hold purchase until ₱30 is verified." }
                                }
                                div { style: "border-top: 1px solid #f1f5f9; padding-top: 6px; margin-top: 6px;",
                                    strong { style: "color: #14532d; font-size: 12px;", "Top-up verified → Purchase eligible" }
                                }
                                div { style: "border-top: 1px solid #f1f5f9; padding-top: 6px; margin-top: 6px;",
                                    p { style: "color: var(--text-muted);", "Rejected, expired or unpaid request → remove the affected item, never auto-buy." }
                                }
                            }
                        },

                        OrderScreenStage::RefundNotification => rsx! {
                            // C20: Refund notification
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| order_stage.set(OrderScreenStage::DeliveryTracking),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Your refund update" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Refund calculation: Automatic and verified.".into())),
                                    "?"
                                }
                            }

                            span { class: "order-stage-banner", "SKD-241108 · STAGE: DELIVERED / REFUND PROCESSING · 11:05 AM" }
                            span { class: "status-pill-success", "Refund processing" }

                            div { style: "padding: 4px 16px 12px 16px;",
                                h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark); line-height: 1.2;",
                                    "Your ₱60.00 is accounted for."
                                }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 6px;",
                                    "You only pay for the groceries you received and your fixed delivery fee."
                                }
                            }

                            // Math Breakdown Card
                            div { class: "refund-math-card",
                                h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 8px;", "Calculated automatically" }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 3px 0;",
                                    span { "Verified payment" }
                                    strong { "₱724.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 3px 0;",
                                    span { "Final charge" }
                                    strong { "- ₱664.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 15px; padding: 8px 0; border-top: 2px solid var(--border-color); margin-top: 6px; color: #15803d;",
                                    strong { "Refundable excess" }
                                    strong { "₱60.00" }
                                }
                            }

                            // Refund Processing Timeline
                            div { class: "refund-stage-tracker",
                                h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 8px;", "Refund RF-241108-01" }
                                div { class: "timeline-row-item",
                                    span { "✓ Calculated · pending processing" }
                                    span { style: "color: var(--text-muted);", "10:48" }
                                }
                                div { class: "timeline-row-item", style: "font-weight: 700; color: #16a34a;",
                                    span { "● Finance processing" }
                                    span { "11:05" }
                                }
                                div { class: "timeline-row-item", style: "color: var(--text-muted);",
                                    span { "○ Sent to GCash" }
                                    span { "Pending" }
                                }
                                div { style: "border-top: 1px solid #f1f5f9; margin-top: 8px; padding-top: 8px; font-size: 12px; color: var(--text-muted);",
                                    p { strong { "Destination: " } "GCash · 0917 ••• 4826" }
                                    p { style: "margin-top: 4px; font-size: 11px;", "Expected 1-2 business days. You'll be notified when the transfer is recorded as sent." }
                                }
                            }

                            div { class: "cant-find-callout", style: "margin: 0 16px 14px 16px;",
                                strong { "ⓘ No request needed" }
                                p { style: "margin-top: 4px; line-height: 1.3;",
                                    "The refundable amount is calculated automatically. It is not marked sent until finance processes and records the GCash transfer."
                                }
                            }

                            button {
                                class: "primary-btn",
                                style: "width: calc(100% - 32px); margin: 0 16px 8px 16px;",
                                onclick: move |_| order_stage.set(OrderScreenStage::DigitalReceipt),
                                "View itemized receipt"
                            }

                            button {
                                style: "background: #ffffff; border: 1px solid var(--border-color); border-radius: 12px; width: calc(100% - 32px); margin: 0 16px 80px 16px; padding: 12px; font-size: 13px; font-weight: 700; color: var(--text-main); cursor: pointer;",
                                onclick: move |_| toast_msg.set(Some("Connected to Suki Refund Support: 0917-555-SUKI".into())),
                                "Ask about my refund"
                            }
                        },

                        OrderScreenStage::ShopperChat => rsx! {
                            // C21: Shopper chat (Encrypted order chat)
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| order_stage.set(OrderScreenStage::DeliveryTracking),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Chat with Jun" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Order chat is end-to-end encrypted for your safety.".into())),
                                    "?"
                                }
                            }

                            span { class: "order-stage-banner", "SKD-241108 · STAGE: SHOPPING · ENCRYPTED ORDER CHAT" }

                            // Shopper Card
                            div { class: "rider-profile-box",
                                div { style: "display: flex; align-items: center; gap: 12px;",
                                    div { class: "rider-avatar-circle", "🛵" }
                                    div {
                                        span { style: "font-size: 10px; font-weight: 700; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; display: block;", "Your Personal Shopper" }
                                        h4 { style: "font-size: 15px; font-weight: 800; color: var(--primary-dark); margin: 0;", "Jun Mercado" }
                                        span { style: "font-size: 11px; color: var(--text-muted);", "★ 4.9 · Verified · 320 market runs" }
                                    }
                                }
                                span { style: "font-size: 18px; color: var(--text-muted);", "💬" }
                            }

                            div { class: "chat-container",
                                span { class: "chat-date-chip", "TODAY · ORDER SKD-241108 · 9:52 AM" }

                                // Message 1: Jun
                                div { class: "chat-bubble-shopper",
                                    p { "Maayong buntag, Ana! Kamatis is ₱90/kg today. I sent a price request for you to review." }
                                    span { class: "chat-timestamp-shopper", "Jun · 9:52 AM" }
                                }

                                // Message 2: Ana
                                div { class: "chat-bubble-customer",
                                    p { "Salamat, Jun. I'll approve the tomatoes. Please skip the kangkong replacement." }
                                    span { class: "chat-timestamp-customer", "Ana · 9:55 AM · Read" }
                                }

                                // Callout Notice
                                div { class: "chat-callout-box",
                                    strong { "✓ Approval recorded separately" }
                                    p { style: "margin-top: 4px;",
                                        "Chat messages do not authorize purchases. Use the order approval card to confirm a price or substitution."
                                    }
                                }

                                // Audit Card
                                div { class: "chat-audit-box",
                                    div { style: "display: flex; justify-content: space-between; font-size: 12px; padding: 3px 0;",
                                        strong { style: "color: #15803d;", "✓ Tomatoes approved · 9:56" }
                                        span { "₱90.00 · 1.00 kg = ₱90/kg" }
                                    }
                                    div { style: "display: flex; justify-content: space-between; font-size: 12px; padding: 3px 0; border-top: 1px dashed var(--border-color); margin-top: 4px; padding-top: 4px;",
                                        strong { style: "color: #b91c1c;", "✕ Kangkong rejected · 9:58" }
                                        span { "Not purchased · ₱0.00 billed" }
                                    }
                                }

                                // Message 3: Jun
                                div { class: "chat-bubble-shopper",
                                    p { "Got it, suki! I'll only purchase approved items within your verified funds." }
                                    span { class: "chat-timestamp-shopper", "Jun · 10:02 AM" }
                                }

                                // Any user typed messages
                                for (idx, msg) in extra_chat_msgs().iter().enumerate() {
                                    div { key: "{idx}", class: "chat-bubble-customer",
                                        p { "{msg}" }
                                        span { class: "chat-timestamp-customer", "Ana · Just now · Sent" }
                                    }
                                }
                            }

                            // Chat Input Bar
                            div { class: "chat-input-bar",
                                input {
                                    class: "chat-input-field",
                                    placeholder: "✏️ Write a message...",
                                    value: "{chat_input}",
                                    oninput: move |e| chat_input.set(e.value()),
                                }
                                button {
                                    class: "btn-chat-send",
                                    onclick: move |_| {
                                        let txt = chat_input().trim().to_string();
                                        if !txt.is_empty() {
                                            extra_chat_msgs.write().push(txt);
                                            chat_input.set(String::new());
                                        }
                                    },
                                    "➤"
                                }
                            }
                        },

                        OrderScreenStage::DigitalReceipt => rsx! {
                            // C22: Digital receipt
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| order_stage.set(OrderScreenStage::DeliveryTracking),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Your digital receipt" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Official electronic billing & Davao BIR audit compliance.".into())),
                                    "?"
                                }
                            }

                            span { class: "order-stage-banner", "SKD-241108 · STAGE: DELIVERED · NOV 08, 10:48 AM" }
                            span { class: "status-pill-success", "Delivered · 4 items received" }

                            div { style: "padding: 4px 16px 12px 16px;",
                                h1 { style: "font-size: 26px; font-weight: 800; color: var(--primary-dark); line-height: 1.1;",
                                    "Salamat, suki!"
                                }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 4px; line-height: 1.3;",
                                    "Delivered to Ana Santos by Jun Mercado, 18 N. Torres St., Obrero, Davao City."
                                }
                            }

                            // From Estimate to Actual Card
                            div { class: "receipt-card",
                                h3 { style: "font-size: 16px; font-weight: 800; color: var(--primary-dark);", "From estimate to actual" }
                                p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px; margin-bottom: 12px;",
                                    "Estimate vs. Actual · Every change is recorded"
                                }

                                // Item 1: Tomatoes
                                div { class: "receipt-item-row",
                                    div { style: "display: flex; align-items: center; gap: 10px;",
                                        img { src: "{images::TOMATOES_IMG}", style: "width: 38px; height: 38px; border-radius: 8px; object-fit: cover;" }
                                        div {
                                            strong { style: "font-size: 13px; color: var(--text-main); display: block;", "Tomatoes · Kamatis" }
                                            span { style: "font-size: 11px; color: var(--text-muted);", "1.00 kg × ₱90 · approved 9:56" }
                                        }
                                    }
                                    strong { style: "font-size: 13px;", "₱90.00" }
                                }

                                // Item 2: Carrots
                                div { class: "receipt-item-row",
                                    div { style: "display: flex; align-items: center; gap: 10px;",
                                        img { src: "{images::CARROTS_IMG}", style: "width: 38px; height: 38px; border-radius: 8px; object-fit: cover;" }
                                        div {
                                            strong { style: "font-size: 13px; color: var(--text-main); display: block;", "Carrots · Karot" }
                                            span { style: "font-size: 11px; color: var(--text-muted);", "0.50 kg × ₱120 · unchanged" }
                                        }
                                    }
                                    strong { style: "font-size: 13px;", "₱60.00" }
                                }

                                // Item 3: Pechay (Excluded)
                                div { class: "receipt-item-row",
                                    div { style: "display: flex; align-items: center; gap: 10px; opacity: 0.6;",
                                        img { src: "{images::CABBAGE_IMG}", style: "width: 38px; height: 38px; border-radius: 8px; object-fit: cover;" }
                                        div {
                                            strong { style: "font-size: 13px; color: var(--text-muted); text-decoration: line-through; display: block;", "Native pechay" }
                                            span { style: "font-size: 11px; color: #b91c1c;", "Unavailable · kangkong rejected · excluded" }
                                        }
                                    }
                                    span { style: "font-size: 13px; color: var(--text-muted);", "₱0.00" }
                                }

                                // Item 4: Chicken
                                div { class: "receipt-item-row",
                                    div { style: "display: flex; align-items: center; gap: 10px;",
                                        img { src: "{images::CHICKEN_IMG}", style: "width: 38px; height: 38px; border-radius: 8px; object-fit: cover;" }
                                        div {
                                            strong { style: "font-size: 13px; color: var(--text-main); display: block;", "Chicken · Manok" }
                                            span { style: "font-size: 11px; color: var(--text-muted);", "1.40 kg × ₱250 · approved 10:02" }
                                        }
                                    }
                                    strong { style: "font-size: 13px;", "₱350.00" }
                                }

                                // Item 5: Bananas
                                div { class: "receipt-item-row",
                                    div { style: "display: flex; align-items: center; gap: 10px;",
                                        img { src: "{images::BANANAS_IMG}", style: "width: 38px; height: 38px; border-radius: 8px; object-fit: cover;" }
                                        div {
                                            strong { style: "font-size: 13px; color: var(--text-main); display: block;", "Lakatan bananas" }
                                            span { style: "font-size: 11px; color: var(--text-muted);", "0.75 kg × ₱120 · actual weight" }
                                        }
                                    }
                                    strong { style: "font-size: 13px;", "₱90.00" }
                                }

                                // Subtotals
                                div { style: "border-top: 1px solid #e2e8f0; margin-top: 10px; padding-top: 10px;",
                                    div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                                        span { "Original groceries estimate" }
                                        span { "₱650.00" }
                                    }
                                    div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                                        strong { "Actual groceries" }
                                        strong { "₱590.00" }
                                    }
                                }

                                p { style: "font-size: 11px; color: var(--text-muted); margin-top: 8px; line-height: 1.35; background: #f8fafc; padding: 8px; border-radius: 8px;",
                                    "Price increases approved before purchase: tomatoes 9:56, chicken 10:02. Kangkong rejected 9:58 ➔ excluded."
                                }
                            }

                            // Final Settlement Card
                            div { class: "receipt-card",
                                h4 { style: "font-size: 15px; font-weight: 800; color: var(--primary-dark); margin-bottom: 8px;", "Final settlement" }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 3px 0;",
                                    span { "Actual groceries · 4 purchased" }
                                    strong { "₱590.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 3px 0;",
                                    span { "Delivery · 4.00 km" }
                                    strong { "₱74.00" }
                                }
                                p { style: "font-size: 11px; color: var(--text-muted); margin: 2px 0 8px 0;",
                                    "₱50 base + ₱6.00 × 4 km · No extra shopper fee"
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 16px; padding: 8px 0; border-top: 2px solid var(--border-color); color: var(--primary-dark);",
                                    strong { "Final charged" }
                                    strong { "₱664.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px; padding: 2px 0;",
                                    span { "Verified GCash payment" }
                                    span { "₱724.00" }
                                }
                                div { style: "display: flex; justify-content: space-between; font-size: 14px; padding: 4px 0; color: #15803d;",
                                    strong { "Refundable excess" }
                                    strong { "₱60.00" }
                                }
                            }

                            // Refund Box
                            div { class: "refund-stage-tracker",
                                h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 4px;", "Refund · RF-241108-01" }
                                div { style: "display: flex; justify-content: space-between; font-size: 13px;",
                                    span { "₱724 paid - ₱664 charged" }
                                    strong { style: "color: #15803d;", "₱60.00" }
                                }
                                span { class: "tag-pill-green", style: "margin-top: 8px; display: inline-block;", "Calculated · Pending processing" }
                                p { style: "font-size: 11px; color: var(--text-muted); margin-top: 6px;",
                                    "GCash 0917 ••• 4826 · Calculation does not mean sent."
                                }
                            }

                            // Interactive Star Rating Box
                            div { class: "star-rating-box",
                                span { style: "display: block; font-weight: 700; color: var(--primary-dark); margin-bottom: 6px;", "How was your market run?" }
                                div { style: "font-size: 22px; cursor: pointer; color: #f59e0b; display: flex; justify-content: center; gap: 4px;",
                                    for star in 1..=5 {
                                        span {
                                            key: "{star}",
                                            onclick: move |_| {
                                                star_rating.set(star);
                                                toast_msg.set(Some(format!("Salamat sa feedback! Rated {} stars ★", star)));
                                            },
                                            if star <= star_rating() { "★" } else { "☆" }
                                        }
                                    }
                                }
                            }

                            button {
                                class: "primary-btn",
                                style: "width: calc(100% - 32px); margin: 0 16px 8px 16px;",
                                onclick: move |_| toast_msg.set(Some("Official receipt PDF downloaded: SKD-241108-RECEIPT.pdf".into())),
                                "Download itemized receipt ↓"
                            }

                            button {
                                style: "background: #ffffff; border: 1px solid var(--border-color); border-radius: 12px; width: calc(100% - 32px); margin: 0 16px 80px 16px; padding: 12px; font-size: 13px; font-weight: 700; color: var(--text-main); cursor: pointer;",
                                onclick: move |_| {
                                    active_tab.set("market");
                                    subview.set(MarketSubView::Catalog);
                                },
                                "Shop fresh again"
                            }
                        },

                        OrderScreenStage::OrderHistory => rsx! {
                            // C23: Order history
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| order_stage.set(OrderScreenStage::DeliveryTracking),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "Your orders" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Past orders and instant re-ordering.".into())),
                                    "?"
                                }
                            }

                            div { style: "padding: 8px 16px 4px 16px;",
                                h1 { style: "font-size: 26px; font-weight: 800; color: var(--primary-dark); line-height: 1.15;",
                                    "Good food. Good memories."
                                }
                            }

                            // Filter Chips
                            div { class: "history-filter-bar",
                                for &tab in &["All orders", "In progress", "Completed"] {
                                    button {
                                        class: if order_history_filter() == tab { "stage-chip-btn active" } else { "stage-chip-btn" },
                                        onclick: move |_| order_history_filter.set(tab),
                                        "{tab}"
                                    }
                                }
                            }

                            // Order 1: Today SKD-241108
                            div { class: "history-card",
                                div { style: "display: flex; justify-content: space-between; align-items: center;",
                                    strong { style: "font-size: 15px; color: var(--primary-dark);", "Today · SKD-241108" }
                                    span { class: "tag-pill-green", "Delivered · 10:48 AM" }
                                }
                                div { class: "history-thumbs-row",
                                    img { class: "history-thumb-img", src: "{images::TOMATOES_IMG}" }
                                    img { class: "history-thumb-img", src: "{images::CARROTS_IMG}" }
                                    img { class: "history-thumb-img", src: "{images::BANANAS_IMG}" }
                                }
                                p { style: "font-size: 12px; color: var(--text-muted);", "4 purchased items · Jun Mercado · Obrero" }
                                div { style: "display: flex; justify-content: space-between; align-items: center; margin-top: 6px;",
                                    span { style: "font-size: 13px; color: var(--text-muted);", "Final charged" }
                                    strong { style: "font-size: 16px; color: var(--primary-dark);", "₱664.00" }
                                }
                                p { style: "font-size: 11px; color: #15803d; margin: 4px 0 10px 0;", "₱60 refund · Processing at 11:05" }
                                button {
                                    class: "primary-btn",
                                    style: "width: 100%;",
                                    onclick: move |_| order_stage.set(OrderScreenStage::DigitalReceipt),
                                    "View receipt & refund"
                                }
                            }

                            // Order 2: Nov 03 SKD-241103
                            div { class: "history-card",
                                div { style: "display: flex; justify-content: space-between; align-items: center;",
                                    strong { style: "font-size: 15px; color: var(--primary-dark);", "Nov 03 · SKD-241103" }
                                    span { class: "tag-pill-green", "Completed" }
                                }
                                p { style: "font-size: 12px; color: var(--text-muted); margin-top: 8px;", "6 market items · Mang Lito's Harvest & 2 stalls" }
                                div { style: "display: flex; justify-content: space-between; align-items: center; margin-top: 6px;",
                                    span { style: "font-size: 13px; color: var(--text-muted);", "Final charged" }
                                    strong { style: "font-size: 16px; color: var(--primary-dark);", "₱812.50" }
                                }
                                p { style: "font-size: 11px; color: var(--text-muted); margin: 4px 0 10px 0;", "Independent previous order · Delivered 11:12 AM" }
                                button {
                                    style: "width: 100%; background: #ffffff; border: 1px solid var(--border-color); border-radius: 12px; padding: 10px; font-size: 13px; font-weight: 700; color: var(--text-main); cursor: pointer;",
                                    onclick: move |_| toast_msg.set(Some("Added items from Nov 03 order to cart!".into())),
                                    "Buy again"
                                }
                            }

                            // Order 3: Oct 29 SKD-241029
                            div { class: "history-card", style: "margin-bottom: 80px;",
                                div { style: "display: flex; justify-content: space-between; align-items: center;",
                                    strong { style: "font-size: 15px; color: var(--primary-dark);", "Oct 29 · SKD-241029" }
                                    span { class: "tag-pill-green", style: "background: #fee2e2; color: #b91c1c;", "Cancelled · Payment not submitted" }
                                }
                                p { style: "font-size: 12px; color: var(--text-muted); margin: 8px 0 10px 0;", "No shopping started. No payment collected." }
                                button {
                                    style: "width: 100%; background: #ffffff; border: 1px solid var(--border-color); border-radius: 12px; padding: 10px; font-size: 13px; font-weight: 700; color: var(--text-main); cursor: pointer;",
                                    onclick: move |_| toast_msg.set(Some("SKD-241029: Timed out awaiting GCash reference.".into())),
                                    "View order"
                                }
                            }
                        },

                        OrderScreenStage::RecoveryStates => rsx! {
                            // C25: First-time & recovery states
                            div { class: "market-top-header",
                                div { class: "market-header-left",
                                    button {
                                        class: "market-back-btn",
                                        onclick: move |_| order_stage.set(OrderScreenStage::DeliveryTracking),
                                        "←"
                                    }
                                    span { class: "market-screen-heading", "First-time & recovery states" }
                                }
                                button {
                                    class: "market-help-icon",
                                    onclick: move |_| toast_msg.set(Some("Empty, loading, and recovery states guide.".into())),
                                    "?"
                                }
                            }

                            div { style: "padding: 8px 16px 4px 16px;",
                                h1 { style: "font-size: 24px; font-weight: 800; color: var(--primary-dark); line-height: 1.2;",
                                    "Thoughtful at every step."
                                }
                                p { style: "font-size: 13px; color: var(--text-muted); margin-top: 4px;",
                                    "Supporting customer states · Distinct static panels"
                                }
                            }

                            // State Panel 1: First visit · No delivery address
                            div { class: "state-panel-card",
                                strong { style: "font-size: 13px; color: var(--primary-dark); display: block; margin-bottom: 6px;", "First visit · No delivery address" }
                                div { style: "font-size: 24px; margin-bottom: 6px;", "📍" }
                                p { style: "font-size: 13px; color: var(--text-muted); line-height: 1.35; margin-bottom: 12px;",
                                    "Maayong pag-abot! Add your Davao City address to see your delivery fee."
                                }
                                button {
                                    class: "primary-btn",
                                    style: "width: 100%;",
                                    onclick: move |_| {
                                        active_tab.set("market");
                                        subview.set(MarketSubView::AddressMapPicker);
                                    },
                                    "Add my first address"
                                }
                            }

                            // State Panel 2: Loading · Finding fresh picks
                            div { class: "state-panel-card",
                                strong { style: "font-size: 13px; color: var(--primary-dark); display: block; margin-bottom: 8px;", "Loading · Finding fresh picks" }
                                div { class: "skeleton-bar", style: "width: 90%;" }
                                div { class: "skeleton-bar", style: "width: 75%;" }
                                div { class: "skeleton-bar", style: "width: 60%;" }
                                p { style: "font-size: 11px; color: var(--text-muted); margin-top: 8px;",
                                    "Refreshing today's market availability..."
                                }
                            }

                            // State Panel 3: Empty basket · A fresh start
                            div { class: "state-panel-card",
                                strong { style: "font-size: 13px; color: var(--primary-dark); display: block; margin-bottom: 6px;", "Empty basket · A fresh start" }
                                div { style: "font-size: 24px; margin-bottom: 6px;", "🧺" }
                                p { style: "font-size: 13px; color: var(--text-muted); line-height: 1.35; margin-bottom: 12px;",
                                    "Your basket is waiting for something fresh. Start with your everyday favorites."
                                }
                                button {
                                    style: "width: 100%; background: #ffffff; border: 1px solid var(--border-color); border-radius: 12px; padding: 10px; font-size: 13px; font-weight: 700; color: var(--text-main); cursor: pointer;",
                                    onclick: move |_| {
                                        active_tab.set("market");
                                        subview.set(MarketSubView::Catalog);
                                    },
                                    "Explore Bankerohan"
                                }
                            }

                            // State Panel 4: No search results
                            div { class: "state-panel-card",
                                strong { style: "font-size: 13px; color: var(--primary-dark); display: block; margin-bottom: 6px;", "No search results · \"Dragonfruit jam\"" }
                                p { style: "font-size: 13px; color: var(--text-muted); line-height: 1.35; margin-bottom: 12px;",
                                    "No matches today. Try another English or Bisaya name, or browse Pantry."
                                }
                                button {
                                    style: "background: none; border: none; color: var(--primary); font-size: 13px; font-weight: 700; cursor: pointer; padding: 4px 0;",
                                    onclick: move |_| {
                                        search_query.set(String::new());
                                        active_tab.set("market");
                                        subview.set(MarketSubView::BilingualSearch);
                                    },
                                    "Clear search"
                                }
                            }

                            // State Panel 5: Success · Address saved
                            div { class: "state-panel-card", style: "margin-bottom: 80px;",
                                strong { style: "font-size: 13px; color: #15803d; display: block; margin-bottom: 6px;", "Success · Address saved" }
                                p { style: "font-size: 13px; font-weight: 700; color: var(--primary-dark);", "✓ Home saved successfully" }
                                p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;",
                                    "Your pin and landmark are ready for checkout."
                                }
                            }
                        }
                    }
                },

                "account" => rsx! {
                    // C24: Profile & account
                    div { class: "market-top-header",
                        div { class: "market-header-left",
                            button {
                                class: "market-back-btn",
                                onclick: move |_| {
                                    active_tab.set("market");
                                    subview.set(MarketSubView::Catalog);
                                },
                                "←"
                            }
                            span { class: "market-screen-heading", "Your suki account" }
                        }
                        button {
                            class: "market-help-icon",
                            onclick: move |_| toast_msg.set(Some("Customer Profile & Verified Address Settings".into())),
                            "?"
                        }
                    }

                    div { style: "padding: 12px 16px 80px 16px;",
                        // User Profile Header Card
                        div { class: "rider-profile-box", style: "margin: 0 0 14px 0;",
                            div { style: "display: flex; align-items: center; gap: 12px;",
                                div { class: "rider-avatar-circle", "👤" }
                                div {
                                    h3 { style: "font-size: 17px; font-weight: 800; color: var(--primary-dark); margin: 0;", "{sess.name}" }
                                    p { style: "font-size: 12px; color: var(--text-muted); margin: 2px 0;", "{sess.phone} · Joined Nov 2026" }
                                    span { class: "tag-pill-green", "✓ Mobile number verified" }
                                }
                            }
                        }

                        // Switch Portal Directly Card (1-Click Role Switcher)
                        div { class: "history-card", style: "margin: 0 0 14px 0; border: 1px solid #bbf7d0; background: #f0fdf4;",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 4px;", "Switch Portal Directly" }
                            p { style: "font-size: 12px; color: var(--text-muted); margin-bottom: 10px;",
                                "Jump instantly to Personal Shopper or Stall Vendor workspaces."
                            }
                            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px;",
                                button {
                                    class: "secondary-btn",
                                    style: "padding: 10px; font-size: 12px; font-weight: 700;",
                                    onclick: move |_| on_switch_role.call(UserRole::Delivery),
                                    "🛵 Shopper Portal →"
                                }
                                button {
                                    class: "secondary-btn",
                                    style: "padding: 10px; font-size: 12px; font-weight: 700;",
                                    onclick: move |_| on_switch_role.call(UserRole::Vendor),
                                    "🏪 Vendor Portal →"
                                }
                            }
                        }

                        // My Details Card
                        div { class: "history-card", style: "margin: 0 0 14px 0;",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 8px;", "My details" }
                            div { class: "account-row-item",
                                span { style: "color: var(--text-muted);", "Email address" }
                                div { style: "display: flex; align-items: center; gap: 4px;",
                                    span { "ana.santos@email.com" }
                                    strong { style: "color: var(--primary);", "Edit →" }
                                }
                            }
                            div { class: "account-row-item",
                                span { style: "color: var(--text-muted);", "Language" }
                                strong { "English / Bisaya" }
                            }
                            div { class: "account-row-item", style: "border-bottom: none;",
                                span { style: "color: var(--text-muted);", "Notifications" }
                                strong { style: "color: #15803d;", "Enabled →" }
                            }
                        }

                        // Saved Addresses Card
                        div { class: "history-card", style: "margin: 0 0 14px 0;",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 8px;", "Saved addresses · 2" }
                            div { style: "padding: 8px 0; border-bottom: 1px solid #f1f5f9;",
                                strong { style: "font-size: 13px; color: var(--primary-dark); display: block;", "Home · Default" }
                                p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "18 N. Torres St., Brgy. Obrero, Davao City" }
                            }
                            div { style: "padding: 8px 0;",
                                strong { style: "font-size: 13px; color: var(--primary-dark); display: block;", "Work" }
                                p { style: "font-size: 12px; color: var(--text-muted); margin-top: 2px;", "Palma Gil St., Brgy. 4-A" }
                            }
                            button {
                                style: "width: 100%; margin-top: 8px; background: #ffffff; border: 1px solid var(--border-color); border-radius: 10px; padding: 10px; font-size: 13px; font-weight: 700; color: var(--primary-dark); cursor: pointer;",
                                onclick: move |_| {
                                    active_tab.set("market");
                                    subview.set(MarketSubView::SavedAddresses);
                                },
                                "Manage addresses"
                            }
                        }

                        // Here for you Card
                        div { class: "history-card", style: "margin: 0 0 14px 0;",
                            h4 { style: "font-size: 14px; font-weight: 800; color: var(--primary-dark); margin-bottom: 8px;", "Here for you" }
                            div {
                                class: "account-row-item",
                                onclick: move |_| toast_msg.set(Some("Order & payment help: support@sukidavao.ph".into())),
                                span { "Order & payment help" }
                                span { "→" }
                            }
                            div {
                                class: "account-row-item",
                                onclick: move |_| toast_msg.set(Some("Privacy & Security: 256-bit encrypted".into())),
                                span { "Privacy & security" }
                                span { "→" }
                            }
                            div {
                                class: "account-row-item",
                                onclick: move |_| toast_msg.set(Some("Terms of Service: Certified Bankerohan Palengke".into())),
                                span { "Terms of service" }
                                span { "→" }
                            }
                            div {
                                class: "account-row-item",
                                onclick: move |_| toast_msg.set(Some("About SUKI DAVAO: Davao's first digital market run.".into())),
                                span { "About SUKI DAVAO" }
                                span { "→" }
                            }
                            div {
                                class: "account-row-item", style: "border-bottom: none; color: var(--primary); font-weight: 700;",
                                onclick: move |_| {
                                    active_tab.set("orders");
                                    order_stage.set(OrderScreenStage::RecoveryStates);
                                },
                                span { "View App Recovery States (C25)" }
                                span { "→" }
                            }
                        }

                        button {
                            class: "btn-logout",
                            style: "width: 100%; margin-top: 8px;",
                            onclick: move |_| on_logout.call(()),
                            "Log out"
                        }

                        p { style: "font-size: 11px; color: var(--text-muted); text-align: center; margin-top: 14px;",
                            "Made for Davao. Made for your everyday."
                        }
                    }
                },

                _ => rsx! {}
            }
        }

        // Bottom Navigation Bar
        div { class: "bottom-nav",
            button {
                class: if active_tab() == "home" { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| active_tab.set("home"),
                span { class: "nav-tab-icon", "🏠" }
                span { "Home" }
            }
            button {
                class: if active_tab() == "market" { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| {
                    active_tab.set("market");
                    subview.set(MarketSubView::Catalog);
                },
                span { class: "nav-tab-icon", "🏪" }
                span { "Market" }
            }
            button {
                class: if active_tab() == "orders" { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| active_tab.set("orders"),
                span { class: "nav-tab-icon", "📦" }
                span { "Orders" }
                if total_items_count > 0 {
                    span { class: "cart-badge", "{total_items_count}" }
                }
            }
            button {
                class: if active_tab() == "account" { "nav-tab-btn active" } else { "nav-tab-btn" },
                onclick: move |_| active_tab.set("account"),
                span { class: "nav-tab-icon", "👤" }
                span { "Account" }
            }
        }
    }
}
