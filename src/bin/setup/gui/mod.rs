//! Native GUI wizard for `ven-setup` (eframe / egui).

mod screens;
mod state;
mod worker;

pub use state::Screen;

use eframe::egui;

use crate::common::SetupCli;
use crate::gui::screens::{apply_theme, draw_nav, draw_screen, NavAction};
use crate::gui::state::WizardState;
use crate::install_steps::ProgressEvent;

/// Returned when the GUI cannot start (no display, winit init failure).
#[derive(Debug)]
pub struct GuiUnavailable;

/// Launch the installer wizard. On failure to create a window, returns
/// [`GuiUnavailable`] so `main` can fall back to the CLI flow.
pub fn run(cli: SetupCli) -> std::result::Result<(), GuiUnavailable> {
    let dry_run = cli.dry_run;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 520.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title(format!("Ven Setup {}", env!("CARGO_PKG_VERSION"))),
        ..Default::default()
    };

    eframe::run_native(
        "Ven Setup",
        options,
        Box::new(move |cc| {
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(VenSetupApp::new(dry_run)))
        }),
    )
    .map_err(|e| {
        eprintln!("ven-setup: GUI failed to start: {e}");
        GuiUnavailable
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

    fn ensure_logo(&mut self, ctx: &egui::Context) {
        if self.state.logo_texture.is_some() {
            return;
        }
        const LOGO: &[u8] = include_bytes!("../../../../assets/Ven_logo.png");
        if let Ok(img) = image::load_from_memory(LOGO) {
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
                self.state.screen = Screen::Done;
                if self.state.progress.success {
                    self.state.done_message = self.state.progress.ven_version.clone();
                }
            }
        }
    }

    fn start_install(&mut self) {
        self.state.progress = state::ProgressState::new_empty();
        self.state.progress_rx = Some(worker::spawn_install(&self.state));
        self.state.screen = Screen::Progress;
    }
}

impl eframe::App for VenSetupApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_logo(ctx);
        if self.state.screen == Screen::Progress {
            self.poll_progress();
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("Ven Setup")
                        .strong()
                        .color(egui::Color32::from_rgb(99, 102, 241)),
                );
                ui.label(
                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                        .small()
                        .color(egui::Color32::GRAY),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.state.screen != Screen::Welcome && self.state.screen != Screen::Done {
                        let step = self.state.screen.index();
                        ui.label(format!("Step {}/7", step));
                    }
                });
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(8.0);
            draw_screen(ui, &mut self.state);
        });

        egui::TopBottomPanel::bottom("nav").show(ctx, |ui| {
            match draw_nav(ui, &mut self.state, ctx) {
                NavAction::None => {}
                NavAction::StartInstall => self.start_install(),
                NavAction::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            }
        });
    }
}
