// Pure Rust Style System for Suki Davao Landing Page
// Defined 100% in Rust using typed Property enum. Zero external CSS files.

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Property {
    Custom(&'static str),
    WebkitFontSmoothing,
    AlignItems,
    AlignSelf,
    Animation,
    AnimationDelay,
    AnimationPlayState,
    AnimationRange,
    AnimationTimeline,
    Background,
    BackgroundColor,
    BackgroundImage,
    BackgroundSize,
    Border,
    BorderBlock,
    BorderBottom,
    BorderColor,
    BorderLeft,
    BorderRadius,
    BorderStyle,
    BorderTop,
    Bottom,
    BoxSizing,
    Color,
    ColorScheme,
    ColumnGap,
    Content,
    Cursor,
    Display,
    Fill,
    Flex,
    FlexDirection,
    FlexShrink,
    FlexWrap,
    Font,
    FontFamily,
    FontSize,
    FontStyle,
    FontWeight,
    Gap,
    GridColumn,
    GridRow,
    GridTemplateColumns,
    Height,
    Inset,
    Isolation,
    JustifyContent,
    Left,
    LetterSpacing,
    LineHeight,
    ListStyle,
    Margin,
    MarginBlock,
    MarginBottom,
    MarginInline,
    MarginLeft,
    MarginRight,
    MarginTop,
    MaxWidth,
    MinHeight,
    MinWidth,
    Opacity,
    Outline,
    OutlineOffset,
    Overflow,
    OverflowWrap,
    Padding,
    PaddingBlock,
    PaddingBottom,
    PaddingInline,
    PaddingLeft,
    PaddingRight,
    PaddingTop,
    PointerEvents,
    Position,
    Right,
    Scale,
    ScrollBehavior,
    ScrollPaddingTop,
    Stroke,
    StrokeWidth,
    TextAlign,
    TextDecoration,
    TextDecorationColor,
    TextTransform,
    TextUnderlineOffset,
    Top,
    Transform,
    Transition,
    Translate,
    VectorEffect,
    WhiteSpace,
    Width,
    ZIndex,
}

#[allow(dead_code)]
impl Property {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Custom(name) => name,
            Self::WebkitFontSmoothing => "-webkit-font-smoothing",
            Self::AlignItems => "align-items",
            Self::AlignSelf => "align-self",
            Self::Animation => "animation",
            Self::AnimationDelay => "animation-delay",
            Self::AnimationPlayState => "animation-play-state",
            Self::AnimationRange => "animation-range",
            Self::AnimationTimeline => "animation-timeline",
            Self::Background => "background",
            Self::BackgroundColor => "background-color",
            Self::BackgroundImage => "background-image",
            Self::BackgroundSize => "background-size",
            Self::Border => "border",
            Self::BorderBlock => "border-block",
            Self::BorderBottom => "border-bottom",
            Self::BorderColor => "border-color",
            Self::BorderLeft => "border-left",
            Self::BorderRadius => "border-radius",
            Self::BorderStyle => "border-style",
            Self::BorderTop => "border-top",
            Self::Bottom => "bottom",
            Self::BoxSizing => "box-sizing",
            Self::Color => "color",
            Self::ColorScheme => "color-scheme",
            Self::ColumnGap => "column-gap",
            Self::Content => "content",
            Self::Cursor => "cursor",
            Self::Display => "display",
            Self::Fill => "fill",
            Self::Flex => "flex",
            Self::FlexDirection => "flex-direction",
            Self::FlexShrink => "flex-shrink",
            Self::FlexWrap => "flex-wrap",
            Self::Font => "font",
            Self::FontFamily => "font-family",
            Self::FontSize => "font-size",
            Self::FontStyle => "font-style",
            Self::FontWeight => "font-weight",
            Self::Gap => "gap",
            Self::GridColumn => "grid-column",
            Self::GridRow => "grid-row",
            Self::GridTemplateColumns => "grid-template-columns",
            Self::Height => "height",
            Self::Inset => "inset",
            Self::Isolation => "isolation",
            Self::JustifyContent => "justify-content",
            Self::Left => "left",
            Self::LetterSpacing => "letter-spacing",
            Self::LineHeight => "line-height",
            Self::ListStyle => "list-style",
            Self::Margin => "margin",
            Self::MarginBlock => "margin-block",
            Self::MarginBottom => "margin-bottom",
            Self::MarginInline => "margin-inline",
            Self::MarginLeft => "margin-left",
            Self::MarginRight => "margin-right",
            Self::MarginTop => "margin-top",
            Self::MaxWidth => "max-width",
            Self::MinHeight => "min-height",
            Self::MinWidth => "min-width",
            Self::Opacity => "opacity",
            Self::Outline => "outline",
            Self::OutlineOffset => "outline-offset",
            Self::Overflow => "overflow",
            Self::OverflowWrap => "overflow-wrap",
            Self::Padding => "padding",
            Self::PaddingBlock => "padding-block",
            Self::PaddingBottom => "padding-bottom",
            Self::PaddingInline => "padding-inline",
            Self::PaddingLeft => "padding-left",
            Self::PaddingRight => "padding-right",
            Self::PaddingTop => "padding-top",
            Self::PointerEvents => "pointer-events",
            Self::Position => "position",
            Self::Right => "right",
            Self::Scale => "scale",
            Self::ScrollBehavior => "scroll-behavior",
            Self::ScrollPaddingTop => "scroll-padding-top",
            Self::Stroke => "stroke",
            Self::StrokeWidth => "stroke-width",
            Self::TextAlign => "text-align",
            Self::TextDecoration => "text-decoration",
            Self::TextDecorationColor => "text-decoration-color",
            Self::TextTransform => "text-transform",
            Self::TextUnderlineOffset => "text-underline-offset",
            Self::Top => "top",
            Self::Transform => "transform",
            Self::Transition => "transition",
            Self::Translate => "translate",
            Self::VectorEffect => "vector-effect",
            Self::WhiteSpace => "white-space",
            Self::Width => "width",
            Self::ZIndex => "z-index",
        }
    }
}

pub fn render_css() -> String {
    let mut s = String::with_capacity(32768);
    // Root Design Tokens
    s.push_str(r#":root {
  --p-bg: #fbf9f4;
  --p-surface: #ffffff;
  --p-surface-muted: #f5f1e8;
  --p-surface-elevated: #ffffff;
  --p-ink: #12362a;
  --p-muted: #536b60;
  --p-subtle: #7f948b;
  --p-accent: #194d3c;
  --p-accent-hover: #123d2f;
  --p-accent-light: #e6efe8;
  --p-gold: #c98b2c;
  --p-border: #e3ded5;
  --p-border-light: #eeebe3;
  --p-tag-bg: #eaf1ed;
  --p-tag-ink: #164e3b;
  --p-savings-bg: #dcfce7;
  --p-savings-ink: #14532d;
  --p-shadow-sm: 0 1px 3px rgba(18, 54, 42, 0.05);
  --p-shadow-md: 0 4px 16px -2px rgba(18, 54, 42, 0.08);
  --p-shadow-lg: 0 12px 32px -4px rgba(18, 54, 42, 0.12);
  --radius-sm: 6px;
  --radius-md: 12px;
  --radius-lg: 18px;
  --radius-full: 9999px;
  --font-display: "Manrope", system-ui, -apple-system, sans-serif;
  --font-text: "Manrope", system-ui, -apple-system, sans-serif;
  --font-mono: "JetBrains Mono", ui-monospace, monospace;
}
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
html { font-family: var(--font-text); color: var(--p-ink); background-color: var(--p-bg); line-height: 1.5; -webkit-font-smoothing: antialiased; scroll-behavior: smooth; }
body { min-height: 100vh; display: flex; flex-direction: column; }
h1, h2, h3, h4, h5, h6 { font-family: var(--font-display); color: var(--p-ink); font-weight: 700; letter-spacing: -0.025em; line-height: 1.2; }
p { color: var(--p-muted); }
a { color: inherit; text-decoration: none; }
.announcement-bar { background-color: var(--p-ink); color: #fbf9f4; font-size: 0.8125rem; padding: 8px 16px; border-bottom: 1px solid rgba(255,255,255,0.1); }
.announcement-content { max-width: 1200px; margin: 0 auto; display: flex; align-items: center; justify-content: center; gap: 10px; flex-wrap: wrap; }
.status-dot { width: 8px; height: 8px; background-color: #4ade80; border-radius: 50%; display: inline-block; }
.announcement-link { color: #dafa9c; font-weight: 600; margin-left: 6px; }
.announcement-link:hover { text-decoration: underline; }
.main-header { position: sticky; top: 0; background-color: rgba(251, 249, 244, 0.94); backdrop-filter: blur(12px); border-bottom: 1px solid var(--p-border); z-index: 100; }
.header-inner { max-width: 1200px; margin: 0 auto; padding: 14px 24px; display: flex; align-items: center; justify-content: space-between; }
.brand-logo { display: flex; align-items: center; gap: 10px; }
.logo-leaf-icon { width: 36px; height: 36px; background-color: var(--p-accent); color: #ffffff; border-radius: var(--radius-sm); display: flex; align-items: center; justify-content: center; }
.logo-text-group { display: flex; flex-direction: column; }
.brand-name { font-weight: 800; font-size: 1.125rem; letter-spacing: 0.05em; color: var(--p-ink); }
.brand-sub { font-size: 0.65rem; letter-spacing: 0.12em; color: var(--p-muted); font-weight: 600; }
.desktop-nav { display: flex; align-items: center; gap: 24px; }
.nav-item { font-size: 0.875rem; font-weight: 600; color: var(--p-muted); transition: color 0.15s ease; }
.nav-item:hover { color: var(--p-ink); }
.header-actions { display: flex; align-items: center; gap: 12px; }
.btn-primary, .btn-primary-lg { background-color: var(--p-accent); color: #ffffff; font-weight: 600; border-radius: var(--radius-sm); display: inline-flex; align-items: center; justify-content: center; gap: 8px; border: 1px solid transparent; cursor: pointer; transition: all 0.15s ease; }
.btn-primary { padding: 8px 16px; font-size: 0.875rem; }
.btn-primary-lg { padding: 14px 24px; font-size: 1rem; }
.btn-primary:hover, .btn-primary-lg:hover { background-color: var(--p-accent-hover); transform: translateY(-1px); box-shadow: var(--p-shadow-md); }
.btn-secondary-lg { background-color: transparent; color: var(--p-ink); border: 1px solid var(--p-border); padding: 14px 24px; font-size: 1rem; font-weight: 600; border-radius: var(--radius-sm); display: inline-flex; align-items: center; justify-content: center; transition: all 0.15s ease; }
.btn-secondary-lg:hover { background-color: var(--p-surface); border-color: var(--p-subtle); transform: translateY(-1px); }
.btn-ghost { padding: 8px 16px; font-size: 0.875rem; font-weight: 600; color: var(--p-ink); border-radius: var(--radius-sm); }
.hero-section { padding: 64px 24px; min-height: calc(100vh - 120px); display: flex; align-items: center; }
.hero-container { max-width: 1200px; margin: 0 auto; display: grid; grid-template-columns: 1.15fr 0.85fr; gap: 56px; align-items: center; }
.hero-pill { display: inline-flex; align-items: center; gap: 8px; background-color: var(--p-surface); border: 1px solid var(--p-border); border-radius: var(--radius-full); padding: 4px 12px 4px 6px; margin-bottom: 24px; box-shadow: var(--p-shadow-sm); }
.pill-badge { background-color: var(--p-accent); color: #ffffff; font-size: 0.6875rem; font-weight: 700; letter-spacing: 0.05em; padding: 2px 8px; border-radius: var(--radius-full); }
.pill-text { font-size: 0.8125rem; font-weight: 600; color: var(--p-ink); }
.hero-headline { font-size: clamp(2.5rem, 4.5vw, 3.75rem); font-weight: 800; line-height: 1.12; margin-bottom: 20px; color: var(--p-ink); }
.hero-headline .text-accent { color: var(--p-accent); }
.hero-subhead { font-size: 1.125rem; line-height: 1.6; color: var(--p-muted); margin-bottom: 32px; max-width: 540px; }
.hero-cta-group { display: flex; align-items: center; gap: 16px; margin-bottom: 40px; flex-wrap: wrap; }
.hero-trust-band { display: flex; align-items: center; gap: 28px; padding-top: 24px; border-top: 1px solid var(--p-border); flex-wrap: wrap; }
.trust-item { display: flex; align-items: center; gap: 8px; font-size: 0.8125rem; font-weight: 600; color: var(--p-ink); }
.trust-check { color: var(--p-accent); }
.hero-visual { position: relative; }
.hero-image-frame { border-radius: var(--radius-lg); overflow: hidden; border: 1px solid var(--p-border); box-shadow: var(--p-shadow-lg); background-color: var(--p-surface); }
.hero-main-img { width: 100%; height: 480px; object-fit: cover; display: block; }
.floating-status-card { position: absolute; bottom: -24px; left: -24px; background: var(--p-surface); border: 1px solid var(--p-border); border-radius: var(--radius-md); padding: 14px 18px; display: flex; align-items: center; gap: 12px; box-shadow: var(--p-shadow-md); z-index: 10; }
.status-card-icon { width: 36px; height: 36px; border-radius: 50%; background-color: var(--p-savings-bg); color: var(--p-savings-ink); display: flex; align-items: center; justify-content: center; }
.status-card-label { font-size: 0.6875rem; text-transform: uppercase; letter-spacing: 0.08em; color: var(--p-muted); font-weight: 700; }
.status-card-val { font-size: 0.9375rem; font-weight: 700; color: var(--p-ink); }
.section-wrapper { padding: 96px 24px; }
.section-wrapper.surface-muted { background-color: var(--p-surface-muted); }
.section-container { max-width: 1200px; margin: 0 auto; }
.section-header { margin-bottom: 48px; max-width: 680px; }
.section-kicker { font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.12em; color: var(--p-accent); font-weight: 800; margin-bottom: 8px; display: block; }
.section-title { font-size: clamp(2rem, 3vw, 2.75rem); font-weight: 800; margin-bottom: 16px; color: var(--p-ink); }
.section-subtitle { font-size: 1.0625rem; line-height: 1.6; color: var(--p-muted); }
.hubs-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px; }
.hub-card { background: var(--p-surface); border: 1px solid var(--p-border); border-radius: var(--radius-md); padding: 28px; display: flex; flex-direction: column; justify-content: space-between; gap: 20px; transition: all 0.2s ease; }
.hub-card:hover { transform: translateY(-2px); box-shadow: var(--p-shadow-md); border-color: var(--p-accent); }
.hub-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; }
.hub-tag { font-size: 0.6875rem; text-transform: uppercase; letter-spacing: 0.08em; color: var(--p-accent); font-weight: 700; display: block; margin-bottom: 4px; }
.hub-title { font-size: 1.25rem; font-weight: 800; color: var(--p-ink); }
.hub-vendor-badge { display: inline-flex; align-items: center; gap: 6px; font-size: 0.75rem; font-weight: 600; color: var(--p-muted); background: var(--p-surface-muted); padding: 4px 8px; border-radius: var(--radius-full); }
.hub-indicator { width: 6px; height: 6px; background-color: #22c55e; border-radius: 50%; }
.hub-desc { font-size: 0.875rem; line-height: 1.6; color: var(--p-muted); }
.hub-pills { display: flex; flex-wrap: wrap; gap: 6px; }
.hub-pill { font-size: 0.6875rem; font-weight: 600; background: var(--p-tag-bg); color: var(--p-tag-ink); padding: 3px 8px; border-radius: var(--radius-sm); }
.hub-footer { display: flex; justify-content: space-between; font-size: 0.75rem; color: var(--p-subtle); border-top: 1px solid var(--p-border-light); padding-top: 14px; }
.hub-meta { display: flex; align-items: center; gap: 6px; }
.price-table-wrapper { background: var(--p-surface); border: 1px solid var(--p-border); border-radius: var(--radius-md); overflow: hidden; box-shadow: var(--p-shadow-sm); }
.price-table { width: 100%; border-collapse: collapse; text-align: left; }
.price-table th { background: var(--p-surface-muted); padding: 14px 18px; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.08em; font-weight: 700; color: var(--p-muted); border-bottom: 1px solid var(--p-border); }
.price-table td { padding: 16px 18px; border-bottom: 1px solid var(--p-border-light); font-size: 0.875rem; vertical-align: middle; }
.price-row:hover { background-color: rgba(25, 77, 60, 0.02); }
.item-thumb-wrapper { display: flex; align-items: center; gap: 12px; }
.item-thumb { width: 44px; height: 44px; border-radius: var(--radius-sm); object-fit: cover; border: 1px solid var(--p-border); }
.item-title { font-weight: 700; color: var(--p-ink); display: block; }
.item-origin { font-size: 0.75rem; color: var(--p-subtle); }
.category-badge { font-size: 0.6875rem; font-weight: 600; background: var(--p-surface-muted); color: var(--p-muted); padding: 3px 8px; border-radius: var(--radius-full); }
.price-palengke { font-family: var(--font-mono); font-weight: 700; color: var(--p-accent); font-size: 0.9375rem; }
.price-supermarket { font-family: var(--font-mono); color: var(--p-subtle); text-decoration: line-through; }
.savings-tag { display: inline-block; background: var(--p-savings-bg); color: var(--p-savings-ink); font-weight: 700; font-size: 0.75rem; padding: 3px 8px; border-radius: var(--radius-full); }
.btn-add-quick { background: transparent; border: 1px solid var(--p-border); color: var(--p-accent); font-weight: 700; font-size: 0.75rem; padding: 6px 12px; border-radius: var(--radius-sm); cursor: pointer; transition: all 0.15s; }
.btn-add-quick:hover { background: var(--p-accent); color: #ffffff; border-color: var(--p-accent); }
.simulator-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 32px; background: var(--p-surface); border: 1px solid var(--p-border); border-radius: var(--radius-lg); padding: 36px; box-shadow: var(--p-shadow-sm); }
.sim-controls-side { display: flex; flex-direction: column; gap: 20px; }
.preset-buttons-row { display: flex; gap: 8px; flex-wrap: wrap; }
.btn-preset { background: var(--p-surface-muted); border: 1px solid var(--p-border); color: var(--p-ink); font-size: 0.75rem; font-weight: 600; padding: 6px 12px; border-radius: var(--radius-full); cursor: pointer; }
.btn-preset:hover { background: var(--p-accent-light); border-color: var(--p-accent); color: var(--p-accent); }
.sim-select { width: 100%; padding: 10px 14px; border: 1px solid var(--p-border); border-radius: var(--radius-sm); font-size: 0.875rem; color: var(--p-ink); background: var(--p-surface); }
.basket-list-wrapper { border: 1px solid var(--p-border); border-radius: var(--radius-sm); max-height: 260px; overflow-y: auto; background: var(--p-bg); }
.basket-item-row { display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; border-bottom: 1px solid var(--p-border-light); font-size: 0.8125rem; }
.basket-item-row:last-child { border-bottom: none; }
.qty-control { display: flex; align-items: center; gap: 6px; }
.qty-btn { width: 22px; height: 22px; border-radius: 4px; border: 1px solid var(--p-border); background: #ffffff; font-weight: 700; cursor: pointer; display: flex; align-items: center; justify-content: center; }
.sim-results-side { background: var(--p-surface-muted); border: 1px solid var(--p-border); border-radius: var(--radius-md); padding: 24px; display: flex; flex-direction: column; justify-content: space-between; }
.quote-breakdown-row { display: flex; justify-content: space-between; padding: 8px 0; font-size: 0.875rem; color: var(--p-muted); }
.quote-breakdown-row.total-row { border-top: 2px solid var(--p-border); padding-top: 14px; margin-top: 8px; font-size: 1.25rem; font-weight: 800; color: var(--p-ink); }
.savings-highlight-box { background: var(--p-savings-bg); color: var(--p-savings-ink); padding: 12px 16px; border-radius: var(--radius-sm); margin: 16px 0; font-weight: 700; font-size: 0.875rem; text-align: center; }
.btn-simulate-order { width: 100%; background: var(--p-accent); color: #ffffff; padding: 14px; border: none; border-radius: var(--radius-sm); font-weight: 700; font-size: 0.9375rem; cursor: pointer; }
.site-footer { background: var(--p-ink); color: #fbf9f4; padding: 64px 24px 32px; border-top: 1px solid rgba(255,255,255,0.1); }
.footer-container { max-width: 1200px; margin: 0 auto; display: flex; flex-direction: column; gap: 48px; }
.footer-top-grid { display: grid; grid-template-columns: 2fr 1fr 1fr 1.5fr; gap: 48px; }
.footer-brand-desc { font-size: 0.875rem; color: rgba(251,249,244,0.7); line-height: 1.6; max-width: 320px; margin-top: 12px; }
.footer-heading { font-size: 0.8125rem; text-transform: uppercase; letter-spacing: 0.08em; color: rgba(251,249,244,0.5); font-weight: 700; margin-bottom: 16px; }
.footer-links-list { list-style: none; display: flex; flex-direction: column; gap: 10px; font-size: 0.875rem; color: rgba(251,249,244,0.8); }
.footer-links-list a:hover { color: #dafa9c; }
.footer-bottom { border-top: 1px solid rgba(255,255,255,0.1); padding-top: 24px; display: flex; justify-content: space-between; font-size: 0.75rem; color: rgba(251,249,244,0.5); flex-wrap: wrap; gap: 12px; }
.footer-runtime-info { display: flex; gap: 8px; color: #dafa9c; font-family: var(--font-mono); }
@media (max-width: 900px) {
  .hero-container { grid-template-columns: 1fr; gap: 36px; }
  .hubs-grid { grid-template-columns: 1fr; }
  .simulator-grid { grid-template-columns: 1fr; padding: 20px; }
  .footer-top-grid { grid-template-columns: 1fr; gap: 32px; }
  .desktop-nav { display: none; }
}
"#);
    s
}
