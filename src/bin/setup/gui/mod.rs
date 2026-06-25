//! Native GUI wizard for `ven-setup` (eframe / egui).
//!
//! v0.2.1 layout:
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │ [logo] Ven Setup                                  v0.2.1   │   <- top header (64 px)
//! ├──────────────┬─────────────────────────────────────────────┤
//! │ 1 Welcome ✓  │                                             │
//! │ 2 Mode ●     │   Central content (scrollable, 32 px pad)   │
//! │ 3 Storage    │                                             │
//! │ 4 Hook/PATH  │                                             │
//! │ 5 Runtimes   │                                             │
//! │ 6 Review     │                                             │
//! │ 7 Install    │                                             │
//! ├──────────────┴─────────────────────────────────────────────┤
//! │ Back                          Cancel    Next / Install     │   <- footer (56 px)
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! The left rail is hidden on `Screen::Welcome` and `Screen::Done`
//! (both hero layouts that own the full canvas).

mod screens;
mod state;
mod theme;
mod widgets;
mod worker;

pub use state::Screen;

use eframe::egui;

use crate::common::SetupCli;
use crate::gui::screens::{draw_central, draw_nav, NavAction};
use crate::gui::state::WizardState;
use crate::install_steps::ProgressEvent;

/// Returned when the GUI cannot start (no display, winit init failure).
#[derive(Debug)]
pub struct GuiUnavailable;

/// Embedded Ven logo PNG, included at compile time so the binary is
/// self-contained.
const LOGO_PNG: &[u8] = include_bytes!("../../../../assets/Ven_logo.png");

/// Launch the installer wizard. On failure to create a window, returns
/// [`GuiUnavailable`] so `main` can fall back to the CLI flow.
pub fn run(cli: SetupCli) -> std::result::Result<(), GuiUnavailable> {
    let dry_run = cli.dry_run;

    let icon = decode_icon();
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([920.0, 640.0])
        .with_min_inner_size([720.0, 540.0])
        .with_decorations(false)
        .with_title(format!("Ven Setup v{}", env!("CARGO_PKG_VERSION")));
    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Ven Setup",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(VenSetupApp::new(dry_run)))
        }),
    )
    .map_err(|e| {
        eprintln!("ven-setup: GUI failed to start: {e}");
        GuiUnavailable
    })
}

/// Decode the embedded logo into an `IconData` so the OS title bar /
/// taskbar / Alt-Tab show our brand icon instead of eframe's default
/// gear. Returns `None` if the PNG is corrupted (build-time guard
/// would have caught it; this is just the runtime fallback).
fn decode_icon() -> Option<egui::IconData> {
    let img = image::load_from_memory(LOGO_PNG).ok()?;
    let rgba = img.into_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}

struct VenSetupApp {
    state: WizardState,
}

impl VenSetupApp {
    fn new(dry_run: bool) -> Self {
        Self {
            state: WizardState::new(dry_run),
        }
    }

    /// Build the in-process `egui::TextureHandle` for the logo on the
    /// first frame. Subsequent calls are a no-op.
    fn ensure_logo(&mut self, ctx: &egui::Context) {
        if self.state.logo_texture.is_some() {
            return;
        }
        if let Ok(img) = image::load_from_memory(LOGO_PNG) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = rgba.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            let tex = ctx.load_texture("ven_logo", color_image, egui::TextureOptions::LINEAR);
            self.state.logo_texture = Some(tex);
        }
    }

    fn poll_progress(&mut self) {
        let mut events = Vec::new();
        if let Some(rx) = &self.state.progress_rx {
            while let Ok(ev) = rx.try_recv() {
                events.push(ev);
            }
        }
        for ev in events {
            let finished = matches!(
                &ev,
                ProgressEvent::InstallCompleted { .. } | ProgressEvent::InstallFailed { .. }
            );
            self.state.progress.apply_event(&ev);
            if finished {
                self.state.set_screen(Screen::Done);
                if self.state.progress.success {
                    self.state.done_message = self.state.progress.ven_version.clone();
                }
            }
        }
    }

    fn start_install(&mut self) {
        self.state.progress = state::ProgressState::new_empty();
        self.state.progress_rx = Some(worker::spawn_install(&self.state));
        self.state.set_screen(Screen::Progress);
    }

    fn show_step_rail(&self) -> bool {
        // Welcome and Done are hero layouts — they own the canvas.
        !matches!(self.state.screen, Screen::Welcome | Screen::Done)
    }

    fn show_header(&self) -> bool {
        // Hide the chrome on the hero screens for the same reason.
        !matches!(self.state.screen, Screen::Welcome | Screen::Done)
    }
}

impl eframe::App for VenSetupApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.state.window_centered {
            if let Some(cmd) = egui::ViewportCommand::center_on_screen(ctx) {
                ctx.send_viewport_cmd(cmd);
            }
            self.state.window_centered = true;
        }

        // Fade transition animation.
        let mut fading_now = self.state.fading;
        if self.state.fading {
            let dt = ctx.input(|i| i.unstable_dt);
            self.state.fade_progress = (self.state.fade_progress + dt / 0.15).min(1.0);
            if self.state.fade_progress >= 1.0 {
                self.state.fading = false;
                fading_now = false;
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        let opacity = if fading_now {
            let t = (self.state.fade_progress * 2.0 - 1.0).abs();
            t * 0.3 + 0.7
        } else {
            1.0
        };

        self.ensure_logo(ctx);
        if self.state.screen == Screen::Progress {
            self.poll_progress();
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        if self.show_header() {
            draw_header(ctx, &self.state);
        }

        if self.show_step_rail() {
            draw_step_rail(ctx, &self.state, opacity);
        }

        let nav_action = std::cell::Cell::new(NavAction::None);
        egui::TopBottomPanel::bottom("nav")
            .resizable(false)
            .min_height(64.0)
            .frame(
                egui::Frame::none()
                    .fill(theme::PANEL)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER))
                    .inner_margin(egui::Margin::symmetric(24.0, 0.0)),
            )
            .show(ctx, |ui| {
                let action = draw_nav(ui, &mut self.state, ctx);
                nav_action.set(action);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(theme::BG)
                    .inner_margin(egui::Margin::symmetric(32.0, 24.0)),
            )
            .show(ctx, |ui| {
                ui.set_opacity(opacity);
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        draw_central(ui, &mut self.state);
                    });
            });

        match nav_action.get() {
            NavAction::None => {}
            NavAction::StartInstall => self.start_install(),
            NavAction::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }
}

// ---------------------------------------------------------------------------
// Custom title bar — logo + title + drag + minimize/close.
// ---------------------------------------------------------------------------

fn draw_header(ctx: &egui::Context, state: &WizardState) {
    egui::TopBottomPanel::top("title_bar")
        .resizable(false)
        .min_height(40.0)
        .frame(
            egui::Frame::none()
                .fill(theme::PANEL)
                .inner_margin(egui::Margin::symmetric(12.0, 0.0)),
        )
        .show(ctx, |ui| {
            let height = 40.0;
            let avail = ui.available_rect_before_wrap();
            let rect = egui::Rect::from_min_size(
                egui::pos2(avail.left(), avail.top()),
                egui::vec2(avail.width(), height),
            );

            // Drag-to-move on the entire title bar.
            let drag_id = ui.next_auto_id();
            if ui
                .interact(rect, drag_id, egui::Sense::click())
                .is_pointer_button_down_on()
            {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            let painter = ui.painter();

            // Logo
            if let Some(tex) = &state.logo_texture {
                painter.image(
                    tex.id(),
                    egui::Rect::from_min_size(
                        egui::pos2(rect.left(), rect.center().y - 14.0),
                        egui::vec2(28.0, 28.0),
                    ),
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }

            // Title
            let title_pos = egui::pos2(rect.left() + 36.0, rect.center().y);
            painter.text(
                title_pos,
                egui::Align2::LEFT_CENTER,
                "Ven Setup",
                egui::FontId::new(theme::SIZE_SUBHEADING, egui::FontFamily::Proportional),
                theme::TEXT,
            );

            // Version badge (painted, not a widget)
            let ver = format!("v{}", env!("CARGO_PKG_VERSION"));
            let ver_font = egui::FontId::new(theme::SIZE_CAPTION, egui::FontFamily::Proportional);
            let ver_galley = painter.layout_no_wrap(ver, ver_font, theme::ACCENT);
            let badge_pad = egui::vec2(6.0, 2.0);
            let badge_min = egui::pos2(
                rect.left()
                    + 36.0
                    + ui.fonts(|f| {
                        f.glyph_width(
                            &egui::FontId::new(
                                theme::SIZE_SUBHEADING,
                                egui::FontFamily::Proportional,
                            ),
                            'V',
                        ) + f.glyph_width(
                            &egui::FontId::new(
                                theme::SIZE_SUBHEADING,
                                egui::FontFamily::Proportional,
                            ),
                            'e',
                        ) + f.glyph_width(
                            &egui::FontId::new(
                                theme::SIZE_SUBHEADING,
                                egui::FontFamily::Proportional,
                            ),
                            'n',
                        ) + f.glyph_width(
                            &egui::FontId::new(
                                theme::SIZE_SUBHEADING,
                                egui::FontFamily::Proportional,
                            ),
                            ' ',
                        ) + f.glyph_width(
                            &egui::FontId::new(
                                theme::SIZE_SUBHEADING,
                                egui::FontFamily::Proportional,
                            ),
                            'S',
                        ) + f.glyph_width(
                            &egui::FontId::new(
                                theme::SIZE_SUBHEADING,
                                egui::FontFamily::Proportional,
                            ),
                            'e',
                        ) + f.glyph_width(
                            &egui::FontId::new(
                                theme::SIZE_SUBHEADING,
                                egui::FontFamily::Proportional,
                            ),
                            't',
                        ) + f.glyph_width(
                            &egui::FontId::new(
                                theme::SIZE_SUBHEADING,
                                egui::FontFamily::Proportional,
                            ),
                            'u',
                        ) + f.glyph_width(
                            &egui::FontId::new(
                                theme::SIZE_SUBHEADING,
                                egui::FontFamily::Proportional,
                            ),
                            'p',
                        )
                    })
                    + 8.0,
                rect.center().y - ver_galley.size().y / 2.0,
            );
            let badge_rect =
                egui::Rect::from_min_size(badge_min, ver_galley.size() + badge_pad * 2.0);
            painter.rect(
                badge_rect,
                egui::Rounding::same(3.0),
                theme::ACCENT.linear_multiply(0.15),
                egui::Stroke::new(1.0, theme::ACCENT.linear_multiply(0.3)),
            );
            painter.galley(badge_min + badge_pad, ver_galley, theme::ACCENT);

            // Right side: step label + minimize + close buttons.
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(rect.right() - 180.0, rect.top()),
                egui::vec2(180.0, height),
            );
            ui.allocate_ui_at_rect(right_rect, |ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::close_button(ui).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if widgets::minimize_button(ui).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    ui.add_space(8.0);
                    ui.label(theme::caption(format!(
                        "Step {} of 7",
                        state.screen.index()
                    )));
                });
            });
        });
}

// ---------------------------------------------------------------------------
// Left step rail — vertical list of all wizard steps with status dots.
// ---------------------------------------------------------------------------

fn draw_step_rail(ctx: &egui::Context, state: &WizardState, opacity: f32) {
    egui::SidePanel::left("steps")
        .resizable(false)
        .exact_width(220.0)
        .frame(
            egui::Frame::none()
                .fill(theme::PANEL)
                .inner_margin(egui::Margin::symmetric(12.0, 16.0)),
        )
        .show(ctx, |ui| {
            ui.set_opacity(opacity);
            ui.add_space(4.0);
            ui.label(theme::caption("INSTALLATION"));
            ui.add_space(8.0);

            const STEPS: [(usize, Screen, &str); 7] = [
                (1, Screen::Welcome, "Welcome"),
                (2, Screen::Mode, "Install mode"),
                (3, Screen::Storage, "Storage"),
                (4, Screen::HookPath, "Shell integration"),
                (5, Screen::Runtimes, "Runtimes"),
                (6, Screen::Review, "Review"),
                (7, Screen::Progress, "Install"),
            ];

            let current = state.screen.index();
            for (idx, screen, label) in STEPS {
                let s_idx = screen.index();
                let status = if s_idx < current {
                    widgets::StepRailStatus::Done
                } else if s_idx == current {
                    widgets::StepRailStatus::Active
                } else {
                    widgets::StepRailStatus::Upcoming
                };
                widgets::step_rail_row(ui, idx, status, label);
                ui.add_space(2.0);
            }
        });
}
