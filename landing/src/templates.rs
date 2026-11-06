use crate::data::{get_davao_districts, get_market_hubs, get_price_index, DavaoDistrict, MarketHub, PriceItem};

pub fn render_landing_page() -> String {
    let hubs = get_market_hubs();
    let price_items = get_price_index();
    let districts = get_davao_districts();
    let css = crate::style::render_css();

    let mut html = String::with_capacity(65536);
    html.push_str("<!doctype html>\n<html lang=\"en\" data-type=\"software\" data-radius=\"soft\" data-ground=\"sand\" data-buttons=\"solid\" data-motion>\n");
    html.push_str(&render_head(&css));
    html.push_str("<body>\n");
    html.push_str(render_announcement());
    html.push_str(render_header());
    html.push_str(render_hero());
    html.push_str(&render_hubs_section(&hubs));
    html.push_str(&render_price_index_section(&price_items));
    html.push_str(&render_simulator_section(&districts));
    html.push_str(render_tech_section());
    html.push_str(render_cold_chain_section());
    html.push_str(render_app_section());
    html.push_str(render_gcash_section());
    html.push_str(render_footer());
    html.push_str("</body>\n</html>\n");

    html
}

fn render_head(css: &str) -> String {
    format!(
        r##"<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
  <title>Suki Davao — Fresh Palengke & Wholesale Catch Delivered in Davao City</title>
  <meta name="description" content="Fresh morning catch from Agdao, dawn bagsakan from Bankerohan, and upland fruits from Toril delivered across Davao City in under 45 minutes with zero supermarket markup.">
  <meta name="site-kind" content="software">
  <meta name="page-type" content="software">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Manrope:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
  <style>{}</style>
</head>
"##,
        css
    )
}

fn render_announcement() -> &'static str {
    r##"  <!-- Top Announcement Bar -->
  <aside class="announcement-bar">
    <div class="announcement-content">
      <span class="status-dot"></span>
      <span><strong>Live Morning Bagsakan:</strong> Bankerohan wholesale open &bull; Agdao fresh catch arriving &bull; 42 Davao motorcycle riders active</span>
      <a href="#calculator" class="announcement-link">Calculate road quote &rarr;</a>
    </div>
  </aside>
"##
}

fn render_header() -> &'static str {
    r##"  <!-- Navigation -->
  <header class="main-header">
    <div class="header-inner">
      <a href="/" class="brand-logo">
        <div class="logo-leaf-icon">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.5 2 2 6.5 2 12c0 3.5 1.8 6.6 4.6 8.4C6.2 19 6 17.5 6 16c0-5 3.5-9.5 8.5-10.8C15 5.1 15.6 5 16.3 5c.7 0 1.4.1 2 .2C17.3 3.3 14.8 2 12 2z"/>
            <path d="M17 7c-4.4 0-8 3.6-8 8 0 2.2.9 4.2 2.3 5.7.5.5 1.1.9 1.7 1.3C13.6 22 14.3 22 15 22c3.9 0 7-3.1 7-7 0-4.4-3.6-8-8-8zm0 13c-2.8 0-5-2.2-5-5s2.2-5 5-5 5 2.2 5 5-2.2 5-5 5z"/>
          </svg>
        </div>
        <div class="logo-text-group">
          <span class="brand-name">SUKI</span>
          <span class="brand-sub">DAVAO CITY</span>
        </div>
      </a>

      <nav class="desktop-nav" aria-label="Main Navigation">
        <a href="#hubs" class="nav-item">Davao Hubs</a>
        <a href="#price-index" class="nav-item">Price Index</a>
        <a href="#calculator" class="nav-item">Quote Simulator</a>
        <a href="#cold-chain" class="nav-item">Cold Chain</a>
        <a href="#tech" class="nav-item">Pure Rust Stack</a>
        <a href="#app" class="nav-item">Mobile App</a>
      </nav>

      <div class="header-actions">
        <a href="#calculator" class="btn-ghost">Test Live Quote</a>
        <a href="#app" class="btn-primary">Get Android App</a>
      </div>
    </div>
  </header>
"##
}

fn render_hero() -> &'static str {
    r##"  <!-- HERO SECTION -->
  <section class="hero-section" id="hero">
    <div class="hero-container">
      <div class="hero-copy">
        <div class="hero-pill">
          <span class="pill-badge">DAVAO PALENGKE PLATFORM</span>
          <span class="pill-text">Direct from Bankerohan, Agdao &amp; Toril</span>
        </div>
        <h1 class="hero-title">
          Skip the 5:00 AM palengke traffic.<br>
          <span class="text-accent">Wholesale prices</span> at your doorstep.
        </h1>
        <p class="hero-lead">
          Direct dawn harvest from Davao's 3 historic public markets. Zero middleman markup, exact centavo road-distance quotes, and instant GCash payment verified in sub-milliseconds.
        </p>

        <div class="hero-cta-group">
          <a href="#calculator" class="btn-primary-lg">
            <span>Build Your Fresh Basket</span>
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"></line><polyline points="12 5 19 12 12 19"></polyline></svg>
          </a>
          <a href="#hubs" class="btn-secondary-lg">
            <span>Explore Davao Markets</span>
          </a>
        </div>

        <div class="hero-metrics">
          <div class="metric-item">
            <span class="metric-number">&#8369;0.00</span>
            <span class="metric-label">Hidden supermarket markup</span>
          </div>
          <div class="metric-divider"></div>
          <div class="metric-item">
            <span class="metric-number">38 min</span>
            <span class="metric-label">Avg. Davao delivery time</span>
          </div>
          <div class="metric-divider"></div>
          <div class="metric-item">
            <span class="metric-number">100% Rust</span>
            <span class="metric-label">Zero JS backend latency</span>
          </div>
        </div>
      </div>

      <div class="hero-media">
        <div class="hero-card-frame">
          <img src="/assets/images/hero.jpg" alt="Davao fresh durian and fruit harvest" class="hero-main-photo" />
          <div class="hero-overlay-tag">
            <span class="pulse-dot"></span>
            <span>Live Dispatch &bull; Davao City</span>
          </div>

          <!-- Floating live dispatch card -->
          <div class="dispatch-card">
            <div class="dispatch-header">
              <span class="dispatch-rider-icon">&#127949;</span>
              <div>
                <span class="dispatch-title">Rider Junjun &bull; Suki Express</span>
                <span class="dispatch-sub">En route to Matina Crossing</span>
              </div>
              <span class="dispatch-time">12 mins ago</span>
            </div>
            <div class="dispatch-items">
              <div class="dispatch-item-chip">2.5 kg Yellowfin Tuna (Agdao)</div>
              <div class="dispatch-item-chip">3 kg Puyat Durian (Toril)</div>
            </div>
            <div class="dispatch-footer">
              <span class="dispatch-paid">&check; Verified GCash Ref: #90412847</span>
              <span class="dispatch-amount">&#8369;890.00</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
"##
}

fn render_hubs_section(hubs: &[MarketHub]) -> String {
    let mut hubs_html = String::new();
    for (i, hub) in hubs.iter().enumerate() {
        let active_cls = if i == 0 { "hub-card active" } else { "hub-card" };
        let mut specialties_html = String::new();
        for spec in hub.specialties {
            specialties_html.push_str(&format!(
                r##"<span class="hub-pill">{}</span>"##,
                spec
            ));
        }

        hubs_html.push_str(&format!(
            r##"
            <article class="{active_cls}" data-hub-id="{id}">
              <div class="hub-header">
                <div>
                  <span class="hub-tag">{tag}</span>
                  <h3 class="hub-title">{name}</h3>
                </div>
                <div class="hub-vendor-badge">
                  <span class="hub-indicator"></span>
                  <span>{vendors} Active Stalls</span>
                </div>
              </div>
              <p class="hub-desc">{desc}</p>
              <div class="hub-pills">
                {specialties_html}
              </div>
              <div class="hub-footer">
                <div class="hub-meta">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                  <span>{hours}</span>
                </div>
                <div class="hub-meta">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/></svg>
                  <span>{loc}</span>
                </div>
              </div>
            </article>
            "##,
            active_cls = active_cls,
            id = hub.id,
            tag = hub.tag,
            name = hub.name,
            vendors = hub.active_vendors,
            desc = hub.description,
            specialties_html = specialties_html,
            hours = hub.operating_hours,
            loc = hub.location
        ));
    }

    format!(
        r##"  <!-- DAVAO THREE MARKETS SECTION -->
  <section class="section hubs-section" id="hubs">
    <div class="section-container">
      <div class="section-header">
        <div class="section-pre">Direct Sourcing Network</div>
        <h2 class="section-title">Davao's Three Legendary Food Hubs</h2>
        <p class="section-lead">We do not store inventory in warehouses. When you place an order, our accredited Davao riders pick your produce directly from verified stallholders at dawn.</p>
      </div>

      <div class="hubs-grid">
        {}
      </div>
    </div>
  </section>
"##,
        hubs_html
    )
}

fn render_price_index_section(prices: &[PriceItem]) -> String {
    let mut price_items_html = String::new();
    for item in prices {
        let unit_cleaned = item.unit.replace("per ", "");
        price_items_html.push_str(&format!(
            r##"
            <tr class="price-row" data-category="{cat}">
              <td class="item-name-cell">
                <div class="item-thumb-wrapper">
                  <img src="{img}" alt="{name}" class="item-thumb" loading="lazy" />
                  <div>
                    <span class="item-title">{name}</span>
                    <span class="item-origin">{origin}</span>
                  </div>
                </div>
              </td>
              <td><span class="category-badge">{cat}</span></td>
              <td class="price-palengke">&#8369;{palengke:.2} <span class="unit-text">/{unit}</span></td>
              <td class="price-supermarket">&#8369;{supermarket:.2}</td>
              <td class="savings-cell">
                <span class="savings-tag">Save {savings}%</span>
              </td>
              <td>
                <a href="#calculator" class="btn-add-quick">
                  + Select
                </a>
              </td>
            </tr>
            "##,
            cat = item.category,
            img = item.image_path,
            name = item.name,
            origin = item.origin_market,
            palengke = item.palengke_price,
            unit = unit_cleaned,
            supermarket = item.supermarket_price,
            savings = item.savings_percent
        ));
    }

    format!(
        r##"  <!-- PALENGKE PRICE INDEX SECTION -->
  <section class="section price-index-section" id="price-index">
    <div class="section-container">
      <div class="section-header">
        <div class="section-pre">Transparent Daily Price Index</div>
        <h2 class="section-title">Palengke Bagsakan vs. Supermarket Markup</h2>
        <p class="section-lead">Audited daily at 5:30 AM across Bankerohan, Agdao, and Toril. Compare what you actually pay versus major Davao retail supermarkets.</p>
      </div>

      <div class="price-table-wrapper">
        <table class="price-table">
          <thead>
            <tr>
              <th>Fresh Market Item</th>
              <th>Category</th>
              <th>Suki Palengke Price</th>
              <th>Supermarket Retail</th>
              <th>Your Savings</th>
              <th>Action</th>
            </tr>
          </thead>
          <tbody>
            {}
          </tbody>
        </table>
      </div>

      <div class="price-audit-note">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>
        <span>Data verified live from Bankerohan Wholesale Fruit Terminal and Agdao Seafood Landing on <strong>November 6, 2026</strong>. Prices include municipal market regulatory compliance.</span>
      </div>
    </div>
  </section>
"##,
        price_items_html
    )
}

fn render_simulator_section(districts: &[DavaoDistrict]) -> String {
    let mut district_options_html = String::new();
    for dist in districts {
        district_options_html.push_str(&format!(
            r##"<option value="{dist_km}" data-minutes="{mins}" data-fee="{fee}">{name} (~{mins} mins)</option>"##,
            dist_km = dist.distance_km,
            mins = dist.est_minutes,
            fee = dist.delivery_fee,
            name = dist.name
        ));
    }

    format!(
        r##"  <!-- INTERACTIVE BASKET & QUOTE SIMULATOR -->
  <section class="section simulator-section" id="calculator">
    <div class="section-container">
      <div class="simulator-wrapper">
        <div class="simulator-info">
          <div class="section-pre">Pure Rust Calculation Engine</div>
          <h2 class="section-title">Live Fresh Basket &amp; Distance Quote Simulator</h2>
          <p class="section-lead">Experience the real pricing algorithm running in Suki's Rust backend. Exact centavo calculations, verified road-distance delivery tiers, and zero hidden surcharges.</p>
          
          <div class="tech-badge-row">
            <span class="rust-pill">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="16 18 22 12 16 6"></polyline><polyline points="8 6 2 12 8 18"></polyline></svg>
              Axum &amp; Tokio Backend
            </span>
            <span class="rust-pill">Sub-millisecond Latency</span>
            <span class="rust-pill">Exact Centavo Math</span>
          </div>

          <div class="sim-preset-card">
            <h4>Quick Davao Produce Presets:</h4>
            <div class="preset-buttons">
              <span class="btn-preset">&#128031; Agdao Tuna &amp; Seafood Pack</span>
              <span class="btn-preset">&#127816; Toril Durian &amp; Mango Feast</span>
              <span class="btn-preset">&#129388; Bankerohan Highland Greens</span>
            </div>
          </div>
        </div>

        <div class="simulator-widget">
          <div class="widget-header">
            <h3>Instant Order Quote Engine</h3>
            <span class="calc-badge">Real-time Centavo Calc</span>
          </div>

          <div class="widget-body">
            <div class="sim-group">
              <label for="district-select" class="sim-label">Delivery Destination (Davao City):</label>
              <select id="district-select" class="sim-select">
                {districts}
              </select>
            </div>

            <div class="basket-items-list" id="basket-items-container">
              <div class="basket-item-row">
                <div>
                  <strong style="color: var(--p-ink);">Yellowfin Tuna Steak</strong>
                  <div style="font-size: 0.75rem; color: var(--p-muted);">&#8369;380.00/kg &times; 1.5 kg = <span style="font-family: var(--font-mono); font-weight: 700; color: var(--p-accent);">&#8369;570.00</span></div>
                </div>
                <span style="font-family: var(--font-mono); font-size: 0.8125rem; font-weight: 700; color: var(--p-accent);">Agdao Catch</span>
              </div>
              <div class="basket-item-row">
                <div>
                  <strong style="color: var(--p-ink);">Davao Puyat Durian</strong>
                  <div style="font-size: 0.75rem; color: var(--p-muted);">&#8369;160.00/kg &times; 2.0 kg = <span style="font-family: var(--font-mono); font-weight: 700; color: var(--p-accent);">&#8369;320.00</span></div>
                </div>
                <span style="font-family: var(--font-mono); font-size: 0.8125rem; font-weight: 700; color: var(--p-accent);">Toril Upland</span>
              </div>
              <div class="basket-item-row">
                <div>
                  <strong style="color: var(--p-ink);">Highland Fresh Cabbage</strong>
                  <div style="font-size: 0.75rem; color: var(--p-muted);">&#8369;45.00/kg &times; 1.0 kg = <span style="font-family: var(--font-mono); font-weight: 700; color: var(--p-accent);">&#8369;45.00</span></div>
                </div>
                <span style="font-family: var(--font-mono); font-size: 0.8125rem; font-weight: 700; color: var(--p-accent);">Marilog Green</span>
              </div>
            </div>

            <div class="quote-summary">
              <div class="summary-line">
                <span>Items Subtotal:</span>
                <span class="mono-num">&#8369;935.00</span>
              </div>
              <div class="summary-line">
                <span>Distance Delivery Fee (Obrero Base):</span>
                <span class="mono-num">&#8369;45.00</span>
              </div>
              <div class="summary-line highlight">
                <span>Supermarket Comparison Price:</span>
                <span class="mono-num strikethrough">&#8369;1,520.00</span>
              </div>
              <div class="summary-line total">
                <span>Final Suki Total:</span>
                <span class="mono-num text-bold">&#8369;980.00</span>
              </div>
              <div class="summary-line savings-green">
                <span>Your Immediate Savings:</span>
                <span class="mono-num">&#8369;540.00 (Save 36%)</span>
              </div>
            </div>

            <div class="widget-actions">
              <a href="#app" class="btn-simulate-order" style="display:flex;align-items:center;justify-content:center;text-decoration:none;">
                <span>Open in Suki Mobile App</span>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"></polyline></svg>
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
"##,
        districts = district_options_html
    )
}

fn render_tech_section() -> &'static str {
    r##"  <!-- PURE RUST ARCHITECTURE SECTION -->
  <section class="section tech-section" id="tech">
    <div class="section-container">
      <div class="section-header">
        <div class="section-pre">Engineering Standard</div>
        <h2 class="section-title">Built 100% in Rust — Why It Matters</h2>
        <p class="section-lead">We migrated away from legacy JavaScript stacks. Suki Davao is engineered in pure Rust from the web presentation to backend databases and native mobile apps.</p>
      </div>

      <div class="tech-grid">
        <div class="tech-card">
          <div class="tech-icon-box">&#9889;</div>
          <h3>Sub-millisecond Distance Quotes</h3>
          <p>Davao City spans over 2,440 square kilometers. Our road-distance quote engine evaluates routes, traffic weights, and exact centavo price caps in microseconds with zero GC pauses.</p>
          <div class="tech-card-footer">
            <code>backend/src/pricing.rs</code>
          </div>
        </div>

        <div class="tech-card">
          <div class="tech-icon-box">&#128737;</div>
          <h3>Verified GCash Transaction Safety</h3>
          <p>Every reference is cryptographically checked and immutable. Transactions utilize SQLx transactions on MariaDB with zero chance of double-spending or corrupted cart state.</p>
          <div class="tech-card-footer">
            <code>backend/src/payments.rs</code>
          </div>
        </div>

        <div class="tech-card">
          <div class="tech-icon-box">&#128241;</div>
          <h3>Native Dioxus Mobile Architecture</h3>
          <p>A single declarative codebase powers our Android APK with instant startup times, high framerates, and direct JNI system bridging without bloated WebView wrappers.</p>
          <div class="tech-card-footer">
            <code>mobile/src/main.rs</code>
          </div>
        </div>

        <div class="tech-card">
          <div class="tech-icon-box">&#127760;</div>
          <h3>Zero-Bloat Landing &amp; Storefront</h3>
          <p>This web landing page runs in under 30MB of RAM, responding to thousands of concurrent requests while providing lightning-fast SEO and instant first contentful paint.</p>
          <div class="tech-card-footer">
            <code>landing/src/main.rs</code>
          </div>
        </div>
      </div>
    </div>
  </section>
"##
}

fn render_cold_chain_section() -> &'static str {
    r##"  <!-- COLD CHAIN & LOGISTICS SECTION -->
  <section class="section cold-chain-section" id="cold-chain">
    <div class="section-container">
      <div class="section-header">
        <div class="section-pre">Direct Delivery Standard</div>
        <h2 class="section-title">The Davao Cold-Chain Rider Protocol</h2>
        <p class="section-lead">Perishable seafood and delicate fruits cannot survive Davao's midday tropical heat without specialized care. Here is how Suki protects every order.</p>
      </div>

      <div class="pillars-grid">
        <div class="pillar-card">
          <div class="pillar-num">01</div>
          <h4>Ice-Insulated Cargo Crates</h4>
          <p>Agdao seafood is packed in food-grade thermal containers with crushed ice within 5 minutes of pier unloading, maintaining a strict 2&deg;C–4&deg;C temperature profile.</p>
        </div>
        <div class="pillar-card">
          <div class="pillar-num">02</div>
          <h4>Dedicated Point-to-Point Runs</h4>
          <p>Couriers do not batch 10 drop-offs at once. Each rider receives a dedicated market-to-door corridor, ensuring delivery in under 45 minutes across all districts.</p>
        </div>
        <div class="pillar-card">
          <div class="pillar-num">03</div>
          <h4>Direct Stall Vendor Payout</h4>
          <p>100% of the listed palengke item price goes straight to the vendor's wallet upon dispatch. Suki charges only the verified delivery courier road fee.</p>
        </div>
        <div class="pillar-card">
          <div class="pillar-num">04</div>
          <h4>Live Authenticated WebSockets</h4>
          <p>Both customer and dispatcher track courier coordinates in real-time with automatic reconnection recovery built directly into the Axum server.</p>
        </div>
      </div>
    </div>
  </section>
"##
}

fn render_app_section() -> &'static str {
    r##"  <!-- MOBILE APP PREVIEW SECTION -->
  <section class="section app-section" id="app">
    <div class="section-container">
      <div class="app-split">
        <div class="app-info">
          <div class="section-pre">Native Mobile Application</div>
          <h2 class="section-title">Get Suki on Your Android Phone</h2>
          <p class="section-lead">Experience the full pure-Rust mobile app built with Dioxus. Seamless login, real-time market stalls, category browsing, and order tracking right in your pocket.</p>
          
          <div class="screen-feature-list">
            <div class="screen-feature-item">
              <span class="sf-icon">&check;</span>
              <div>
                <strong>Native Rust Onboarding:</strong> Instant startup with zero JavaScript engine warm-up.
              </div>
            </div>
            <div class="screen-feature-item">
              <span class="sf-icon">&check;</span>
              <div>
                <strong>Secure Customer Accounts:</strong> Argon2id encrypted sessions and secure credential management.
              </div>
            </div>
            <div class="screen-feature-item">
              <span class="sf-icon">&check;</span>
              <div>
                <strong>Live Bankerohan Marketplace:</strong> Tabbed browsing across Seafood, Vegetables, Fruits, Meats, and Davao Specialties.
              </div>
            </div>
          </div>

          <div class="app-download-actions">
            <a href="/mobile/suki-debug.apk" class="btn-primary-lg" download>
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor"><path d="M17.523 15.3414c-.5511 0-.9993-.4486-.9993-.9997s.4482-.9993.9993-.9993c.551 0 .9993.4482.9993.9993.0001.5511-.4483.9997-.9993.9997m-11.046 0c-.5511 0-.9993-.4486-.9993-.9997s.4482-.9993.9993-.9993c.5511 0 .9993.4482.9993.9993 0 .5511-.4482.9997-.9993.9997m11.4045-6.02l1.9973-3.4592a.416.416 0 00-.1521-.5676.416.416 0 00-.5676.1521l-2.0223 3.503C15.5902 8.4116 13.8533 8.125 12 8.125s-3.5902.2866-5.1368.8247L4.8409 5.4467a.4161.4161 0 00-.5677-.1521.4157.4157 0 00-.1521.5676l1.9973 3.4592C2.6889 11.1867.3432 14.6589 0 18.761h24c-.3432-4.1021-2.6889-7.5743-6.1185-9.4396"/></svg>
              <span>Download Android APK (v0.1.0)</span>
            </a>
            <a href="https://github.com" target="_blank" rel="noopener" class="btn-secondary-lg">
              <span>View Rust Workspace</span>
            </a>
          </div>
        </div>

        <div class="app-mockup-wrapper">
          <div class="phone-mockup">
            <div class="phone-notch"></div>
            <div class="phone-screen-content">
              <div class="mock-top-bar">
                <span>Davao City &bull; 5G Live</span>
                <span>Bankerohan Central</span>
              </div>
              <div class="mock-app-header">
                <span class="mock-title">Suki Davao</span>
                <span class="mock-badge">Bankerohan Live</span>
              </div>
              <div class="mock-category-row">
                <span class="mock-cat-chip active">All</span>
                <span class="mock-cat-chip">Seafood</span>
                <span class="mock-cat-chip">Fruits</span>
                <span class="mock-cat-chip">Vegetables</span>
              </div>
              <div class="mock-product-cards">
                <div class="mock-card">
                  <img src="/assets/images/fish.jpg" alt="Yellowfin Tuna" />
                  <div class="mock-card-body">
                    <strong>Yellowfin Tuna</strong>
                    <span>&#8369;380 / kg</span>
                  </div>
                </div>
                <div class="mock-card">
                  <img src="/assets/images/hero.jpg" alt="Durian" />
                  <div class="mock-card-body">
                    <strong>Puyat Durian</strong>
                    <span>&#8369;160 / kg</span>
                  </div>
                </div>
              </div>
              <div class="mock-order-bar">
                <span>&#128722; 2 items (&#8369;540.00)</span>
                <button type="button" class="mock-btn-order">Order Now</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
"##
}

fn render_gcash_section() -> &'static str {
    r##"  <!-- GCASH VERIFICATION INFO -->
  <section class="section-wrapper surface-muted" style="padding: 48px 24px; border-top: 1px solid var(--p-border);">
    <div class="section-container" style="max-width: 800px; text-align: center;">
      <span class="section-kicker">INSTANT GCASH DISPATCH ENGINE</span>
      <h3 style="font-size: 1.5rem; margin-bottom: 12px; color: var(--p-ink);">Centavo-Exact GCash Reference Matching</h3>
      <p style="font-size: 0.9375rem; color: var(--p-muted); line-height: 1.6; margin-bottom: 20px;">
        In Suki's pure Rust backend, every GCash transaction reference number (e.g. <code>90412847123</code>) is verified against automated Davao dispatch in sub-1 millisecond via SHA-256 state tracking.
      </p>
      <div style="display: inline-flex; align-items: center; gap: 8px; background: #dcfce7; color: #14532d; padding: 8px 16px; border-radius: 9999px; font-weight: 700; font-size: 0.8125rem;">
        <span>&#10003; Zero Third-Party Payment Gateway Delay</span>
        <span>&bull;</span>
        <span>Rust Microsecond Dispatch</span>
      </div>
    </div>
  </section>
"##
}

fn render_footer() -> &'static str {
    r##"  <!-- FOOTER -->
  <footer class="main-footer">
    <div class="footer-container">
      <div class="footer-top">
        <div class="footer-col brand-col">
          <div class="footer-logo">
            <span class="logo-text">SUKI DAVAO</span>
          </div>
          <p class="footer-text">Davao City's premier fresh palengke logistics platform. Direct from Bankerohan, Agdao, and Toril to homes and restaurants across Davao.</p>
          <div class="rust-powered-badge">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2l9 4.5v11L12 22l-9-4.5v-11z"/></svg>
            <span>Powered 100% by Pure Rust &bull; Zero JavaScript Backend</span>
          </div>
        </div>

        <div class="footer-col">
          <h4 class="footer-heading">Davao Market Hubs</h4>
          <ul class="footer-links">
            <li><a href="#hubs">Bankerohan Central Wholesale (24/7)</a></li>
            <li><a href="#hubs">Agdao Fish Port &amp; Marine Landing</a></li>
            <li><a href="#hubs">Toril Fruit &amp; Orchard Gateway</a></li>
            <li><a href="#price-index">Daily Palengke Price Index</a></li>
          </ul>
        </div>

        <div class="footer-col">
          <h4 class="footer-heading">Davao Coverage</h4>
          <ul class="footer-links">
            <li><a href="#calculator">District 1: Poblacion, Talomo</a></li>
            <li><a href="#calculator">District 2: Buhangin, Agdao, Bunawan</a></li>
            <li><a href="#calculator">District 3: Toril, Calinan, Tugbok</a></li>
            <li><a href="#cold-chain">Cold Chain Temperature Standard</a></li>
          </ul>
        </div>

        <div class="footer-col">
          <h4 class="footer-heading">Rust Architecture</h4>
          <ul class="footer-links">
            <li><a href="#tech">suki-api (Axum + SQLx)</a></li>
            <li><a href="#tech">suki-mobile (Dioxus Native)</a></li>
            <li><a href="#tech">suki-landing (Pure Rust Web)</a></li>
            <li><a href="#tech">suki-foundation-api (Argon2id)</a></li>
          </ul>
        </div>
      </div>

      <div class="footer-bottom">
        <div class="footer-legal">
          <span>&copy; 2026 Suki Davao Platform. Davao City, Philippines.</span>
          <span>Photographs sourced from Unsplash Davao market vendors with credit.</span>
        </div>
        <div class="footer-runtime-info">
          <span>Server response: sub-1ms</span>
          <span>&bull;</span>
          <span>Rust Edition 2024</span>
        </div>
      </div>
    </div>
  </footer>
"##
}
