//! All wizard screen UIs (Welcome through Done).

use egui::{Color32, RichText, Ui, Vec2};

use crate::common::InstallMode;
use crate::install_steps::{default_install_dir, default_storage_path, TOTAL_STEPS};
use crate::gui::state::{ProgressState, RuntimeOption, Screen, StepStatus, WizardState, RUNTIME_OPTIONS};

// Brand palette (aligned with ven website dark theme).
const ACCENT: Color32 = Color32::from_rgb(99, 102, 241);
const ACCENT_DIM: Color32 = Color32::from_rgb(67, 56, 202);
const SURFACE: Color32 = Color32::from_rgb(24, 24, 27);
const CARD: Color32 = Color32::from_rgb(39, 39, 42);
const TEXT: Color32 = Color32::from_rgb(250, 250, 250);
const MUTED: Color32 = Color32::from_rgb(161, 161, 170);
const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);
const ERROR: Color32 = Color32::from_rgb(239, 68, 68);

pub fn draw_screen(ui: &mut Ui, state: &mut WizardState) {
    ui.visuals_mut().widgets.noninteractive.fg_stroke.color = TEXT;
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

pub fn draw_nav(ui: &mut Ui, state: &mut WizardState, ctx: &egui::Context) -> NavAction {
    let mut action = NavAction::None;
    if state.show_cancel_confirm {
        egui::Window::new("Cancel setup?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Exit the installer without making changes?");
                ui.horizontal(|ui| {
                    if ui.button("Stay").clicked() {
                        state.show_cancel_confirm = false;
                    }
                    if ui.button(RichText::new("Exit").color(ERROR)).clicked() {
                        action = NavAction::Quit;
                    }
                });
            });
        return action;
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let on_progress = state.screen == Screen::Progress;
        let on_done = state.screen == Screen::Done;
        let on_review = state.screen == Screen::Review;

        if !on_progress && !on_done {
            if ui
                .add_enabled(state.screen.prev().is_some(), egui::Button::new("Back"))
                .clicked()
            {
                if let Some(prev) = state.screen.prev() {
                    state.screen = prev;
                }
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if on_done {
                if ui.button(RichText::new("Finish").strong()).clicked() {
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
                if ui
                    .add_enabled(between_steps, egui::Button::new("Cancel"))
                    .clicked()
                {
                    state.show_cancel_confirm = true;
                }
                return;
            }

            if ui.button(RichText::new("Cancel").color(MUTED)).clicked() {
                state.show_cancel_confirm = true;
            }

            let next_label = if on_review { "Install" } else { "Next" };
            let can_next = match state.screen {
                Screen::Storage => state.can_advance_from_storage(),
                _ => true,
            };
            if ui
                .add_enabled(can_next, egui::Button::new(RichText::new(next_label).strong()))
                .clicked()
            {
                if state.screen == Screen::Storage {
                    state.validate_storage();
                    if !state.can_advance_from_storage() {
                        return action;
                    }
                }
                action = if on_review {
                    NavAction::StartInstall
                } else if let Some(next) = state.screen.next() {
                    state.screen = next;
                    NavAction::None
                } else {
                    NavAction::None
                };
            }
        });
    });
    action
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavAction {
    None,
    StartInstall,
    Quit,
}

fn welcome(ui: &mut Ui, state: &mut WizardState) {
    ui.vertical_centered(|ui| {
        ui.add_space(24.0);
        if let Some(tex) = &state.logo_texture {
            ui.image((tex.id(), Vec2::new(96.0, 96.0)));
        } else {
            ui.heading(RichText::new("ven").size(48.0).color(ACCENT));
        }
        ui.add_space(12.0);
        ui.heading(RichText::new("Welcome to Ven Setup").color(TEXT).size(24.0));
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                .color(MUTED)
                .size(14.0),
        );
        ui.add_space(16.0);
        ui.label(
            RichText::new(
                "This wizard installs ven and ven-launcher, configures your storage root, \
                 and optionally pre-installs language runtimes.",
            )
            .color(MUTED),
        );
        ui.add_space(12.0);
        if ui
            .link(RichText::new("Licensed under MIT — ven documentation").color(ACCENT))
            .clicked()
        {
            let _ = webbrowser::open("https://github.com/bhuwanb23/ven");
        }
    });
}

fn mode(ui: &mut Ui, state: &mut WizardState) {
    ui.heading("Choose install mode");
    ui.add_space(8.0);
    ui.label(RichText::new("Who should be able to use this ven install?").color(MUTED));
    ui.add_space(12.0);

    card(ui, |ui| {
        let user_selected = matches!(state.install_mode, InstallMode::User);
        if ui
            .radio(user_selected, RichText::new("User install (recommended)").strong())
            .clicked()
        {
            state.install_mode = InstallMode::User;
        }
        ui.label(
            RichText::new("Installs to your home directory. No administrator or sudo required.")
                .color(MUTED)
                .small(),
        );
        ui.add_space(12.0);
        let sys_selected = matches!(state.install_mode, InstallMode::System);
        if ui
            .radio(sys_selected, RichText::new("System install").strong())
            .clicked()
        {
            state.install_mode = InstallMode::System;
        }
        ui.label(
            RichText::new(
                "Installs for all users on this machine. Requires administrator (Windows UAC) or sudo (Unix).",
            )
            .color(MUTED)
            .small(),
        );
    });

    if matches!(state.install_mode, InstallMode::System) {
        ui.add_space(12.0);
        ui.colored_label(
            ACCENT,
            "Note: On Windows you will see a UAC prompt. On Linux/macOS you may need to re-run with sudo.",
        );
    }
}

fn storage(ui: &mut Ui, state: &mut WizardState) {
    ui.heading("Storage location");
    ui.add_space(8.0);
    ui.label(
        RichText::new("Where should ven store runtimes, cache, and project data ($VEN_HOME)?")
            .color(MUTED),
    );
    ui.add_space(12.0);

    card(ui, |ui| {
        let mut path_str = state.storage_path.display().to_string();
        ui.horizontal(|ui| {
            ui.label("Path:");
            if ui
                .add(egui::TextEdit::singleline(&mut path_str).desired_width(360.0))
                .changed()
            {
                state.storage_path = path_str.into();
                state.validate_storage();
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Browse…").clicked() {
                state.pending_browse = true;
            }
            if ui.button("Use default").clicked() {
                state.storage_path = default_storage_path();
                state.validate_storage();
                path_str = state.storage_path.display().to_string();
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
        ui.label(
            RichText::new(format!("Default: {}", default_storage_path().display()))
                .small()
                .color(MUTED),
        );

        if let Some(err) = &state.storage_error {
            ui.colored_label(ERROR, err);
        } else {
            ui.colored_label(SUCCESS, "Path is writable");
        }
    });
}

fn hook_path(ui: &mut Ui, state: &mut WizardState) {
    ui.heading("Shell integration");
    ui.add_space(8.0);
    card(ui, |ui| {
        ui.checkbox(
            &mut state.add_to_path,
            RichText::new("Add ven to PATH").strong(),
        );
        ui.label(
            RichText::new(
                "Lets you run `ven` from any terminal without a full path. Recommended.",
            )
            .color(MUTED)
            .small(),
        );
        ui.add_space(12.0);
        ui.checkbox(
            &mut state.install_hook,
            RichText::new("Install shell hook for auto-activation").strong(),
        );
        ui.label(
            RichText::new(
                "Automatically applies ven.toml when you cd into a project (PowerShell, bash, zsh, fish).",
            )
            .color(MUTED)
            .small(),
        );
    });
}

fn runtimes(ui: &mut Ui, state: &mut WizardState) {
    ui.heading("Pre-install runtimes (optional)");
    ui.add_space(4.0);
    ui.label(
        RichText::new("Select languages to install with `ven install <lang> latest` after setup.")
            .color(MUTED),
    );
    ui.add_space(8.0);

    egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
        egui::Grid::new("runtimes_grid")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                for (i, rt) in RUNTIME_OPTIONS.iter().enumerate() {
                    runtime_card(ui, rt, &mut state.runtime_selected[i]);
                }
            });
    });
}

fn runtime_card(ui: &mut Ui, rt: &RuntimeOption, selected: &mut bool) {
    let frame = egui::Frame::none()
        .fill(CARD)
        .inner_margin(12.0)
        .rounding(8.0);
    frame.show(ui, |ui| {
        ui.set_min_width(200.0);
        ui.checkbox(selected, RichText::new(rt.label).strong());
        ui.label(RichText::new(rt.description).small().color(MUTED));
        ui.label(
            RichText::new(format!("ven install {} latest", rt.slug))
                .small()
                .color(ACCENT_DIM),
        );
    });
}

fn review(ui: &mut Ui, state: &WizardState) {
    ui.heading("Review your choices");
    ui.add_space(8.0);
    let cfg_mode = state.install_mode;
    let install_dir = default_install_dir(cfg_mode);

    card(ui, |ui| {
        let mode_label = match cfg_mode {
            InstallMode::User => "User (per-account)",
            InstallMode::System => "System (all users)",
        };
        summary_row(ui, "Install mode", mode_label.into());
        summary_row(ui, "Binaries", install_dir.display().to_string());
        summary_row(ui, "Storage ($VEN_HOME)", state.storage_path.display().to_string());
        summary_row(
            ui,
            "PATH",
            if state.add_to_path { "Yes" } else { "No" },
        );
        summary_row(
            ui,
            "Shell hook",
            if state.install_hook { "Yes" } else { "No" },
        );
        let runtimes = state.selected_runtimes();
        summary_row(
            ui,
            "Pre-install",
            if runtimes.is_empty() {
                "None".into()
            } else {
                runtimes.join(", ")
            },
        );
        if state.dry_run {
            ui.colored_label(ACCENT, "Dry-run: no files will be modified.");
        }
    });
}

fn summary_row(ui: &mut Ui, label: &str, value: String) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(MUTED));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(value).strong());
        });
    });
    ui.add_space(4.0);
}

fn progress(ui: &mut Ui, progress: &ProgressState) {
    ui.heading("Installing ven…");
    ui.add_space(8.0);
    ui.add(
        egui::ProgressBar::new(progress.overall_percent)
            .text(format!("{:.0}%", progress.overall_percent * 100.0)),
    );
    ui.add_space(12.0);

    egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
        for (i, step) in progress.steps.iter().enumerate() {
            let icon = match step.status {
                StepStatus::Pending => "○",
                StepStatus::Running => "◐",
                StepStatus::Done => "✓",
                StepStatus::Skipped => "—",
                StepStatus::Failed => "✗",
            };
            let color = match step.status {
                StepStatus::Done => SUCCESS,
                StepStatus::Failed => ERROR,
                StepStatus::Running => ACCENT,
                _ => MUTED,
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).color(color));
                ui.vertical(|ui| {
                    ui.label(RichText::new(format!("{}/{} {}", i + 1, TOTAL_STEPS, step.label)));
                    if !step.detail.is_empty() {
                        ui.label(RichText::new(&step.detail).small().color(MUTED));
                    }
                    for line in step.log_lines.iter().rev().take(5).rev() {
                        ui.label(RichText::new(line).small().monospace().color(MUTED));
                    }
                });
            });
            ui.add_space(6.0);
        }
    });

    if progress.finished && !progress.success {
        if let Some(err) = &progress.error {
            ui.colored_label(ERROR, err);
        }
    }
}

fn done(ui: &mut Ui, state: &WizardState) {
    let success = state.progress.success;
    ui.vertical_centered(|ui| {
        ui.add_space(24.0);
        if success {
            ui.heading(RichText::new("Installation complete").color(SUCCESS).size(22.0));
        } else {
            ui.heading(RichText::new("Installation incomplete").color(ERROR).size(22.0));
        }
        ui.add_space(12.0);
        if let Some(msg) = &state.done_message {
            ui.label(msg);
        } else if let Some(v) = &state.progress.ven_version {
            ui.label(RichText::new(v).strong());
        }
        ui.add_space(16.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Open documentation").clicked() {
                let _ = webbrowser::open("https://github.com/bhuwanb23/ven");
            }
            if ui.button("Open new terminal").clicked() {
                open_terminal();
            }
        });
        ui.add_space(8.0);
        ui.label(
            RichText::new("Open a new terminal and run: ven --version")
                .color(MUTED)
                .small(),
        );
    });
}

fn card(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::none()
        .fill(CARD)
        .inner_margin(16.0)
        .rounding(8.0)
        .show(ui, add_contents);
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
        for term in ["x-terminal-emulator", "gnome-terminal", "konsole", "xfce4-terminal"]
        {
            if std::process::Command::new(term).spawn().is_ok() {
                break;
            }
        }
    }
}

pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = SURFACE;
    visuals.window_fill = SURFACE;
    visuals.extreme_bg_color = SURFACE;
    visuals.faint_bg_color = CARD;
    visuals.widgets.noninteractive.bg_fill = CARD;
    visuals.widgets.inactive.bg_fill = CARD;
    visuals.widgets.hovered.bg_fill = ACCENT_DIM;
    visuals.widgets.active.bg_fill = ACCENT;
    visuals.selection.bg_fill = ACCENT;
    ctx.set_visuals(visuals);
}
