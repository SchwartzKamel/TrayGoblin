pub mod actions;
pub mod app;
pub mod config;
pub mod events;
pub mod icon;
pub mod monitor;
pub mod session;
pub mod status;

/// The notification-area shell is Windows-only; every other module stays
/// platform-neutral so parsing, monitoring, and tray decisions are testable
/// on any host.
#[cfg(windows)]
pub mod tray;

pub const APPLICATION_NAME: &str = "TrayGoblin";

#[cfg(test)]
mod tests {
    use super::APPLICATION_NAME;

    // This smoke test proves the platform-neutral crate executes on the host toolchain.
    #[test]
    fn exposes_the_application_name() {
        assert_eq!(APPLICATION_NAME, "TrayGoblin");
    }
}
