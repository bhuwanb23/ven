//! Background install worker for the GUI Progress screen.

use std::sync::mpsc::{self, Sender};
use std::thread;

use anyhow::Result;

use crate::common::InstallMode;
use crate::gui::state::WizardState;
use crate::install_steps::{default_storage_path, InstallConfig, ProgressEvent, ProgressSink};

/// Forward install events to the GUI thread.
pub struct ChannelSink(pub Sender<ProgressEvent>);

impl ProgressSink for ChannelSink {
    fn emit(&mut self, event: ProgressEvent) {
        let _ = self.0.send(event);
    }
}

/// Build the install config the pipeline expects from wizard choices.
pub fn config_from_wizard(state: &WizardState) -> InstallConfig {
    let mut cfg = InstallConfig::default_for_mode(state.install_mode);
    cfg.dry_run = state.dry_run;
    cfg.add_to_path = state.add_to_path;
    cfg.install_hook = state.install_hook;
    cfg.runtimes_to_install = state.selected_runtimes();
    let default = default_storage_path();
    if state.storage_path != default {
        cfg.storage_path = Some(state.storage_path.clone());
    }
    cfg
}

/// Spawn the install on a background thread. Returns the receiver the UI polls each frame.
pub fn spawn_install(state: &WizardState) -> mpsc::Receiver<ProgressEvent> {
    let cfg = config_from_wizard(state);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        if let Err(e) = run_install_job(cfg, &tx) {
            let _ = tx.send(ProgressEvent::InstallFailed {
                error: format!("{e:#}"),
            });
        }
    });
    rx
}

fn run_install_job(cfg: InstallConfig, tx: &Sender<ProgressEvent>) -> Result<()> {
    // System install may need elevation before we touch protected dirs.
    if matches!(cfg.mode, InstallMode::System) && !cfg.dry_run {
        #[cfg(windows)]
        {
            if !crate::windows::is_elevated()? {
                crate::windows::relaunch_elevated_system(&cfg)?;
                let _ = tx.send(ProgressEvent::InstallCompleted {
                    ven_version: Some(
                        "Elevated installer launched — complete the UAC prompt, then open a new terminal."
                            .into(),
                    ),
                });
                return Ok(());
            }
        }
        #[cfg(unix)]
        {
            if !crate::unix::is_root() {
                let resume = crate::unix::resume_file_path()?;
                cfg.save_to_file(&resume)?;
                let exe = std::env::current_exe()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "ven-setup".into());
                anyhow::bail!(
                    "System install requires root.\n\nRe-run in a terminal:\n  sudo {exe} --mode system --elevated-child --resume \"{}\"",
                    resume.display()
                );
            }
        }
    }

    let mut sink = ChannelSink(tx.clone());
    crate::install_steps::run(&cfg, &mut sink)?;
    Ok(())
}
