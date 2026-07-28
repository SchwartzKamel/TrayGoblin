//! TrayGoblin's notification-area executable.
//!
//! On Windows this starts the Win32 tray shell. On other hosts it explains
//! how to exercise the same platform-neutral monitor through the diagnostic
//! probe, so a non-Windows developer still gets an actionable message.

// Release builds have no console window; the tray is the entire interface.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::process::ExitCode;

#[cfg(windows)]
fn run() -> ExitCode {
    match tray_goblin::tray::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tray_goblin::tray::show_startup_error(&error);
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn run() -> ExitCode {
    eprintln!(
        "error: {} runs in the Windows notification area and cannot start on this host.",
        tray_goblin::APPLICATION_NAME
    );
    eprintln!(
        "Use `cargo run --bin tray-goblin-probe -- --session-root <path>` to inspect session state here."
    );
    ExitCode::FAILURE
}

fn main() -> ExitCode {
    run()
}
