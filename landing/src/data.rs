use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MarketHub {
    pub id: &'static str,
    pub name: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub specialties: &'static [&'static str],
    pub operating_hours: &'static str,
    pub active_vendors: u32,
    pub location: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceItem {
    pub name: &'static str,
    pub category: &'static str,
    pub unit: &'static str,
    pub palengke_price: f64,
    pub supermarket_price: f64,
    pub savings_percent: u32,
    pub image_path: &'static str,
    pub origin_market: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct DavaoDistrict {
    pub name: &'static str,
    pub distance_km: f64,
    pub est_minutes: u32,
    pub delivery_fee: f64,
}

pub fn get_market_hubs() -> Vec<MarketHub> {
    vec![
        MarketHub {
            id: "bankerohan",
            name: "Bankerohan Public Market",
            tag: "Wholesale & Upland Produce Hub",
            description: "Davao City's historic 24/7 central trading post along the Davao River. The primary entry point for Marilog and Bukidnon highland vegetables, native poultry, and farm-fresh eggs.",
            specialties: &["Marilog Highland Cabbage", "Native Davao Poultry", "Fresh Tomatoes & Spices", "Root Crops & Grains"],
            operating_hours: "24 Hours (Wholesale Bagsakan: 11:00 PM – 6:00 AM)",
            active_vendors: 148,
            location: "Bankerohan, Poblacion District, Davao City",
        },
        MarketHub {
            id: "agdao",
            name: "Agdao Public Market",
            tag: "Seafood Capital of Davao Gulf",
            description: "Famous throughout Mindanao for having the freshest morning fish catch directly from the Davao Gulf fishing boats. Premium yellowfin tuna, bangus, blue marlin, and shellfish.",
            specialties: &["Yellowfin Tuna Loins", "Sarangani & Davao Bangus", "Fresh Gulf Squid", "Live Mud Crabs & Shellfish"],
            operating_hours: "4:00 AM – 7:30 PM (Daily Pier Deliveries at Dawn)",
            active_vendors: 86,
            location: "Agdao District, Davao City",
        },
        MarketHub {
            id: "toril",
            name: "Toril Public Market",
            tag: "Southern Fruit & Orchard Gateway",
            description: "Nestled at the foothills of Mt. Apo, Toril is Davao's agricultural treasure chest. Famous for native Puyat durian, golden pomelo, marang, and fresh highland honey.",
            specialties: &["Puyat & Arancillo Durian", "Golden Davao Pomelo", "Sweet Davao Mangoes", "Highland Greens & Coffee"],
            operating_hours: "5:00 AM – 8:00 PM (Bagsakan on Tuesdays & Fridays)",
            active_vendors: 64,
            location: "Toril District, Davao City",
        },
    ]
}

pub fn get_price_index() -> Vec<PriceItem> {
    vec![
        PriceItem {
            name: "Yellowfin Tuna Steak",
            category: "Seafood",
            unit: "per kg",
            palengke_price: 380.00,
            supermarket_price: 590.00,
            savings_percent: 36,
            image_path: "/assets/images/fish.jpg",
            origin_market: "Agdao Seafood Dock",
        },
        PriceItem {
            name: "Davao Puyat Durian",
            category: "Fruits",
            unit: "per kg",
            palengke_price: 160.00,
            supermarket_price: 280.00,
            savings_percent: 43,
            image_path: "/assets/images/hero.jpg",
            origin_market: "Toril Fruit Terminal",
        },
        PriceItem {
            name: "Sweet Davao Mangoes",
            category: "Fruits",
            unit: "per kg",
            palengke_price: 140.00,
            supermarket_price: 220.00,
            savings_percent: 36,
            image_path: "/assets/images/mangoes.jpg",
            origin_market: "Toril Fruit Terminal",
        },
        PriceItem {
            name: "Highland Fresh Cabbage",
            category: "Vegetables",
            unit: "per kg",
            palengke_price: 45.00,
            supermarket_price: 85.00,
            savings_percent: 47,
            image_path: "/assets/images/cabbage.jpg",
            origin_market: "Bankerohan Bagsakan",
        },
        PriceItem {
            name: "Native Davao Tomatoes",
            category: "Vegetables",
            unit: "per kg",
            palengke_price: 60.00,
            supermarket_price: 105.00,
            savings_percent: 43,
            image_path: "/assets/images/tomatoes.jpg",
            origin_market: "Bankerohan Bagsakan",
        },
        PriceItem {
            name: "Fresh Whole Dressed Chicken",
            category: "Meat & Poultry",
            unit: "per kg",
            palengke_price: 210.00,
            supermarket_price: 315.00,
            savings_percent: 33,
            image_path: "/assets/images/chicken.jpg",
            origin_market: "Bankerohan Meat Section",
        },
        PriceItem {
            name: "Organic Highland Carrots",
            category: "Vegetables",
            unit: "per kg",
            palengke_price: 65.00,
            supermarket_price: 110.00,
            savings_percent: 41,
            image_path: "/assets/images/carrots.jpg",
            origin_market: "Bankerohan Bagsakan",
        },
        PriceItem {
            name: "Davao Native Eggplant (Talong)",
            category: "Vegetables",
            unit: "per kg",
            palengke_price: 50.00,
            supermarket_price: 90.00,
            savings_percent: 44,
            image_path: "/assets/images/eggplant.jpg",
            origin_market: "Bankerohan Bagsakan",
        },
        PriceItem {
            name: "Lakatan Prime Bananas",
            category: "Fruits",
            unit: "per kg",
            palengke_price: 65.00,
            supermarket_price: 105.00,
            savings_percent: 38,
            image_path: "/assets/images/bananas.jpg",
            origin_market: "Toril Fruit Terminal",
        },
    ]
}

pub fn get_davao_districts() -> Vec<DavaoDistrict> {
    vec![
        DavaoDistrict { name: "Poblacion (Downtown)", distance_km: 2.4, est_minutes: 18, delivery_fee: 45.00 },
        DavaoDistrict { name: "Bajada / J.P. Laurel", distance_km: 4.8, est_minutes: 22, delivery_fee: 55.00 },
        DavaoDistrict { name: "Lanang / Sasa", distance_km: 8.2, est_minutes: 28, delivery_fee: 75.00 },
        DavaoDistrict { name: "Matina (Crossing & Aplaya)", distance_km: 5.5, est_minutes: 25, delivery_fee: 60.00 },
        DavaoDistrict { name: "Buhangin / Indangan", distance_km: 9.0, est_minutes: 32, delivery_fee: 80.00 },
        DavaoDistrict { name: "Maa / Diversion Road", distance_km: 6.2, est_minutes: 26, delivery_fee: 65.00 },
        DavaoDistrict { name: "Toril Poblacion", distance_km: 14.5, est_minutes: 38, delivery_fee: 110.00 },
        DavaoDistrict { name: "Calinan / Mintal", distance_km: 19.8, est_minutes: 45, delivery_fee: 145.00 },
    ]
}
