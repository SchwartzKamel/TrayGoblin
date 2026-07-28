pub mod config;
pub mod events;
pub mod monitor;
pub mod session;
pub mod status;

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
