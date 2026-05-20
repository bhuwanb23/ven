//! Wizard state shared across all installer screens.

use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use crate::common::InstallMode;
use crate::install_steps::{default_storage_path, ProgressEvent, TOTAL_STEPS};

/// Installer wizard screens (in navigation order).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Mode,
    Storage,
    HookPath,
    Runtimes,
    Review,
    Progress,
    Done,
}

impl Screen {
    pub fn index(self) -> usize {
        match self {
            Screen::Welcome => 0,
            Screen::Mode => 1,
            Screen::Storage => 2,
            Screen::HookPath => 3,
            Screen::Runtimes => 4,
            Screen::Review => 5,
            Screen::Progress => 6,
            Screen::Done => 7,
        }
    }

    /// Human-readable title for the screen — surfaced by the window title
    /// bar on platforms that show one. Currently unused (the wizard window
    /// keeps a fixed "ven setup" title), kept around for future per-screen
    /// titles in the platform window decoration.
    #[allow(dead_code)]
    pub fn title(self) -> &'static str {
        match self {
            Screen::Welcome => "Welcome",
            Screen::Mode => "Install mode",
            Screen::Storage => "Storage location",
            Screen::HookPath => "Shell integration",
            Screen::Runtimes => "Runtimes",
            Screen::Review => "Review",
            Screen::Progress => "Installing",
            Screen::Done => "Complete",
        }
    }

    pub fn next(self) -> Option<Screen> {
        match self {
            Screen::Welcome => Some(Screen::Mode),
            Screen::Mode => Some(Screen::Storage),
            Screen::Storage => Some(Screen::HookPath),
            Screen::HookPath => Some(Screen::Runtimes),
            Screen::Runtimes => Some(Screen::Review),
            Screen::Review => Some(Screen::Progress),
            Screen::Progress => Some(Screen::Done),
            Screen::Done => None,
        }
    }

    pub fn prev(self) -> Option<Screen> {
        match self {
            Screen::Welcome => None,
            Screen::Mode => Some(Screen::Welcome),
            Screen::Storage => Some(Screen::Mode),
            Screen::HookPath => Some(Screen::Storage),
            Screen::Runtimes => Some(Screen::HookPath),
            Screen::Review => Some(Screen::Runtimes),
            Screen::Progress => None,
            Screen::Done => None,
        }
    }
}

/// One of the eight pre-installable language runtimes.
#[derive(Clone, Copy, Debug)]
pub struct RuntimeOption {
    pub slug: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

pub const RUNTIME_OPTIONS: [RuntimeOption; 8] = [
    RuntimeOption {
        slug: "node",
        label: "Node.js",
        description: "JavaScript runtime",
    },
    RuntimeOption {
        slug: "python",
        label: "Python",
        description: "CPython interpreter",
    },
    RuntimeOption {
        slug: "go",
        label: "Go",
        description: "Go toolchain",
    },
    RuntimeOption {
        slug: "rust",
        label: "Rust",
        description: "rustc + cargo via rustup",
    },
    RuntimeOption {
        slug: "java",
        label: "Java",
        description: "OpenJDK distribution",
    },
    RuntimeOption {
        slug: "deno",
        label: "Deno",
        description: "Secure TypeScript runtime",
    },
    RuntimeOption {
        slug: "bun",
        label: "Bun",
        description: "All-in-one JS toolkit",
    },
    RuntimeOption {
        slug: "ruby",
        label: "Ruby",
        description: "Ruby interpreter",
    },
];

/// Per-step status on the Progress screen.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum StepStatus {
    #[default]
    Pending,
    Running,
    Done,
    Skipped,
    Failed,
}

#[derive(Clone, Debug, Default)]
pub struct StepView {
    pub label: String,
    pub detail: String,
    pub status: StepStatus,
    pub log_lines: Vec<String>,
}

/// Live install progress (fed by the worker thread).
#[derive(Clone, Debug, Default)]
pub struct ProgressState {
    pub steps: Vec<StepView>,
    pub overall_percent: f32,
    pub finished: bool,
    pub success: bool,
    pub ven_version: Option<String>,
    pub error: Option<String>,
    /// Set when UAC/sudo relaunch was triggered instead of in-process
    /// install. Currently scaffolding — the worker writes these so the
    /// Done/Progress screens can render an "elevation in progress" banner
    /// in a follow-up patch. Marked `dead_code` until the GUI starts
    /// reading them so CI's `-D warnings` stays green.
    #[allow(dead_code)]
    pub elevation_launched: bool,
    #[allow(dead_code)]
    pub elevation_message: Option<String>,
}

impl ProgressState {
    pub fn new_empty() -> Self {
        let steps = (1..=TOTAL_STEPS)
            .map(|i| StepView {
                label: format!("Step {i}"),
                detail: String::new(),
                status: StepStatus::Pending,
                log_lines: Vec::new(),
            })
            .collect();
        Self {
            steps,
            overall_percent: 0.0,
            finished: false,
            success: false,
            ven_version: None,
            error: None,
            elevation_launched: false,
            elevation_message: None,
        }
    }

    pub fn apply_event(&mut self, event: &ProgressEvent) {
        match event {
            ProgressEvent::StepStarted { index, label, .. } => {
                let i = index.saturating_sub(1);
                if let Some(s) = self.steps.get_mut(i) {
                    s.label = label.clone();
                    s.detail.clear();
                    s.status = StepStatus::Running;
                }
                self.overall_percent = ((i as f32) / TOTAL_STEPS as f32).min(1.0);
            }
            ProgressEvent::StepDetail { sub_label } => {
                if let Some(s) = self
                    .steps
                    .iter_mut()
                    .find(|s| s.status == StepStatus::Running)
                {
                    s.detail = sub_label.clone();
                }
            }
            ProgressEvent::StepLog { line } => {
                if let Some(s) = self
                    .steps
                    .iter_mut()
                    .find(|s| s.status == StepStatus::Running)
                {
                    s.log_lines.push(line.clone());
                    if s.log_lines.len() > 200 {
                        s.log_lines.remove(0);
                    }
                }
            }
            ProgressEvent::StepCompleted { index, skipped } => {
                let i = index.saturating_sub(1);
                if let Some(s) = self.steps.get_mut(i) {
                    // `skipped` is `&bool` here because `apply_event` takes
                    // `event: &ProgressEvent` and we destructure by reference.
                    s.status = if *skipped {
                        StepStatus::Skipped
                    } else {
                        StepStatus::Done
                    };
                }
                self.overall_percent = (*index as f32 / TOTAL_STEPS as f32).min(1.0);
            }
            ProgressEvent::InstallCompleted { ven_version } => {
                self.finished = true;
                self.success = true;
                self.ven_version = ven_version.clone();
                self.overall_percent = 1.0;
                for s in &mut self.steps {
                    if s.status == StepStatus::Running {
                        s.status = StepStatus::Done;
                    }
                }
            }
            ProgressEvent::InstallFailed { error } => {
                self.finished = true;
                self.success = false;
                self.error = Some(error.clone());
                if let Some(s) = self
                    .steps
                    .iter_mut()
                    .find(|s| s.status == StepStatus::Running)
                {
                    s.status = StepStatus::Failed;
                }
            }
        }
    }
}

/// All user choices + UI-only state for the wizard.
pub struct WizardState {
    pub screen: Screen,
    pub install_mode: InstallMode,
    pub storage_path: PathBuf,
    pub storage_error: Option<String>,
    pub add_to_path: bool,
    pub install_hook: bool,
    /// Parallel to [`RUNTIME_OPTIONS`]: whether to pre-install each runtime.
    pub runtime_selected: [bool; 8],
    pub progress: ProgressState,
    pub progress_rx: Option<Receiver<ProgressEvent>>,
    pub done_message: Option<String>,
    pub show_cancel_confirm: bool,
    pub pending_browse: bool,
    pub logo_texture: Option<egui::TextureHandle>,
    pub dry_run: bool,
}

impl WizardState {
    pub fn new(dry_run: bool) -> Self {
        Self {
            screen: Screen::Welcome,
            install_mode: InstallMode::User,
            storage_path: default_storage_path(),
            storage_error: None,
            add_to_path: true,
            install_hook: true,
            runtime_selected: [false; 8],
            progress: ProgressState::new_empty(),
            progress_rx: None,
            done_message: None,
            show_cancel_confirm: false,
            pending_browse: false,
            logo_texture: None,
            dry_run,
        }
    }

    pub fn selected_runtimes(&self) -> Vec<String> {
        RUNTIME_OPTIONS
            .iter()
            .zip(self.runtime_selected.iter())
            .filter(|(_, on)| **on)
            .map(|(r, _)| r.slug.to_string())
            .collect()
    }

    pub fn validate_storage(&mut self) {
        let p = &self.storage_path;
        if p.as_os_str().is_empty() {
            self.storage_error = Some("Path cannot be empty".into());
            return;
        }
        if let Some(parent) = p.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    self.storage_error = Some(format!("Cannot create parent directory: {e}"));
                    return;
                }
            }
        }
        let probe = p.join(".ven-setup-write-test");
        match std::fs::write(&probe, b"x") {
            Ok(()) => {
                let _ = std::fs::remove_file(&probe);
                self.storage_error = None;
            }
            Err(e) => {
                self.storage_error = Some(format!("Directory is not writable: {e}"));
            }
        }
    }

    pub fn can_advance_from_storage(&self) -> bool {
        self.storage_error.is_none() && !self.storage_path.as_os_str().is_empty()
    }
}
