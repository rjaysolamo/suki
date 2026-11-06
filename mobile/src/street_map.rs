use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MapMode {
    Tracking,   // Customer tracking & Courier navigation (shows route & live courier)
    Picker,     // Location & address confirmation (shows draggable pin)
}

#[component]
pub fn DavaoStreetMap(
    #[props(default = MapMode::Tracking)]
    mode: MapMode,
    #[props(default = "Jun Mercado".to_string())]
    shopper_name: String,
    #[props(default = "18 N. Torres St., Brgy. Obrero".to_string())]
    destination_label: String,
    #[props(default = "4.00 km".to_string())]
    distance: String,
    #[props(default = "10:40–11:00 AM".to_string())]
    eta: String,
    #[props(default = false)]
    is_shopper_view: bool,
    #[props(default = None)]
    on_open_external_nav: Option<EventHandler<()>>,
) -> Element {
    let mut zoom_level = use_signal(|| 1u32); // 1 = Overview (Bankerohan to Obrero), 2 = Neighborhood zoom (Torres St & Obrero)
    let show_sat = use_signal(|| false);
    let mut toast_local = use_signal(|| Option::<String>::None);

    let view_box = if zoom_level() == 1 {
        "0 0 400 240"
    } else {
        "100 20 250 160"
    };

    rsx! {
        div { class: "street-map-card",
            // Floating Local Toast
            if let Some(msg) = toast_local() {
                div {
                    class: "toast-success",
                    style: "position: absolute; top: 10px; left: 12px; right: 12px; z-index: 50; padding: 8px 12px; font-size: 11px;",
                    div { style: "display: flex; justify-content: space-between; align-items: center;",
                        span { "{msg}" }
                        button {
                            style: "background: none; border: none; font-size: 14px; font-weight: 700; cursor: pointer; color: #14532d;",
                            onclick: move |_| toast_local.set(None),
                            "×"
                        }
                    }
                }
            }

            // Top Status Overlay Bar
            div { class: "street-map-hud",
                div { class: "street-map-hud-left",
                    span { class: "status-pill-gps-live",
                        span { class: "gps-live-dot" }
                        "GPS sharing on · 10:31 AM"
                    }
                }
                div { class: "street-map-hud-right",
                    button {
                        class: if zoom_level() == 2 { "map-btn-icon active" } else { "map-btn-icon" },
                        title: "Toggle Zoom",
                        onclick: move |_| {
                            if zoom_level() == 1 {
                                zoom_level.set(2);
                                toast_local.set(Some("Zoomed in: Torres St & Obrero neighborhood".into()));
                            } else {
                                zoom_level.set(1);
                                toast_local.set(Some("Zoomed out: Bankerohan to Obrero corridor".into()));
                            }
                        },
                        if zoom_level() == 1 { "🔍+" } else { "🔍−" }
                    }
                    button {
                        class: "map-btn-icon",
                        title: "Recenter on courier",
                        onclick: move |_| {
                            zoom_level.set(1);
                            toast_local.set(Some("Recentered on Jun Mercado (Torres St)".into()));
                        },
                        "🎯"
                    }
                }
            }

            // SVG Street Map Viewport
            div { class: "street-map-viewport",
                svg {
                    view_box: "{view_box}",
                    style: "width: 100%; height: 100%; display: block;",

                    // Background Canvas
                    rect {
                        x: "0", y: "0", width: "400", height: "240",
                        fill: if show_sat() { "#1e293b" } else { "#f5f4ee" }
                    }

                    // City Blocks / Parcels (Cream/Beige urban lots)
                    g { id: "city-blocks",
                        // Obrero Residential Lots
                        rect { x: "245", y: "15", width: "65", height: "35", rx: "3", fill: "#ede9de", stroke: "#e3ded3", stroke_width: "1" }
                        rect { x: "320", y: "15", width: "70", height: "35", rx: "3", fill: "#ede9de", stroke: "#e3ded3", stroke_width: "1" }
                        rect { x: "245", y: "65", width: "65", height: "45", rx: "3", fill: "#ede9de", stroke: "#e3ded3", stroke_width: "1" }
                        rect { x: "320", y: "65", width: "70", height: "45", rx: "3", fill: "#ede9de", stroke: "#e3ded3", stroke_width: "1" }
                        rect { x: "245", y: "125", width: "65", height: "45", rx: "3", fill: "#ede9de", stroke: "#e3ded3", stroke_width: "1" }
                        rect { x: "320", y: "125", width: "70", height: "45", rx: "3", fill: "#ede9de", stroke: "#e3ded3", stroke_width: "1" }

                        // Bajada Commercial Blocks
                        rect { x: "125", y: "45", width: "55", height: "40", rx: "3", fill: "#ece7dc", stroke: "#dfd9cb", stroke_width: "1" }
                        rect { x: "125", y: "100", width: "55", height: "40", rx: "3", fill: "#ece7dc", stroke: "#dfd9cb", stroke_width: "1" }
                        rect { x: "60", y: "100", width: "50", height: "40", rx: "3", fill: "#ece7dc", stroke: "#dfd9cb", stroke_width: "1" }
                    }

                    // Green Parks & Institutional Grounds
                    g { id: "green-zones",
                        // Davao City National High School (Green Campus)
                        rect { x: "260", y: "180", width: "110", height: "45", rx: "6", fill: "#dcfce7", stroke: "#bbf7d0", stroke_width: "1" }
                        text {
                            x: "270", y: "205",
                            font_size: "7",
                            font_family: "system-ui, -apple-system, sans-serif",
                            font_weight: "700",
                            fill: "#15803d",
                            "Davao City National High School"
                        }

                        // Rizal Memorial / Colleges
                        rect { x: "10", y: "145", width: "70", height: "40", rx: "4", fill: "#f0fdf4", stroke: "#dcfce7", stroke_width: "1" }
                        text {
                            x: "15", y: "168",
                            font_size: "6.5",
                            font_family: "system-ui, -apple-system, sans-serif",
                            font_weight: "700",
                            fill: "#166534",
                            "Memorial Colleges"
                        }
                    }

                    // Davao River & Generoso Bridge (South-West)
                    g { id: "river-and-bridge",
                        path {
                            d: "M -10 240 Q 40 180 70 145 T 90 70 T 120 0",
                            fill: "none",
                            stroke: "#99c8bc",
                            stroke_width: "28",
                            stroke_linecap: "round"
                        }
                        path {
                            d: "M -10 240 Q 40 180 70 145 T 90 70 T 120 0",
                            fill: "none",
                            stroke: "#bfe0d7",
                            stroke_width: "22",
                            stroke_linecap: "round"
                        }
                        // River text
                        text {
                            x: "32", y: "185",
                            font_size: "7",
                            font_style: "italic",
                            font_weight: "600",
                            fill: "#0f766e",
                            transform: "rotate(-40 32 185)",
                            "Davao River"
                        }
                        // Generoso / Bankerohan Bridge crossing
                        rect { x: "42", y: "168", width: "24", height: "10", rx: "2", fill: "#94a3b8", stroke: "#475569", stroke_width: "1" }
                        line { x1: "42", y1: "173", x2: "66", y2: "173", stroke: "#f8fafc", stroke_width: "2" }
                    }

                    // Street Network: Local Streets (White with subtle casing)
                    g { id: "local-streets",
                        // Horizontal cross streets
                        line { x1: "0", y1: "20", x2: "400", y2: "20", stroke: "#e2e8f0", stroke_width: "7" }
                        line { x1: "0", y1: "20", x2: "400", y2: "20", stroke: "#ffffff", stroke_width: "5" }

                        line { x1: "190", y1: "55", x2: "400", y2: "55", stroke: "#e2e8f0", stroke_width: "7" }
                        line { x1: "190", y1: "55", x2: "400", y2: "55", stroke: "#ffffff", stroke_width: "5" }

                        line { x1: "190", y1: "115", x2: "400", y2: "115", stroke: "#e2e8f0", stroke_width: "7" }
                        line { x1: "190", y1: "115", x2: "400", y2: "115", stroke: "#ffffff", stroke_width: "5" }

                        line { x1: "190", y1: "175", x2: "400", y2: "175", stroke: "#e2e8f0", stroke_width: "7" }
                        line { x1: "190", y1: "175", x2: "400", y2: "175", stroke: "#ffffff", stroke_width: "5" }

                        // Vertical connecting streets
                        line { x1: "315", y1: "0", x2: "315", y2: "240", stroke: "#e2e8f0", stroke_width: "8" }
                        line { x1: "315", y1: "0", x2: "315", y2: "240", stroke: "#ffffff", stroke_width: "6" }

                        line { x1: "390", y1: "0", x2: "390", y2: "240", stroke: "#e2e8f0", stroke_width: "7" }
                        line { x1: "390", y1: "0", x2: "390", y2: "240", stroke: "#ffffff", stroke_width: "5" }
                    }

                    // Arterial Road: J.P. Laurel Avenue (Bajada)
                    // High-contrast golden road casing & bright bed
                    g { id: "arterial-jp-laurel",
                        path {
                            d: "M 80 240 L 140 160 L 195 80 L 235 0",
                            fill: "none",
                            stroke: "#f59e0b",
                            stroke_width: "16",
                            stroke_linecap: "round",
                            stroke_linejoin: "round"
                        }
                        path {
                            d: "M 80 240 L 140 160 L 195 80 L 235 0",
                            fill: "none",
                            stroke: "#fef08a",
                            stroke_width: "12",
                            stroke_linecap: "round",
                            stroke_linejoin: "round"
                        }
                        // Dashed center line
                        path {
                            d: "M 80 240 L 140 160 L 195 80 L 235 0",
                            fill: "none",
                            stroke: "#f59e0b",
                            stroke_width: "1",
                            stroke_dasharray: "4,3"
                        }
                    }

                    // Secondary Arterial: F. Torres Street (Connecting Bajada to Obrero)
                    g { id: "torres-street",
                        path {
                            d: "M 140 160 L 200 135 L 240 100 L 275 55",
                            fill: "none",
                            stroke: "#cbd5e1",
                            stroke_width: "13",
                            stroke_linecap: "round",
                            stroke_linejoin: "round"
                        }
                        path {
                            d: "M 140 160 L 200 135 L 240 100 L 275 55",
                            fill: "none",
                            stroke: "#ffffff",
                            stroke_width: "9",
                            stroke_linecap: "round",
                            stroke_linejoin: "round"
                        }
                    }

                    // Street Text Labels (Angled & styled like real cartographic maps)
                    g { id: "street-labels",
                        // J.P. Laurel Avenue Label
                        text {
                            x: "155", y: "140",
                            font_size: "7.5",
                            font_family: "system-ui, -apple-system, sans-serif",
                            font_weight: "800",
                            fill: "#78350f",
                            letter_spacing: "0.5px",
                            transform: "rotate(-50 155 140)",
                            "J. P. Laurel Avenue"
                        }

                        // Torres Street Label
                        text {
                            x: "215", y: "115",
                            font_size: "7.5",
                            font_family: "system-ui, -apple-system, sans-serif",
                            font_weight: "800",
                            fill: "#334155",
                            transform: "rotate(-32 215 115)",
                            "Torres Street"
                        }

                        // Columbia Street Label
                        text {
                            x: "330", y: "51",
                            font_size: "6.5",
                            font_family: "system-ui, -apple-system, sans-serif",
                            font_weight: "600",
                            fill: "#64748b",
                            "Columbia Street"
                        }

                        // Mabini Extension Label
                        text {
                            x: "330", y: "111",
                            font_size: "6.5",
                            font_family: "system-ui, -apple-system, sans-serif",
                            font_weight: "600",
                            fill: "#64748b",
                            "Mabini Extension"
                        }

                        // Lope Jaena Street Label
                        text {
                            x: "320", y: "160",
                            font_size: "6",
                            font_family: "system-ui, -apple-system, sans-serif",
                            font_weight: "600",
                            fill: "#64748b",
                            transform: "rotate(90 320 160)",
                            "Lope Jaena St"
                        }

                        // Barangay Area Labels
                        text {
                            x: "250", y: "30",
                            font_size: "8.5",
                            font_family: "system-ui, -apple-system, sans-serif",
                            font_weight: "800",
                            fill: "#475569",
                            opacity: "0.8",
                            "Barangay Obrero"
                        }

                        text {
                            x: "15", y: "135",
                            font_size: "7.5",
                            font_family: "system-ui, -apple-system, sans-serif",
                            font_weight: "700",
                            fill: "#64748b",
                            opacity: "0.75",
                            "Barangay 9-A"
                        }
                    }

                    // Active Delivery Route (Emerald glowing polyline)
                    if mode == MapMode::Tracking {
                        g { id: "delivery-route-layer",
                            // Route Shadow / Glow
                            path {
                                d: "M 55 180 L 80 180 L 120 165 L 140 160 L 200 135 L 240 100 L 275 55",
                                fill: "none",
                                stroke: "#86efac",
                                stroke_width: "8",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                opacity: "0.6"
                            }
                            // Precision Route Line
                            path {
                                d: "M 55 180 L 80 180 L 120 165 L 140 160 L 200 135 L 240 100 L 275 55",
                                fill: "none",
                                stroke: "#15803d",
                                stroke_width: "4",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_dasharray: "6,3"
                            }
                        }
                    }

                    // Origin Pin: Bankerohan Public Market (x=55, y=180)
                    g { id: "pin-bankerohan",
                        circle { cx: "55", cy: "180", r: "10", fill: "#b45309" }
                        circle { cx: "55", cy: "180", r: "5", fill: "#ffffff" }
                        text {
                            x: "55", y: "182",
                            font_size: "7",
                            font_weight: "800",
                            text_anchor: "middle",
                            fill: "#b45309",
                            "•"
                        }
                    }

                    // Live Courier Beacon: Jun Mercado (x=200, y=135 on Torres St)
                    if mode == MapMode::Tracking {
                        g { id: "courier-beacon",
                            // Pulsing halo radar
                            circle {
                                cx: "200", cy: "135", r: "15",
                                fill: "#22c55e",
                                opacity: "0.3"
                            }
                            circle {
                                cx: "200", cy: "135", r: "10",
                                fill: "#16a34a",
                                stroke: "#ffffff",
                                stroke_width: "2.5"
                            }
                            // Courier Marker Center Dot
                            circle { cx: "200", cy: "135", r: "4", fill: "#ffffff" }
                        }
                    }

                    // Destination Doorstep Pin: 18 N. Torres St (x=275, y=55)
                    g { id: "pin-doorstep",
                        circle { cx: "275", cy: "55", r: "11", fill: "#14532d" }
                        circle { cx: "275", cy: "55", r: "5", fill: "#ffffff" }
                        circle { cx: "275", cy: "55", r: "2.5", fill: "#ef4444" }
                    }
                }

                // Floating HTML Badges on Map Canvas
                // 1. Bankerohan Market Pill
                div {
                    class: "map-pin-market",
                    style: "bottom: 22px; left: 18px;",
                    "🏪 Bankerohan Market"
                }

                // 2. Courier Scooter Float Badge
                if mode == MapMode::Tracking {
                    div {
                        class: "map-courier-pill",
                        style: "top: 115px; left: 130px;",
                        span { "🛵" }
                        span { "{shopper_name} · En route" }
                    }
                }

                // 3. Your Doorstep Pill
                div {
                    class: "map-pin-pill",
                    style: "top: 26px; right: 28px;",
                    span { "📍" }
                    span { if is_shopper_view { "Ana's doorstep" } else { "Your doorstep" } }
                }
            }

            // Map Bottom Info Bar
            div { class: "street-map-footer",
                div { class: "street-map-metrics",
                    span { style: "font-size: 11px; font-weight: 700; color: var(--primary-dark);",
                        "📍 {destination_label}"
                    }
                    div { style: "display: flex; gap: 8px; margin-top: 2px; font-size: 11px; color: var(--text-muted);",
                        span { "{distance} driving distance" }
                        span { "·" }
                        span { "Arrival {eta}" }
                    }
                }

                button {
                    class: "btn-open-nav",
                    onclick: move |_| {
                        if let Some(handler) = on_open_external_nav {
                            handler.call(());
                        } else {
                            toast_local.set(Some("Opening navigation to 18 N. Torres St., Obrero...".into()));
                        }
                    },
                    "Open device navigation ↗"
                }
            }
        }
    }
}
