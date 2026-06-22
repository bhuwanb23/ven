//! All wizard screen UIs (Welcome through Done) + the bottom navigation row.
//!
//! Every screen here is a pure render-and-respond function: state mutations
//! happen through `WizardState`, navigation through the [`NavAction`] return
//! value of `draw_nav`. The visual chrome lives in [`super::theme`] and
//! [`super::widgets`]; this file should never paint a raw `Color32`.

use eframe::egui::{self, Align, FontId, Layout, RichText, Stroke, Ui, Vec2};

use crate::common::InstallMode;
use crate::gui::state::{
    ProgressState, RuntimeOption, Screen, StepStatus, WizardState, RUNTIME_OPTIONS,
};
use crate::install_steps::{default_install_dir, default_storage_path, TOTAL_STEPS};

use super::theme;
use super::widgets;
use super::widgets::ValidKind;

// ---------------------------------------------------------------------------
// Central panel dispatcher.
// ---------------------------------------------------------------------------

/// Render the per-screen content. Called by `mod.rs::update` from inside a
/// [`egui::CentralPanel`] with a 32 px inner margin already applied.
pub fn draw_central(ui: &mut Ui, state: &mut WizardState) {
    match state.screen {
        Screen::Welcome => welcome(ui, state),
        Screen::Mode => mode(ui, state),
        Screen::Storage => storage(ui, state),
        Screen::HookPath => hook_path(ui, state),
        Screen::Runtimes => runtimes(ui, state),
        Screen::Review => review(ui, state),
        Screen::Progress => progress(ui, &state.progress),
        Screen::Done => done(ui, state),
    }
}

// ---------------------------------------------------------------------------
// Footer / navigation.
// ---------------------------------------------------------------------------

/// Outcome of the footer interaction. Mapped to side-effects in
/// `mod.rs::update`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavAction {
    None,
    StartInstall,
    Quit,
}

/// Render the cancel-confirm modal *and* the bottom button row, returning
/// whatever action the user triggered this frame.
pub fn draw_nav(ui: &mut Ui, state: &mut WizardState, ctx: &egui::Context) -> NavAction {
    let mut action = NavAction::None;

    if state.show_cancel_confirm {
        action = draw_cancel_modal(ctx, state);
        if action != NavAction::None {
            return action;
        }
    }

    ui.add_space(8.0);

    let on_progress = state.screen == Screen::Progress;
    let on_done = state.screen == Screen::Done;
    let on_review = state.screen == Screen::Review;
    let on_welcome = state.screen == Screen::Welcome;

    // Single horizontal closure so the borrow checker doesn't see the
    // two slot closures as overlapping mutable captures of `state`.
    ui.horizontal(|ui| {
        if !on_progress && !on_done {
            let has_prev = state.screen.prev().is_some();
            ui.add_enabled_ui(has_prev, |ui| {
                if widgets::text_button(ui, "Back").clicked() {
                    if let Some(prev) = state.screen.prev() {
                        state.screen = prev;
                    }
                }
            });
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if on_done {
                if widgets::primary_button(ui, "Finish").clicked() {
                    action = NavAction::Quit;
                }
                return;
            }

            if on_progress {
                let between_steps = state.progress.finished
                    || state
                        .progress
                        .steps
                        .iter()
                        .all(|s| s.status != StepStatus::Running);
                ui.add_enabled_ui(between_steps, |ui| {
                    if widgets::text_button(ui, "Cancel").clicked() {
                        state.show_cancel_confirm = true;
                    }
                });
                return;
            }

            // Standard wizard footer: Cancel (text) then primary CTA.
            // The right-to-left layout means we draw primary first.
            let next_label = if on_review {
                "Install"
            } else if on_welcome {
                "Get started"
            } else {
                "Next"
            };

            let can_next = match state.screen {
                Screen::Storage => state.can_advance_from_storage(),
                _ => true,
            };

            let mut clicked_next = false;
            ui.add_enabled_ui(can_next, |ui| {
                if widgets::primary_button(ui, next_label).clicked() {
                    clicked_next = true;
                }
            });

            if clicked_next {
                if state.screen == Screen::Storage {
                    state.validate_storage();
                    if !state.can_advance_from_storage() {
                        return;
                    }
                }
                if on_review {
                    action = NavAction::StartInstall;
                } else if let Some(next) = state.screen.next() {
                    state.screen = next;
                }
            }

            if !on_welcome && widgets::text_button(ui, "Cancel").clicked() {
                state.show_cancel_confirm = true;
            }
        });
    });
    ui.add_space(4.0);

    action
}

fn draw_cancel_modal(ctx: &egui::Context, state: &mut WizardState) -> NavAction {
    let mut action = NavAction::None;
    egui::Window::new("Cancel setup?")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(theme::CARD)
                .stroke(Stroke::new(1.0, theme::BORDER))
                .rounding(theme::RADIUS_CARD)
                .inner_margin(20.0),
        )
        .show(ctx, |ui| {
            ui.set_min_width(360.0);
            ui.label(theme::subheading("Cancel setup?"));
            ui.add_space(8.0);
            ui.label(theme::muted_body(
                "Exit the installer without making changes? Any partial files will be cleaned up.",
            ));
            ui.add_space(16.0);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if widgets::destructive_button(ui, "Exit").clicked() {
                    action = NavAction::Quit;
                }
                if widgets::primary_button(ui, "Stay").clicked() {
                    state.show_cancel_confirm = false;
                }
            });
        });
    action
}

// ---------------------------------------------------------------------------
// Welcome — hero layout with logo, headline, subtitle, primary CTA hint.
// (The primary "Get started" button lives in the footer, not the body, so
// the user always knows where to click next.)
// ---------------------------------------------------------------------------

fn welcome(ui: &mut Ui, state: &mut WizardState) {
    ui.add_space(48.0);
    widgets::hero_logo(ui, state.logo_texture.as_ref(), 96.0);
    ui.add_space(20.0);
    ui.vertical_centered(|ui| {
        ui.label(theme::display("Welcome to Ven"));
        ui.add_space(8.0);
        ui.label(theme::muted_body(format!(
            "Setup wizard v{}",
            env!("CARGO_PKG_VERSION")
        )));
        ui.add_space(20.0);
        ui.label(
            theme::body(
                "This wizard installs ven and ven-launcher, configures your\nstorage root, and optionally pre-installs language runtimes.",
            )
            .color(theme::MUTED),
        );
        ui.add_space(28.0);

        // Three-column "what you'll set up" hint row.
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 540.0).max(0.0) * 0.5);
            for (icon, label) in [
                ("◆", "Install ven"),
                ("◇", "Pick storage root"),
                ("◈", "Pre-install runtimes"),
            ] {
                ui.allocate_ui(Vec2::new(180.0, 48.0), |ui| {
                    widgets::card(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(icon)
                                    .size(20.0)
                                    .color(theme::ACCENT),
                            );
                            ui.label(theme::caption(label));
                        });
                    });
                });
            }
        });

        // Existing-install detection banner.
        if !state.existing_installs.is_empty() {
            ui.add_space(24.0);
            widgets::card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚠").size(20.0).color(theme::WARNING));
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(theme::body("Existing ven installation detected"));
                        for inst in &state.existing_installs {
                            let mode_label = match inst.mode {
                                InstallMode::User => "user",
                                InstallMode::System => "system",
                            };
                            ui.label(theme::caption(format!(
                                "ven {} at {} ({})",
                                inst.version,
                                inst.install_dir.display(),
                                mode_label,
                            )));
                        }
                        ui.add_space(4.0);
                        ui.label(
                            theme::caption("This wizard will replace the existing binary with the new version.")
                                .color(theme::MUTED),
                        );
                    });
                });
            });
        }

        ui.add_space(16.0);
        if ui
            .link(
                RichText::new("Licensed under MIT — view documentation")
                    .size(theme::SIZE_CAPTION)
                    .color(theme::ACCENT),
            )
            .clicked()
        {
            let _ = webbrowser::open("https://github.com/bhuwanb23/ven");
        }
    });
}

// ---------------------------------------------------------------------------
// Mode — 2 option_cards (User / System).
// ---------------------------------------------------------------------------

fn mode(ui: &mut Ui, state: &mut WizardState) {
    widgets::section_heading(
        ui,
        "Choose install mode",
        Some("Who should be able to use this ven install?"),
    );

    if widgets::option_card(
        ui,
        matches!(state.install_mode, InstallMode::User),
        "User install (recommended)",
        "Installs to your home directory. No administrator or sudo required.",
        Some("No admin needed · ~/.ven on Unix · %USERPROFILE%\\.ven on Windows"),
    )
    .clicked()
    {
        state.install_mode = InstallMode::User;
    }
    ui.add_space(12.0);
    if widgets::option_card(
        ui,
        matches!(state.install_mode, InstallMode::System),
        "System install",
        "Installs for all users on this machine.",
        Some("Requires UAC on Windows · sudo on Unix · /usr/local or C:\\Program Files"),
    )
    .clicked()
    {
        state.install_mode = InstallMode::System;
    }

    if matches!(state.install_mode, InstallMode::System) {
        ui.add_space(theme::SECTION_GAP);
        widgets::validation_line(
            ui,
            ValidKind::Warn,
            "On Windows you will see a UAC prompt; on Linux/macOS you may need to re-run with sudo.",
        );
    }
}

// ---------------------------------------------------------------------------
// Storage — path input + Browse + Use default + validation_line.
// ---------------------------------------------------------------------------

fn storage(ui: &mut Ui, state: &mut WizardState) {
    widgets::section_heading(
        ui,
        "Storage location",
        Some("Where should ven store runtimes, cache, and project data ($VEN_HOME)?"),
    );

    widgets::card(ui, |ui| {
        let mut path_str = state.storage_path.display().to_string();
        ui.label(theme::caption("VEN_HOME"));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            // Wider text input than stock — feels less like a form field
            // and more like a desktop installer.
            let edit = egui::TextEdit::singleline(&mut path_str)
                .desired_width(ui.available_width() - 200.0)
                .font(FontId::new(theme::SIZE_BODY, egui::FontFamily::Monospace));
            if ui.add(edit).changed() {
                state.storage_path = std::path::PathBuf::from(path_str.as_str());
                state.validate_storage();
            }
            if widgets::secondary_button(ui, "Browse…").clicked() {
                state.pending_browse = true;
            }
        });

        if state.pending_browse {
            state.pending_browse = false;
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                state.storage_path = folder;
                state.validate_storage();
            }
        }

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if widgets::text_button(ui, "Use default").clicked() {
                state.storage_path = default_storage_path();
                state.validate_storage();
            }
            ui.add_space(8.0);
            ui.label(theme::caption(format!(
                "Default: {}",
                default_storage_path().display()
            )));
        });

        ui.add_space(12.0);
        if let Some(err) = &state.storage_error {
            widgets::validation_line(ui, ValidKind::Error, err);
        } else {
            widgets::validation_line(ui, ValidKind::Ok, "Path is writable.");
        }
    });

    ui.add_space(theme::SECTION_GAP);
    ui.label(
        theme::caption(
            "Tip: pick a directory on a fast drive — runtimes download here. The wizard will create it for you if it doesn't exist.",
        ),
    );
}

// ---------------------------------------------------------------------------
// Hook / PATH — 2 option_card toggles.
// ---------------------------------------------------------------------------

fn hook_path(ui: &mut Ui, state: &mut WizardState) {
    widgets::section_heading(
        ui,
        "Shell integration",
        Some("Choose what ven wires up in your shells. Both are recommended."),
    );

    if widgets::option_card(
        ui,
        state.add_to_path,
        "Add ven to PATH",
        "Lets you run `ven` from any terminal without a full path.",
        Some("Edits HKCU\\Environment\\Path on Windows · ~/.ven/env on Unix"),
    )
    .clicked()
    {
        state.add_to_path = !state.add_to_path;
    }
    ui.add_space(12.0);
    if widgets::option_card(
        ui,
        state.install_hook,
        "Install shell hook for auto-activation",
        "Automatically applies ven.toml when you cd into a project.",
        Some("Supports PowerShell, bash, zsh, fish · idempotent · removable with `ven uninstall`"),
    )
    .clicked()
    {
        state.install_hook = !state.install_hook;
    }

    if !state.add_to_path {
        ui.add_space(theme::SECTION_GAP);
        widgets::validation_line(
            ui,
            ValidKind::Warn,
            "Without PATH, you'll have to invoke ven by full path until you add it manually.",
        );
    }
}

// ---------------------------------------------------------------------------
// Runtimes — 8 option_cards in a 2-column grid.
// ---------------------------------------------------------------------------

fn runtimes(ui: &mut Ui, state: &mut WizardState) {
    widgets::section_heading(
        ui,
        "Pre-install runtimes (optional)",
        Some(
            "Select languages to install with `ven install <lang> latest` after the main install.",
        ),
    );

    let avail = ui.available_width();
    let col_w = ((avail - 16.0) * 0.5).max(280.0);
    egui::Grid::new("runtimes_grid")
        .num_columns(2)
        .spacing([16.0, 12.0])
        .show(ui, |ui| {
            for (i, rt) in RUNTIME_OPTIONS.iter().enumerate() {
                ui.allocate_ui(Vec2::new(col_w, 96.0), |ui| {
                    runtime_card(ui, rt, &mut state.runtime_selected[i]);
                });
                if i % 2 == 1 {
                    ui.end_row();
                }
            }
        });

    let count = state.runtime_selected.iter().filter(|x| **x).count();
    ui.add_space(theme::SECTION_GAP);
    if count == 0 {
        widgets::validation_line(
            ui,
            ValidKind::Warn,
            "No runtimes selected — that's fine, you can install them later with `ven install`.",
        );
    } else {
        widgets::validation_line(
            ui,
            ValidKind::Ok,
            &format!(
                "{count} runtime{} will be installed after the wizard finishes.",
                if count == 1 { "" } else { "s" }
            ),
        );
    }
}

fn runtime_card(ui: &mut Ui, rt: &RuntimeOption, selected: &mut bool) {
    let response = widgets::option_card(
        ui,
        *selected,
        rt.label,
        rt.description,
        Some(&format!("ven install {} latest", rt.slug)),
    );
    if response.clicked() {
        *selected = !*selected;
    }
}

// ---------------------------------------------------------------------------
// Review — 6-row summary table inside a single card + dry-run banner.
// ---------------------------------------------------------------------------

fn review(ui: &mut Ui, state: &WizardState) {
    widgets::section_heading(
        ui,
        "Review your choices",
        Some("Last chance to change anything. The Install button kicks off the worker."),
    );

    let cfg_mode = state.install_mode;
    let install_dir = default_install_dir(cfg_mode);
    let mode_label = match cfg_mode {
        InstallMode::User => "User (per-account)",
        InstallMode::System => "System (all users)",
    };

    let runtimes_value = {
        let r = state.selected_runtimes();
        if r.is_empty() {
            "None".to_string()
        } else {
            r.join(", ")
        }
    };

    widgets::card(ui, |ui| {
        widgets::summary_row(ui, "Install mode", mode_label);

        // Show upgrade info when existing install is detected.
        if let Some(existing) = state.existing_installs.first() {
            widgets::summary_row(ui, "Upgrade from", &format!("ven {}", existing.version));
        }

        widgets::summary_row(ui, "Binaries", &install_dir.display().to_string());
        widgets::summary_row(
            ui,
            "Storage ($VEN_HOME)",
            &state.storage_path.display().to_string(),
        );
        widgets::summary_row(
            ui,
            "Add to PATH",
            if state.add_to_path { "Yes" } else { "No" },
        );
        widgets::summary_row(
            ui,
            "Shell hook",
            if state.install_hook { "Yes" } else { "No" },
        );
        widgets::summary_row(ui, "Pre-install runtimes", &runtimes_value);
    });

    if state.dry_run {
        ui.add_space(theme::SECTION_GAP);
        widgets::validation_line(
            ui,
            ValidKind::Warn,
            "Dry-run mode: no files will be modified.",
        );
    }
}

// ---------------------------------------------------------------------------
// Progress — big progress bar + 6 step rows + JetBrains-Mono log tail.
// ---------------------------------------------------------------------------

fn progress(ui: &mut Ui, progress: &ProgressState) {
    widgets::section_heading(
        ui,
        "Installing ven…",
        Some("This usually takes 30–90 seconds. Keep this window open."),
    );

    // Big progress bar with percent overlay painted in the middle.
    let bar_rect = widgets::big_progress_bar(ui, progress.overall_percent, 24.0);
    ui.painter().text(
        bar_rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{:.0}%", progress.overall_percent * 100.0),
        FontId::new(theme::SIZE_CAPTION, egui::FontFamily::Proportional),
        theme::TEXT,
    );
    ui.add_space(theme::SECTION_GAP);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (i, step) in progress.steps.iter().enumerate() {
                progress_row(
                    ui,
                    i + 1,
                    step.status.clone(),
                    &step.label,
                    &step.detail,
                    &step.log_lines,
                );
                ui.add_space(8.0);
            }
        });

    if progress.finished && !progress.success {
        if let Some(err) = &progress.error {
            ui.add_space(theme::SECTION_GAP);
            widgets::validation_line(ui, ValidKind::Error, err);
        }
    }
}

fn progress_row(
    ui: &mut Ui,
    idx: usize,
    status: StepStatus,
    label: &str,
    detail: &str,
    log_lines: &[String],
) {
    widgets::card(ui, |ui| {
        ui.horizontal(|ui| {
            // Status icon column — fixed 32 px so labels align.
            ui.allocate_ui(Vec2::new(32.0, 24.0), |ui| {
                draw_step_icon(ui, &status);
            });
            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(theme::body(format!("{idx}/{TOTAL_STEPS}")).color(theme::MUTED));
                    ui.add_space(6.0);
                    ui.label(theme::subheading(label));
                });
                if !detail.is_empty() {
                    ui.add_space(2.0);
                    ui.label(theme::muted_body(detail));
                }
                if !log_lines.is_empty() {
                    ui.add_space(4.0);
                    for line in log_lines.iter().rev().take(5).rev() {
                        ui.label(
                            RichText::new(line)
                                .size(theme::SIZE_CAPTION)
                                .family(egui::FontFamily::Monospace)
                                .color(theme::MUTED),
                        );
                    }
                }
            });
        });
    });
}

fn draw_step_icon(ui: &mut Ui, status: &StepStatus) {
    let center = ui.cursor().min + Vec2::new(12.0, 12.0);
    let painter = ui.painter();
    match status {
        StepStatus::Pending => {
            painter.circle_stroke(center, 9.0, Stroke::new(1.5, theme::BORDER));
        }
        StepStatus::Running => {
            // Real spinner — paint over the same 18×18 cell.
            ui.allocate_ui(Vec2::splat(24.0), |ui| {
                ui.add(egui::Spinner::new().size(18.0).color(theme::ACCENT));
            });
            return;
        }
        StepStatus::Done => {
            painter.circle_filled(center, 9.0, theme::SUCCESS);
            paint_check_glyph(painter, center, 4.5, egui::Color32::BLACK);
        }
        StepStatus::Skipped => {
            painter.circle_filled(center, 9.0, theme::MUTED);
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                "—",
                FontId::new(theme::SIZE_CAPTION, egui::FontFamily::Proportional),
                egui::Color32::BLACK,
            );
        }
        StepStatus::Failed => {
            painter.circle_filled(center, 9.0, theme::ERROR);
            paint_x_glyph(painter, center, 4.5, egui::Color32::WHITE);
        }
    }
    ui.allocate_exact_size(Vec2::splat(24.0), egui::Sense::hover());
}

fn paint_check_glyph(
    painter: &egui::Painter,
    center: egui::Pos2,
    scale: f32,
    color: egui::Color32,
) {
    let p1 = center + Vec2::new(-scale * 0.7, scale * 0.1);
    let p2 = center + Vec2::new(-scale * 0.2, scale * 0.6);
    let p3 = center + Vec2::new(scale * 0.7, -scale * 0.5);
    painter.line_segment([p1, p2], Stroke::new(1.5, color));
    painter.line_segment([p2, p3], Stroke::new(1.5, color));
}

fn paint_x_glyph(painter: &egui::Painter, center: egui::Pos2, scale: f32, color: egui::Color32) {
    let p1a = center + Vec2::new(-scale, -scale);
    let p1b = center + Vec2::new(scale, scale);
    let p2a = center + Vec2::new(-scale, scale);
    let p2b = center + Vec2::new(scale, -scale);
    painter.line_segment([p1a, p1b], Stroke::new(1.5, color));
    painter.line_segment([p2a, p2b], Stroke::new(1.5, color));
}

// ---------------------------------------------------------------------------
// Done — hero layout with success / failure check_circle, headline,
// version line, two big buttons.
// ---------------------------------------------------------------------------

fn done(ui: &mut Ui, state: &WizardState) {
    let success = state.progress.success;
    ui.add_space(48.0);

    ui.vertical_centered(|ui| {
        widgets::check_circle(ui, success, 96.0);
        ui.add_space(20.0);
        if success {
            ui.label(theme::display("Installation complete"));
        } else {
            ui.label(theme::display("Installation incomplete"));
        }
        ui.add_space(8.0);
        if let Some(msg) = &state.done_message {
            ui.label(theme::muted_body(msg));
        } else if let Some(v) = &state.progress.ven_version {
            ui.label(theme::body(v));
        } else if !success {
            if let Some(err) = &state.progress.error {
                ui.label(theme::muted_body(err));
            }
        }
        ui.add_space(28.0);

        ui.horizontal(|ui| {
            // Center the two-button row by padding the leading horizontal
            // space with the spare width.
            let row_w = 360.0;
            let pad = ((ui.available_width() - row_w) * 0.5).max(0.0);
            ui.add_space(pad);
            if widgets::secondary_button(ui, "Open documentation").clicked() {
                let _ = webbrowser::open("https://github.com/bhuwanb23/ven");
            }
            ui.add_space(12.0);
            if widgets::primary_button_sized(ui, "Open new terminal", Vec2::new(180.0, 36.0))
                .clicked()
            {
                open_terminal();
            }
        });

        ui.add_space(16.0);
        ui.label(theme::caption(
            "Open a new terminal and run `ven --version` to confirm. If it still shows an old version, run `ven doctor` — another install may be winning on PATH.",
        ));
    });
}

fn open_terminal() {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("cmd.exe")
            .args(["/C", "start", "cmd"])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .args(["-a", "Terminal"])
            .spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for term in [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
        ] {
            if std::process::Command::new(term).spawn().is_ok() {
                break;
            }
        }
    }
}
