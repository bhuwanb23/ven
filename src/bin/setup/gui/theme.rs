//! Visual design tokens for the `ven-setup` GUI wizard.
//!
//! v0.2.1 replaces the ad-hoc 8-constant palette with a website-aligned
//! token set, a typed type scale, and bundled fonts. Everything the
//! wizard renders should source its color and size *only* from this
//! module; the per-screen code never reaches for a raw `Color32`.
//!
//! Why bundle fonts: the previous wizard relied on egui's "default
//! fonts" feature, which serves DejaVu Sans on most platforms but
//! falls back to whatever `winit` finds first. The result was that
//! a `RichText::strong()` button looked materially different on
//! Windows vs the website's Inter rendering. Bundling Inter +
//! JetBrains Mono fixes that at the cost of ~900 KB in the binary.

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontId, RichText};

// ---------------------------------------------------------------------------
// Color palette (matches `ven_website/src/styles/tokens` and Tailwind
// dark mode tokens used on `ven.dev`).
// ---------------------------------------------------------------------------

/// Page background — the dark canvas the whole wizard sits on.
pub const BG: Color32 = Color32::from_rgb(0x0c, 0x0c, 0x0e);
/// Header / sidebar panel surface. One step lighter than [`BG`] so the
/// chrome reads as elevated against the central content.
pub const PANEL: Color32 = Color32::from_rgb(0x16, 0x16, 0x1a);
/// Card / popup surface. Two steps lighter than [`BG`] — same role as
/// the website's `--surface-2`.
pub const CARD: Color32 = Color32::from_rgb(0x1c, 0x1c, 0x22);
/// Hover state for cards and `option_card`. Subtle: barely visible
/// unless the card is actually tracked.
pub const CARD_HOVER: Color32 = Color32::from_rgb(0x23, 0x23, 0x2b);

/// Default border on cards / inputs. Reads as a hairline rule, not a
/// hard divider.
pub const BORDER: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x32);
/// Active / focused border. Picks up the brand cyan so a selected
/// option_card or focused input visibly "lights up".
pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(0x00, 0xdb, 0xe7);

/// Primary text — slightly off-white so it doesn't ring against [`BG`].
pub const TEXT: Color32 = Color32::from_rgb(0xfa, 0xfa, 0xfa);
/// Secondary text (descriptions, captions, summary labels).
pub const MUTED: Color32 = Color32::from_rgb(0x9c, 0xa3, 0xaf);

/// Brand cyan — primary buttons, focus rings, the spinner, the
/// "Step N/7" pill, the accent stroke on hero hero-section icons.
pub const ACCENT: Color32 = Color32::from_rgb(0x00, 0xdb, 0xe7);
/// Hover variant of [`ACCENT`]. One shade darker so the click target
/// reads as pressed-in.
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(0x33, 0xe2, 0xec);
/// Dimmed variant for muted accent text (e.g. monospace command hints
/// inside option_card descriptions).
pub const ACCENT_DIM: Color32 = Color32::from_rgb(0x00, 0xa3, 0xad);

/// Success indicator (Done step icons, "Path is writable" line).
pub const SUCCESS: Color32 = Color32::from_rgb(0x22, 0xc5, 0x5e);
/// Warning indicator (dry-run banner, hook/PATH disabled hint).
pub const WARNING: Color32 = Color32::from_rgb(0xf5, 0x9e, 0x0b);
/// Error indicator (failed step icons, validation_line errors).
pub const ERROR: Color32 = Color32::from_rgb(0xef, 0x44, 0x44);

// ---------------------------------------------------------------------------
// Typography scale — matches the website's Tailwind type tokens.
// ---------------------------------------------------------------------------

/// Hero / Done screen headline (28 px). Used sparingly — at most once
/// per screen.
pub const SIZE_DISPLAY: f32 = 28.0;
/// Section title above a card (22 px).
pub const SIZE_HEADING: f32 = 22.0;
/// Inline subheading (16 px) — used for `summary_row` labels and the
/// "Step N: …" rows on the progress screen.
pub const SIZE_SUBHEADING: f32 = 16.0;
/// Body copy (14 px). Default text size for descriptions.
pub const SIZE_BODY: f32 = 14.0;
/// Tertiary / caption text (12 px). Default-path hint, footnotes,
/// keyboard-shortcut hints.
pub const SIZE_CAPTION: f32 = 12.0;

// ---------------------------------------------------------------------------
// Geometry tokens — used by widgets.rs so card padding / corner radius
// are consistent across the wizard.
// ---------------------------------------------------------------------------

/// Corner radius applied to cards, panels, and the cancel modal.
pub const RADIUS_CARD: f32 = 10.0;
/// Corner radius applied to interactive controls (buttons, inputs,
/// the painted progress bar overlay).
pub const RADIUS_CONTROL: f32 = 8.0;
/// Inner padding on a standard card body.
pub const CARD_PADDING: f32 = 16.0;
/// Outer margin between major content blocks.
pub const SECTION_GAP: f32 = 16.0;

// ---------------------------------------------------------------------------
// Pre-styled `RichText` helpers. Per-screen code prefers these over
// hand-rolled `RichText::new(s).size(...).color(...)` chains so the
// type scale stays canonical.
// ---------------------------------------------------------------------------

/// Build a display-sized headline (28 px, primary text, semi-bold).
pub fn display(s: impl Into<String>) -> RichText {
    RichText::new(s).size(SIZE_DISPLAY).strong().color(TEXT)
}

/// Build a section heading (22 px, semi-bold).
pub fn heading(s: impl Into<String>) -> RichText {
    RichText::new(s).size(SIZE_HEADING).strong().color(TEXT)
}

/// Build a card / row title (16 px, semi-bold).
pub fn subheading(s: impl Into<String>) -> RichText {
    RichText::new(s).size(SIZE_SUBHEADING).strong().color(TEXT)
}

/// Build body copy (14 px, regular weight).
pub fn body(s: impl Into<String>) -> RichText {
    RichText::new(s).size(SIZE_BODY).color(TEXT)
}

/// Build muted body copy (14 px, MUTED color).
pub fn muted_body(s: impl Into<String>) -> RichText {
    RichText::new(s).size(SIZE_BODY).color(MUTED)
}

/// Build a caption (12 px, MUTED).
pub fn caption(s: impl Into<String>) -> RichText {
    RichText::new(s).size(SIZE_CAPTION).color(MUTED)
}

/// Build a JetBrains-Mono code line (12 px, accent_dim color).
///
/// Provided for symmetry with the rest of the typography helpers; not
/// currently called because every monospace surface in the wizard
/// builds its own `RichText` (e.g. log-tail rows on the Progress
/// screen, `ven install <slug>` captions on runtime cards). Kept
/// available so future screens get a one-line helper.
#[allow(dead_code)]
pub fn code(s: impl Into<String>) -> RichText {
    RichText::new(s)
        .size(SIZE_CAPTION)
        .family(FontFamily::Monospace)
        .color(ACCENT_DIM)
}

// ---------------------------------------------------------------------------
// Theme entry point — one call from `gui::run`'s creation closure
// installs the palette, font definitions, and visual tweaks.
// ---------------------------------------------------------------------------

/// Bundled font payloads. Loaded at compile time so the binary is
/// fully self-contained — no system font fallback path.
const INTER_REGULAR: &[u8] = include_bytes!("../../../../assets/fonts/Inter-Regular.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../../../assets/fonts/Inter-SemiBold.ttf");
const JETBRAINS_MONO_REGULAR: &[u8] =
    include_bytes!("../../../../assets/fonts/JetBrainsMono-Regular.ttf");

/// Apply the wizard theme to `ctx`. Idempotent: calling more than once
/// just re-asserts the same values (used by the egui hot-reload path
/// in debug builds).
pub fn apply(ctx: &egui::Context) {
    install_fonts(ctx);
    install_visuals(ctx);
    install_styles(ctx);
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    // Register Inter Regular as the new "Proportional" default and
    // Inter SemiBold as a named family the widgets module can pull in
    // for emphasized labels (button text, summary_row values).
    fonts.font_data.insert(
        "inter-regular".to_string(),
        FontData::from_static(INTER_REGULAR),
    );
    fonts.font_data.insert(
        "inter-semibold".to_string(),
        FontData::from_static(INTER_SEMIBOLD),
    );
    fonts.font_data.insert(
        "jetbrains-mono".to_string(),
        FontData::from_static(JETBRAINS_MONO_REGULAR),
    );

    // Proportional family: Inter Regular first, then SemiBold as a
    // fallback so `RichText::strong()` actually picks up the heavier
    // glyph (egui::TextFormat doesn't have a real "weight" field).
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .splice(
            0..0,
            ["inter-regular".to_owned(), "inter-semibold".to_owned()],
        );

    // Monospace: JetBrains Mono first, fall back to whatever was there.
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .splice(0..0, ["jetbrains-mono".to_owned()]);

    // Named family used by `widgets::primary_button` to force the
    // bolder cut without having to ask egui for a "strong" variant.
    fonts
        .families
        .entry(FontFamily::Name("inter-semibold".into()))
        .or_default()
        .push("inter-semibold".to_owned());

    ctx.set_fonts(fonts);
}

fn install_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Surface colors — wallpaper everything in our token set so any
    // egui widget that didn't get a direct override still picks up
    // the brand palette.
    visuals.panel_fill = PANEL;
    visuals.window_fill = CARD;
    visuals.extreme_bg_color = BG;
    visuals.faint_bg_color = CARD;

    // Default text color (used by ui.label without explicit color).
    visuals.override_text_color = Some(TEXT);
    visuals.widgets.noninteractive.fg_stroke.color = TEXT;
    visuals.widgets.inactive.fg_stroke.color = TEXT;
    visuals.widgets.hovered.fg_stroke.color = TEXT;
    visuals.widgets.active.fg_stroke.color = TEXT;

    // Card / button bg fills.
    visuals.widgets.noninteractive.bg_fill = CARD;
    visuals.widgets.inactive.bg_fill = CARD;
    visuals.widgets.hovered.bg_fill = CARD_HOVER;
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.widgets.open.bg_fill = CARD_HOVER;

    // Borders — hairline rules everywhere, accent on focus.
    let hairline = egui::Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.bg_stroke = hairline;
    visuals.widgets.inactive.bg_stroke = hairline;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, BORDER_ACTIVE);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, BORDER_ACTIVE);

    // Selection (text input drag, list row highlight).
    visuals.selection.bg_fill = ACCENT.linear_multiply(0.35);
    visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT);

    // Window shadows — used by the cancel-confirm modal.
    visuals.popup_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 6.0),
        blur: 24.0,
        spread: 0.0,
        color: Color32::from_black_alpha(160),
    };
    visuals.window_shadow = visuals.popup_shadow;

    // Soften widget rounding everywhere so there are no hard
    // 0-radius corners in the wizard.
    let r = egui::Rounding::same(RADIUS_CONTROL);
    visuals.widgets.noninteractive.rounding = r;
    visuals.widgets.inactive.rounding = r;
    visuals.widgets.hovered.rounding = r;
    visuals.widgets.active.rounding = r;
    visuals.widgets.open.rounding = r;
    visuals.window_rounding = egui::Rounding::same(RADIUS_CARD);
    visuals.menu_rounding = egui::Rounding::same(RADIUS_CONTROL);

    ctx.set_visuals(visuals);
}

fn install_styles(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Default text styles — explicitly set every named role so the
    // wizard never falls back to egui's stock 14 px.
    use egui::TextStyle::{Body, Button, Heading, Monospace, Small};
    style
        .text_styles
        .insert(Heading, FontId::new(SIZE_HEADING, FontFamily::Proportional));
    style
        .text_styles
        .insert(Body, FontId::new(SIZE_BODY, FontFamily::Proportional));
    style
        .text_styles
        .insert(Button, FontId::new(SIZE_BODY, FontFamily::Proportional));
    style
        .text_styles
        .insert(Small, FontId::new(SIZE_CAPTION, FontFamily::Proportional));
    style
        .text_styles
        .insert(Monospace, FontId::new(SIZE_CAPTION, FontFamily::Monospace));

    // A bit more breathing room — egui defaults to 4px which feels
    // cramped at the larger viewport size we use.
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.button_padding = egui::vec2(14.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(0.0);
    style.spacing.menu_margin = egui::Margin::same(8.0);

    // Smoother scrollbar: thin floating bars that appear on hover.
    use egui::style::ScrollStyle;
    style.spacing.scroll = ScrollStyle {
        floating: true,
        bar_width: 4.0,
        floating_width: 3.0,
        floating_allocated_width: 4.0,
        handle_min_length: 24.0,
        bar_inner_margin: 2.0,
        bar_outer_margin: 0.0,
        foreground_color: true,
        dormant_background_opacity: 0.0,
        active_background_opacity: 0.3,
        interact_background_opacity: 0.5,
        dormant_handle_opacity: 0.0,
        active_handle_opacity: 0.5,
        interact_handle_opacity: 0.8,
    };

    ctx.set_style(style);
}
