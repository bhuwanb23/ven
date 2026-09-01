//! Reusable, branded widgets for the `ven-setup` wizard.
//!
//! Stock egui widgets (`ui.button`, `ui.checkbox`, `ui.radio`) look
//! generic — fine for an internal tool, wrong for a user-facing
//! installer. Everything in this module paints its own frame using
//! the [`super::theme`] tokens so the wizard reads as a real product.
//!
//! All widgets follow the same conventions:
//!
//! - Return an `egui::Response` so callers can `.clicked()` /
//!   `.changed()` / `.on_hover_text(...)` exactly the way they would
//!   with a stock widget.
//! - Take ownership of their text via `Into<String>` so call sites
//!   can pass `&str`, `String`, or `RichText`.
//! - Never read or write `WizardState` directly — they are
//!   pure render-and-respond primitives.

use eframe::egui::{
    self, vec2, Align, Align2, Color32, FontId, Layout, Rect, Response, RichText, Sense, Stroke,
    TextureHandle, Ui, Vec2,
};

use super::theme;

// ---------------------------------------------------------------------------
// Buttons — three weights matching the website's tone scale.
// ---------------------------------------------------------------------------

/// Primary call-to-action — filled cyan, white text, semi-bold.
///
/// Used once per screen at most: "Get started", "Next", "Install",
/// "Finish".
pub fn primary_button(ui: &mut Ui, label: impl Into<String>) -> Response {
    primary_button_sized(ui, label, vec2(120.0, 36.0))
}

/// Primary button with an explicit minimum size — used by the hero
/// screens which want a wider 200 px CTA.
pub fn primary_button_sized(ui: &mut Ui, label: impl Into<String>, size: Vec2) -> Response {
    let label: String = label.into();
    let (bg, fg) = (theme::ACCENT, Color32::BLACK);
    let (bg_hover, fg_hover) = (theme::ACCENT_HOVER, Color32::BLACK);
    fill_button(ui, label, size, bg, fg, bg_hover, fg_hover, true)
}

/// Destructive primary — filled red. Used by the cancel-confirm modal
/// for the "Exit" action.
pub fn destructive_button(ui: &mut Ui, label: impl Into<String>) -> Response {
    let label: String = label.into();
    let bg = theme::ERROR;
    let bg_hover = Color32::from_rgb(0xff, 0x6b, 0x6b);
    fill_button(
        ui,
        label,
        vec2(96.0, 32.0),
        bg,
        Color32::WHITE,
        bg_hover,
        Color32::WHITE,
        true,
    )
}

/// Secondary action — outlined cyan, transparent fill. Used for the
/// Done screen's "Open documentation".
pub fn secondary_button(ui: &mut Ui, label: impl Into<String>) -> Response {
    outline_button(ui, label, vec2(160.0, 36.0))
}

/// Text-only button — muted text, no border, hover fills with a faint
/// card tone. Used for "Back" and "Cancel".
pub fn text_button(ui: &mut Ui, label: impl Into<String>) -> Response {
    let label: String = label.into();
    let size = vec2(80.0, 32.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let bg_fill = if response.hovered() {
        theme::CARD_HOVER
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, theme::RADIUS_CONTROL, bg_fill);
    let fg = if response.hovered() {
        theme::TEXT
    } else {
        theme::MUTED
    };
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(theme::SIZE_BODY),
        fg,
    );
    response
}

fn fill_button(
    ui: &mut Ui,
    label: String,
    size: Vec2,
    bg: Color32,
    fg: Color32,
    bg_hover: Color32,
    fg_hover: Color32,
    bold: bool,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let (fill, text_color) = if !response.enabled() {
        (bg.linear_multiply(0.4), theme::MUTED)
    } else if response.is_pointer_button_down_on() {
        (bg_hover.linear_multiply(0.85), fg_hover)
    } else if response.hovered() {
        (bg_hover, fg_hover)
    } else {
        (bg, fg)
    };
    painter.rect_filled(rect, theme::RADIUS_CONTROL, fill);
    let mut text = RichText::new(label)
        .size(theme::SIZE_BODY)
        .color(text_color);
    if bold {
        text = text.strong();
    }
    let galley = ui.fonts(|f| {
        f.layout_no_wrap(
            text.text().to_string(),
            FontId::new(theme::SIZE_BODY, egui::FontFamily::Proportional),
            text_color,
        )
    });
    let galley_pos = rect.center() - vec2(galley.size().x * 0.5, galley.size().y * 0.5);
    painter.galley(galley_pos, galley, text_color);
    response
}

fn outline_button(ui: &mut Ui, label: impl Into<String>, size: Vec2) -> Response {
    let label: String = label.into();
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let stroke_color = if response.hovered() {
        theme::ACCENT_HOVER
    } else {
        theme::ACCENT
    };
    let fill = if response.hovered() {
        theme::ACCENT.linear_multiply(0.08)
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, theme::RADIUS_CONTROL, fill);
    painter.rect_stroke(
        rect,
        theme::RADIUS_CONTROL,
        Stroke::new(1.5_f32, stroke_color),
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(theme::SIZE_BODY),
        stroke_color,
    );
    response
}

// ---------------------------------------------------------------------------
// `option_card` — a clickable, branded replacement for `ui.radio` /
// `ui.checkbox`. Used by the Mode / HookPath / Runtimes screens.
// ---------------------------------------------------------------------------

/// Interactive card for a single option (radio or checkbox semantics).
///
/// Layout (left to right):
///
/// ```text
/// ┌──────────────────────────────────────────────────────┐
/// │ ◉   Title (bold)            ◉ icon on right side     │
/// │     Description (muted)                              │
/// │     `command hint` (mono)        ← optional caption  │
/// └──────────────────────────────────────────────────────┘
/// ```
///
/// `caption` may be empty; when non-empty it renders as a JetBrains
/// Mono row at the bottom (used for `ven install <slug> latest` on
/// runtime cards).
///
/// Returns the click response so the caller can `if .clicked() { ... }`
/// to flip the selection.
pub fn option_card(
    ui: &mut Ui,
    selected: bool,
    title: &str,
    description: &str,
    caption_text: Option<&str>,
) -> Response {
    let height_estimate = if caption_text.is_some() { 88.0 } else { 72.0 };
    let width = ui.available_width().max(320.0);
    let (rect, response) = ui.allocate_exact_size(vec2(width, height_estimate), Sense::click());
    let painter = ui.painter();

    let fill = if selected {
        theme::ACCENT.linear_multiply(0.12)
    } else if response.hovered() {
        theme::CARD_HOVER
    } else {
        theme::CARD
    };
    let stroke_color = if selected {
        theme::BORDER_ACTIVE
    } else if response.hovered() {
        theme::BORDER_ACTIVE.linear_multiply(0.5)
    } else {
        theme::BORDER
    };
    let stroke_width = if selected { 2.0_f32 } else { 1.0_f32 };

    painter.rect_filled(rect, theme::RADIUS_CARD, fill);
    painter.rect_stroke(
        rect,
        theme::RADIUS_CARD,
        Stroke::new(stroke_width, stroke_color),
    );

    // Left margin where the radio / check indicator sits.
    let pad = 16.0;
    let indicator_center = egui::pos2(rect.left() + pad + 8.0, rect.top() + pad + 6.0);
    paint_indicator(painter, indicator_center, selected);

    // Title + description column.
    let text_x = rect.left() + pad + 32.0;
    let title_pos = egui::pos2(text_x, rect.top() + pad);
    painter.text(
        title_pos,
        Align2::LEFT_TOP,
        title,
        FontId::new(theme::SIZE_SUBHEADING, egui::FontFamily::Proportional),
        theme::TEXT,
    );
    let desc_pos = egui::pos2(text_x, rect.top() + pad + 22.0);
    painter.text(
        desc_pos,
        Align2::LEFT_TOP,
        description,
        FontId::new(theme::SIZE_BODY, egui::FontFamily::Proportional),
        theme::MUTED,
    );

    if let Some(caption) = caption_text {
        let cap_pos = egui::pos2(text_x, rect.top() + pad + 44.0);
        painter.text(
            cap_pos,
            Align2::LEFT_TOP,
            caption,
            FontId::new(theme::SIZE_CAPTION, egui::FontFamily::Monospace),
            theme::ACCENT_DIM,
        );
    }

    response
}

fn paint_indicator(painter: &egui::Painter, center: egui::Pos2, selected: bool) {
    let radius = 8.0;
    if selected {
        painter.circle_filled(center, radius, theme::ACCENT);
        // Inner check mark (small).
        let p1 = center + vec2(-3.5, 0.5);
        let p2 = center + vec2(-1.0, 3.0);
        let p3 = center + vec2(4.0, -2.5);
        painter.line_segment([p1, p2], Stroke::new(1.8_f32, Color32::BLACK));
        painter.line_segment([p2, p3], Stroke::new(1.8_f32, Color32::BLACK));
    } else {
        painter.circle_stroke(center, radius, Stroke::new(1.5_f32, theme::BORDER));
    }
}

// ---------------------------------------------------------------------------
// Step rail — vertical list of "1 Welcome", "2 Mode", … on the left
// of the wizard window.
// ---------------------------------------------------------------------------

/// Visual status of a step in the left rail.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StepRailStatus {
    /// Already passed by the user (rendered with a check mark).
    Done,
    /// The current screen.
    Active,
    /// Future step (rendered muted).
    Upcoming,
}

/// Render one row of the left rail. Non-interactive — clicking a step
/// in the rail is intentionally not allowed (forward jumps would skip
/// validation).
pub fn step_rail_row(ui: &mut Ui, idx: usize, status: StepRailStatus, label: &str) {
    let row_height = 36.0;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), row_height), Sense::hover());
    let painter = ui.painter();

    if status == StepRailStatus::Active {
        // Soft cyan tint behind the active row.
        painter.rect_filled(
            rect.expand2(vec2(-4.0, 2.0)),
            theme::RADIUS_CONTROL,
            theme::ACCENT.linear_multiply(0.10),
        );
    }

    let bullet_center = egui::pos2(rect.left() + 18.0, rect.center().y);
    match status {
        StepRailStatus::Done => {
            painter.circle_filled(bullet_center, 10.0, theme::SUCCESS);
            paint_check_glyph(painter, bullet_center, 5.0, Color32::BLACK);
        }
        StepRailStatus::Active => {
            painter.circle_filled(bullet_center, 10.0, theme::ACCENT);
            painter.text(
                bullet_center,
                Align2::CENTER_CENTER,
                idx.to_string(),
                FontId::new(theme::SIZE_CAPTION, egui::FontFamily::Proportional),
                Color32::BLACK,
            );
        }
        StepRailStatus::Upcoming => {
            painter.circle_stroke(bullet_center, 10.0, Stroke::new(1.0_f32, theme::BORDER));
            painter.text(
                bullet_center,
                Align2::CENTER_CENTER,
                idx.to_string(),
                FontId::new(theme::SIZE_CAPTION, egui::FontFamily::Proportional),
                theme::MUTED,
            );
        }
    }

    let label_color = match status {
        StepRailStatus::Done => theme::TEXT,
        StepRailStatus::Active => theme::TEXT,
        StepRailStatus::Upcoming => theme::MUTED,
    };
    let label_pos = egui::pos2(rect.left() + 38.0, rect.center().y);
    painter.text(
        label_pos,
        Align2::LEFT_CENTER,
        label,
        FontId::new(theme::SIZE_BODY, egui::FontFamily::Proportional),
        label_color,
    );
}

fn paint_check_glyph(painter: &egui::Painter, center: egui::Pos2, scale: f32, color: Color32) {
    let p1 = center + vec2(-scale * 0.7, scale * 0.1);
    let p2 = center + vec2(-scale * 0.2, scale * 0.6);
    let p3 = center + vec2(scale * 0.7, -scale * 0.5);
    painter.line_segment([p1, p2], Stroke::new(1.6_f32, color));
    painter.line_segment([p2, p3], Stroke::new(1.6_f32, color));
}

// ---------------------------------------------------------------------------
// Validation lines — a single row with an icon + message, used under
// the Storage path field and as the dry-run banner on Review.
// ---------------------------------------------------------------------------

/// Tone of a validation_line.
pub enum ValidKind {
    Ok,
    Warn,
    Error,
}

/// Render `[icon] [msg]` in the appropriate tone color.
pub fn validation_line(ui: &mut Ui, kind: ValidKind, msg: &str) {
    let (color, glyph) = match kind {
        ValidKind::Ok => (theme::SUCCESS, "✓"),
        ValidKind::Warn => (theme::WARNING, "!"),
        ValidKind::Error => (theme::ERROR, "✗"),
    };
    ui.horizontal(|ui| {
        ui.add_space(2.0);
        // Painted bullet so the icon stays consistent across systems
        // (some glyphs render at very different metrics depending on
        // the underlying font).
        let (rect, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
        let painter = ui.painter();
        painter.circle_filled(rect.center(), 8.0, color.linear_multiply(0.18));
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            glyph,
            FontId::new(theme::SIZE_CAPTION, egui::FontFamily::Proportional),
            color,
        );
        ui.add_space(2.0);
        ui.label(RichText::new(msg).size(theme::SIZE_BODY).color(color));
    });
}

// ---------------------------------------------------------------------------
// Headings, hero artwork, painted check.
// ---------------------------------------------------------------------------

/// Render `title` + optional `subtitle` at the top of a content area.
pub fn section_heading(ui: &mut Ui, title: &str, subtitle: Option<&str>) {
    ui.label(theme::heading(title));
    if let Some(s) = subtitle {
        ui.add_space(4.0);
        ui.label(theme::muted_body(s));
    }
    ui.add_space(theme::SECTION_GAP);
}

/// Centered hero logo on the Welcome screen. Falls back to a painted
/// circle if the texture hasn't loaded yet (won't normally happen
/// because `mod.rs::ensure_logo` runs before `update`).
pub fn hero_logo(ui: &mut Ui, tex: Option<&TextureHandle>, size: f32) {
    ui.vertical_centered(|ui| {
        if let Some(t) = tex {
            ui.image((t.id(), Vec2::splat(size)));
        } else {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
            ui.painter()
                .circle_filled(rect.center(), size * 0.5, theme::ACCENT);
        }
    });
}

/// Hero check / X icon for the Done screen. `size` is the diameter
/// of the painted circle.
pub fn check_circle(ui: &mut Ui, success: bool, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let painter = ui.painter();
    let (bg, fg, glyph) = if success {
        (theme::SUCCESS, Color32::BLACK, "✓")
    } else {
        (theme::ERROR, Color32::WHITE, "✗")
    };
    painter.circle_filled(rect.center(), size * 0.5, bg);
    let _ = glyph;
    // Painted glyph so it's crisp at large sizes.
    if success {
        let s = size * 0.18;
        let c = rect.center();
        let p1 = c + vec2(-s * 1.3, 0.0);
        let p2 = c + vec2(-s * 0.2, s * 1.2);
        let p3 = c + vec2(s * 1.4, -s * 1.0);
        painter.line_segment([p1, p2], Stroke::new(size * 0.06, fg));
        painter.line_segment([p2, p3], Stroke::new(size * 0.06, fg));
    } else {
        let s = size * 0.22;
        let c = rect.center();
        painter.line_segment(
            [c + vec2(-s, -s), c + vec2(s, s)],
            Stroke::new(size * 0.06, fg),
        );
        painter.line_segment(
            [c + vec2(-s, s), c + vec2(s, -s)],
            Stroke::new(size * 0.06, fg),
        );
    }
}

// ---------------------------------------------------------------------------
// Card frame helper — thin wrapper over egui::Frame using theme tokens.
// ---------------------------------------------------------------------------

/// Render `add_contents` inside a branded card (CARD fill, BORDER stroke,
/// rounded corners, [`theme::CARD_PADDING`] inner margin).
pub fn card<R>(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
    egui::Frame::none()
        .fill(theme::CARD)
        .stroke(Stroke::new(1.0_f32, theme::BORDER))
        .inner_margin(theme::CARD_PADDING)
        .rounding(theme::RADIUS_CARD)
        .show(ui, add_contents)
}

/// Render a centered label inside a small "pill" frame — used by the
/// header for the version tag. Currently unused: the title bar now paints the
/// version badge directly via the painter for precise layout with the logo.
#[expect(dead_code)]
pub fn version_pill(ui: &mut Ui, version: &str) -> Response {
    let label = format!("v{version}");
    let size = vec2(72.0, 22.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        theme::RADIUS_CONTROL,
        theme::ACCENT.linear_multiply(0.15),
    );
    painter.rect_stroke(
        rect,
        theme::RADIUS_CONTROL,
        Stroke::new(1.0_f32, theme::ACCENT.linear_multiply(0.5)),
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::new(theme::SIZE_CAPTION, egui::FontFamily::Proportional),
        theme::ACCENT,
    );
    response
}

// ---------------------------------------------------------------------------
// Footer layout helper used by `screens::draw_nav` so Back / Cancel /
// primary action stay aligned consistently.
// ---------------------------------------------------------------------------

/// Reserve the standard two-row spacing above the footer divider.
///
/// Currently `screens::draw_nav` paints its own divider via the bottom
/// panel's frame, so this helper is unused by the v0.2.1 layout. Kept
/// for screens that opt in to a manual footer (e.g. a future "splash"
/// screen with a custom button strip) so the spacing stays consistent.
#[allow(dead_code)]
pub fn footer_padding(ui: &mut Ui) {
    ui.add_space(8.0);
    ui.painter().line_segment(
        [
            ui.cursor().min,
            egui::pos2(ui.cursor().min.x + ui.available_width(), ui.cursor().min.y),
        ],
        Stroke::new(1.0_f32, theme::BORDER),
    );
    ui.add_space(8.0);
}

/// Lay out the standard footer button row: `[Back .... Cancel|Primary]`.
///
/// `add_left` runs in the left slot (Back). `add_right` runs in the
/// right slot (Cancel + primary action) and is given a right-to-left
/// layout so the primary CTA sits flush right.
///
/// Not currently used by `screens::draw_nav` (which inlines this layout
/// because two `&mut state` closures can't both be FnOnce-captured at
/// the same time without a borrow-check workaround). Retained as a
/// drop-in helper for future screens with a single-borrow footer.
#[allow(dead_code)]
pub fn footer_row(ui: &mut Ui, add_left: impl FnOnce(&mut Ui), add_right: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        add_left(ui);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            add_right(ui);
        });
    });
}

/// Title bar button — small square for minimize/close in the custom title bar.
pub fn title_bar_button(
    ui: &mut Ui,
    label: &str,
    size: Vec2,
    hover_bg: Color32,
    fg: Color32,
) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter();
    let bg = if response.hovered() {
        hover_bg
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(rect, 0.0, bg);
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::new(theme::SIZE_BODY, egui::FontFamily::Proportional),
        fg,
    );
    response
}

/// Close/X button for the custom title bar.
pub fn close_button(ui: &mut Ui) -> Response {
    title_bar_button(
        ui,
        "✕",
        vec2(40.0, 36.0),
        theme::ERROR.linear_multiply(0.7),
        theme::TEXT,
    )
}

/// Minimize button for the custom title bar.
pub fn minimize_button(ui: &mut Ui) -> Response {
    title_bar_button(ui, "—", vec2(40.0, 36.0), theme::CARD_HOVER, theme::MUTED)
}

/// Helper used by the Review screen — renders a `(label, value)`
/// row inside a card body.
pub fn summary_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            vec2(180.0, 22.0),
            egui::Label::new(theme::muted_body(label)).truncate(),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(theme::body(value).strong());
        });
    });
    ui.add_space(6.0);
    let (sep_rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    let _ = sep_rect; // separator is just visual
    ui.painter().line_segment(
        [
            egui::pos2(sep_rect.left(), sep_rect.center().y),
            egui::pos2(sep_rect.right(), sep_rect.center().y),
        ],
        Stroke::new(1.0_f32, theme::BORDER.linear_multiply(0.5)),
    );
    ui.add_space(6.0);
}

/// Painted-rect helper used by the Progress screen for the big
/// outer progress bar. Returns the rect so the caller can overlay
/// percentage text.
pub fn big_progress_bar(ui: &mut Ui, fraction: f32, height: f32) -> Rect {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, theme::RADIUS_CONTROL, theme::CARD);
    painter.rect_stroke(
        rect,
        theme::RADIUS_CONTROL,
        Stroke::new(1.0_f32, theme::BORDER),
    );
    let fill_width = (rect.width() * fraction.clamp(0.0, 1.0)).max(0.0);
    let fill_rect = Rect::from_min_size(rect.min, vec2(fill_width, rect.height()));
    if fill_width > 1.0 {
        painter.rect_filled(fill_rect, theme::RADIUS_CONTROL, theme::ACCENT);
    }
    rect
}
