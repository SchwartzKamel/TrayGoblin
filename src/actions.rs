//! Platform-neutral tray menu actions and launch-target selection.
//!
//! Target selection is pure: it maps a menu command plus the currently known
//! session context to a local target, or to a typed, content-free error. The
//! Windows shell owns the actual process launch, so this module stays
//! host-testable and never embeds an operating-system error string that
//! could echo session content.

use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// Command identifiers are also used as Win32 menu item identifiers, so they
/// must be non-zero: `TrackPopupMenu` reports 0 when nothing was selected.
const FIRST_COMMAND_ID: u32 = 1;

/// JSON written when the user opens settings for the first time. Kept in
/// sync with [`crate::config::MonitorConfig`] by `actions::tests`.
pub const DEFAULT_SETTINGS_TEMPLATE: &str = concat!(
    "{\n",
    "  \"pollIntervalMs\": 1000,\n",
    "  \"sessionRoot\": null\n",
    "}\n"
);

/// The tray's right-click menu, in display order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrayAction {
    RefreshNow,
    OpenInVsCode,
    ViewCopilotLogs,
    OpenSettings,
    Quit,
}

/// Menu contents required by `specs/tray-status.md`.
pub const MENU_ITEMS: [TrayAction; 5] = [
    TrayAction::RefreshNow,
    TrayAction::OpenInVsCode,
    TrayAction::ViewCopilotLogs,
    TrayAction::OpenSettings,
    TrayAction::Quit,
];

impl TrayAction {
    pub fn label(self) -> &'static str {
        match self {
            Self::RefreshNow => "Refresh now",
            Self::OpenInVsCode => "Open in VS Code",
            Self::ViewCopilotLogs => "View Copilot logs",
            Self::OpenSettings => "Open settings",
            Self::Quit => "Quit",
        }
    }

    pub fn command_id(self) -> u32 {
        let index = MENU_ITEMS
            .iter()
            .position(|item| *item == self)
            .expect("every action is listed in MENU_ITEMS");
        FIRST_COMMAND_ID + index as u32
    }

    pub fn from_command_id(command_id: u32) -> Option<Self> {
        let index = command_id.checked_sub(FIRST_COMMAND_ID)? as usize;
        MENU_ITEMS.get(index).copied()
    }
}

/// Everything target selection is allowed to know. Only local paths, never
/// prompt, response, tool, token, or credential data.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActionContext {
    pub session_root: Option<PathBuf>,
    pub active_session: Option<PathBuf>,
    pub active_directory: Option<PathBuf>,
    pub settings_path: Option<PathBuf>,
}

/// A local target the Windows shell should open.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LaunchTarget {
    /// Open a directory in VS Code.
    Editor { directory: PathBuf },
    /// Reveal a directory in the file manager.
    Folder { directory: PathBuf },
    /// Open a file with its registered handler.
    File { path: PathBuf },
}

/// Content-free action failures. Display text names the action and the
/// missing prerequisite only; it never includes a path, an operating-system
/// message, or any session data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionError {
    NotLaunchable(TrayAction),
    NoActiveDirectory,
    NoSessionFolder,
    NoSettingsPath,
    SettingsUnavailable,
    LaunchFailed(TrayAction),
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotLaunchable(action) => {
                write!(f, "{} does not open a local target", action.label())
            }
            Self::NoActiveDirectory => write!(
                f,
                "Open in VS Code is unavailable: no active Copilot directory is known"
            ),
            Self::NoSessionFolder => write!(
                f,
                "View Copilot logs is unavailable: no session folder is configured"
            ),
            Self::NoSettingsPath => write!(
                f,
                "Open settings is unavailable: no settings location could be determined"
            ),
            Self::SettingsUnavailable => {
                write!(f, "Open settings failed: the settings file is not writable")
            }
            Self::LaunchFailed(action) => write!(f, "{} could not be started", action.label()),
        }
    }
}

impl std::error::Error for ActionError {}

/// Selects the local target for a menu command.
///
/// `Refresh now` and `Quit` are handled by the application loop and are
/// therefore reported as not launchable rather than silently ignored.
pub fn resolve_launch_target(
    action: TrayAction,
    context: &ActionContext,
) -> Result<LaunchTarget, ActionError> {
    match action {
        TrayAction::OpenInVsCode => context
            .active_directory
            .clone()
            .map(|directory| LaunchTarget::Editor { directory })
            .ok_or(ActionError::NoActiveDirectory),
        TrayAction::ViewCopilotLogs => context
            .active_session
            .clone()
            .or_else(|| context.session_root.clone())
            .map(|directory| LaunchTarget::Folder { directory })
            .ok_or(ActionError::NoSessionFolder),
        TrayAction::OpenSettings => context
            .settings_path
            .clone()
            .map(|path| LaunchTarget::File { path })
            .ok_or(ActionError::NoSettingsPath),
        TrayAction::RefreshNow | TrayAction::Quit => Err(ActionError::NotLaunchable(action)),
    }
}

/// Best-effort per-user settings location: `%APPDATA%\TrayGoblin\config.json`
/// on Windows, with an XDG-style fallback so host tooling can resolve one too.
pub fn default_settings_path() -> Option<PathBuf> {
    if let Some(app_data) = std::env::var_os("APPDATA") {
        return Some(
            PathBuf::from(app_data)
                .join(crate::APPLICATION_NAME)
                .join("config.json"),
        );
    }

    if let Some(profile) = std::env::var_os("USERPROFILE") {
        return Some(
            PathBuf::from(profile)
                .join("AppData")
                .join("Roaming")
                .join(crate::APPLICATION_NAME)
                .join("config.json"),
        );
    }

    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("tray-goblin")
            .join("config.json"),
    )
}

/// Creates the settings file with documented defaults when it is missing so
/// `Open settings` always has something to open. An existing file is never
/// overwritten. I/O failures collapse into a content-free error.
pub fn ensure_settings_file(path: &Path) -> Result<(), ActionError> {
    match create_settings_file(path) {
        Ok(()) => Ok(()),
        Err(_) => Err(ActionError::SettingsUnavailable),
    }
}

fn create_settings_file(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| io::Write::write_all(&mut file, DEFAULT_SETTINGS_TEMPLATE.as_bytes()))
        .or_else(|error| {
            // Losing a race with another instance is not a failure.
            if error.kind() == io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(error)
            }
        })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        ActionContext, ActionError, DEFAULT_SETTINGS_TEMPLATE, LaunchTarget, MENU_ITEMS,
        TrayAction, ensure_settings_file, resolve_launch_target,
    };
    use crate::config::{DEFAULT_POLL_INTERVAL_MS, MonitorConfig};

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("test-scratch")
            .join("actions")
            .join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn context() -> ActionContext {
        ActionContext {
            session_root: Some(PathBuf::from("C:/Users/dev/.copilot/session-state")),
            active_session: Some(PathBuf::from("C:/Users/dev/.copilot/session-state/abc")),
            active_directory: Some(PathBuf::from("C:/src/demo")),
            settings_path: Some(PathBuf::from(
                "C:/Users/dev/AppData/Roaming/TrayGoblin/config.json",
            )),
        }
    }

    // This pins the exact menu required by the acceptance spec.
    #[test]
    fn menu_matches_the_specified_commands_in_order() {
        let labels: Vec<&str> = MENU_ITEMS.iter().map(|action| action.label()).collect();

        assert_eq!(
            labels,
            vec![
                "Refresh now",
                "Open in VS Code",
                "View Copilot logs",
                "Open settings",
                "Quit"
            ]
        );
    }

    // Win32 reports 0 for "no selection", so every command id must be
    // non-zero, unique, and round-trip back to its action.
    #[test]
    fn command_ids_are_non_zero_and_round_trip() {
        let mut seen = Vec::new();
        for action in MENU_ITEMS {
            let id = action.command_id();

            assert_ne!(id, 0);
            assert!(!seen.contains(&id), "command ids must be unique");
            assert_eq!(TrayAction::from_command_id(id), Some(action));
            seen.push(id);
        }

        assert_eq!(TrayAction::from_command_id(0), None);
        assert_eq!(TrayAction::from_command_id(u32::MAX), None);
    }

    // Open in VS Code targets the observed active directory, never a
    // repository name or any parsed event content.
    #[test]
    fn open_in_vs_code_targets_the_active_directory() {
        let target = resolve_launch_target(TrayAction::OpenInVsCode, &context()).unwrap();

        assert_eq!(
            target,
            LaunchTarget::Editor {
                directory: PathBuf::from("C:/src/demo")
            }
        );
    }

    // Without an active session there is nothing to open, and the failure
    // must be typed rather than launching an arbitrary directory.
    #[test]
    fn open_in_vs_code_without_active_directory_is_a_safe_error() {
        let context = ActionContext {
            active_directory: None,
            ..context()
        };

        assert_eq!(
            resolve_launch_target(TrayAction::OpenInVsCode, &context).unwrap_err(),
            ActionError::NoActiveDirectory
        );
    }

    // The logs command prefers the active session folder so the user lands
    // on the state that produced the current tray status.
    #[test]
    fn view_copilot_logs_prefers_the_active_session_folder() {
        let target = resolve_launch_target(TrayAction::ViewCopilotLogs, &context()).unwrap();

        assert_eq!(
            target,
            LaunchTarget::Folder {
                directory: PathBuf::from("C:/Users/dev/.copilot/session-state/abc")
            }
        );
    }

    // With no active session the session root is still useful, which keeps
    // the command working during the Idle-with-no-session case.
    #[test]
    fn view_copilot_logs_falls_back_to_the_session_root() {
        let context = ActionContext {
            active_session: None,
            ..context()
        };

        let target = resolve_launch_target(TrayAction::ViewCopilotLogs, &context).unwrap();

        assert_eq!(
            target,
            LaunchTarget::Folder {
                directory: PathBuf::from("C:/Users/dev/.copilot/session-state")
            }
        );
    }

    // Settings are a documented JSON file per ADR 0001 (no settings window).
    #[test]
    fn open_settings_targets_the_settings_file() {
        let target = resolve_launch_target(TrayAction::OpenSettings, &context()).unwrap();

        assert_eq!(
            target,
            LaunchTarget::File {
                path: PathBuf::from("C:/Users/dev/AppData/Roaming/TrayGoblin/config.json")
            }
        );
    }

    #[test]
    fn open_settings_without_a_location_is_a_safe_error() {
        let context = ActionContext {
            settings_path: None,
            ..context()
        };

        assert_eq!(
            resolve_launch_target(TrayAction::OpenSettings, &context).unwrap_err(),
            ActionError::NoSettingsPath
        );
    }

    // Refresh and Quit are loop concerns; resolving them must not silently
    // succeed with a bogus target.
    #[test]
    fn loop_only_commands_are_not_launch_targets() {
        for action in [TrayAction::RefreshNow, TrayAction::Quit] {
            assert_eq!(
                resolve_launch_target(action, &context()).unwrap_err(),
                ActionError::NotLaunchable(action)
            );
        }
    }

    // Privacy contract: user-visible failures name the action only. No path,
    // session identifier, or operating-system message may appear.
    #[test]
    fn action_errors_are_content_free() {
        let secretive = ActionContext {
            session_root: Some(PathBuf::from("C:/secret-root")),
            active_session: Some(PathBuf::from("C:/secret-session")),
            active_directory: Some(PathBuf::from("C:/secret-directory")),
            settings_path: Some(PathBuf::from("C:/secret-settings.json")),
        };
        let errors = [
            ActionError::NoActiveDirectory,
            ActionError::NoSessionFolder,
            ActionError::NoSettingsPath,
            ActionError::SettingsUnavailable,
            ActionError::LaunchFailed(TrayAction::OpenInVsCode),
            ActionError::NotLaunchable(TrayAction::Quit),
        ];

        for error in errors {
            let text = error.to_string();

            assert!(!text.contains("secret"), "leaked path fragment: {text}");
            assert!(
                !text.contains('\\') && !text.contains('/'),
                "leaked a path: {text}"
            );
            assert!(text.len() < 100, "error text should stay concise: {text}");
        }

        // Resolution must not mutate or echo the context either.
        let before = secretive.clone();
        let _ = resolve_launch_target(TrayAction::RefreshNow, &secretive);
        assert_eq!(secretive, before);
    }

    // The generated defaults must be readable by the configuration parser,
    // otherwise Open settings would hand the user an unusable file.
    #[test]
    fn default_settings_template_parses_with_documented_defaults() {
        let config = MonitorConfig::parse(DEFAULT_SETTINGS_TEMPLATE).unwrap();

        assert_eq!(config.poll_interval_ms(), DEFAULT_POLL_INTERVAL_MS);
        assert_eq!(config.session_root(), None);
    }

    // Opening settings creates the file once and never clobbers user edits.
    #[test]
    fn ensure_settings_file_creates_once_and_preserves_edits() {
        let dir = scratch_dir("ensure-settings");
        let path = dir.join("nested").join("config.json");

        ensure_settings_file(&path).unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            DEFAULT_SETTINGS_TEMPLATE
        );

        std::fs::write(&path, r#"{"pollIntervalMs":2000}"#).unwrap();
        ensure_settings_file(&path).unwrap();

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            r#"{"pollIntervalMs":2000}"#
        );
    }

    // An unwritable location must surface the content-free error instead of
    // panicking inside the tray message loop.
    #[test]
    fn ensure_settings_file_reports_a_safe_error() {
        let dir = scratch_dir("ensure-settings-error");
        let blocker = dir.join("blocker");
        std::fs::write(&blocker, "not a directory").unwrap();

        let error = ensure_settings_file(&blocker.join("config.json")).unwrap_err();

        assert_eq!(error, ActionError::SettingsUnavailable);
    }
}
