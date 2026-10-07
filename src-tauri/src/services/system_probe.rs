use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tracing::warn;

use crate::domain::{NetworkOutcome, PowerSource};
use crate::error::AppError;

const PMSET: &str = "/usr/bin/pmset";
const ROUTE: &str = "/sbin/route";
const NETWORKSETUP: &str = "/usr/sbin/networksetup";
const CAFFEINATE: &str = "/usr/bin/caffeinate";
const NETWORK_POLL: Duration = Duration::from_secs(2);

/// Machine state the job runner depends on. Faked in tests.
pub trait SystemProbe: Send + Sync {
    fn power_source(&self) -> PowerSource;
    fn is_online(&self) -> bool;
    fn ensure_wifi_on(&self);
    fn hold_awake(&self, prevent_system_sleep: bool) -> AwakeHold;
}

/// Releases the sleep assertion when dropped.
pub struct AwakeHold {
    child: Option<Child>,
}

impl AwakeHold {
    pub fn none() -> Self {
        Self { child: None }
    }
}

impl Drop for AwakeHold {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub struct MacSystemProbe;

impl SystemProbe for MacSystemProbe {
    fn power_source(&self) -> PowerSource {
        command_stdout(PMSET, &["-g", "batt"])
            .map(|out| parse_power_source(&out))
            .unwrap_or(PowerSource::Unknown)
    }

    fn is_online(&self) -> bool {
        command_stdout(ROUTE, &["-n", "get", "default"])
            .map(|out| parse_has_default_route(&out))
            .unwrap_or(false)
    }

    fn ensure_wifi_on(&self) {
        let Some(device) = command_stdout(NETWORKSETUP, &["-listallhardwareports"])
            .and_then(|out| parse_wifi_device(&out))
        else {
            return;
        };
        let powered = command_stdout(NETWORKSETUP, &["-getairportpower", &device])
            .map(|out| out.trim_end().ends_with("On"))
            .unwrap_or(true);
        if !powered {
            if let Err(err) = run_quiet(NETWORKSETUP, &["-setairportpower", &device, "on"]) {
                warn!(error = %err, "could not turn Wi-Fi on");
            }
        }
    }

    fn hold_awake(&self, prevent_system_sleep: bool) -> AwakeHold {
        let flags = if prevent_system_sleep { "-ims" } else { "-im" };
        let owner = std::process::id().to_string();
        let child = Command::new(CAFFEINATE)
            .args([flags, "-w", &owner])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        match child {
            Ok(child) => AwakeHold { child: Some(child) },
            Err(err) => {
                warn!(error = %err, "could not start caffeinate");
                AwakeHold::none()
            }
        }
    }
}

/// Poll for a default route until online or the grace period ends.
pub fn wait_for_network(probe: &dyn SystemProbe, grace: Duration) -> NetworkOutcome {
    probe.ensure_wifi_on();
    let deadline = Instant::now() + grace;
    loop {
        if probe.is_online() {
            return NetworkOutcome::Online;
        }
        if Instant::now() >= deadline {
            return NetworkOutcome::Offline;
        }
        thread::sleep(NETWORK_POLL.min(grace));
    }
}

fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).stdin(Stdio::null()).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_quiet(program: &str, args: &[&str]) -> Result<(), AppError> {
    let status = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::message(format!("{program} exited with {status}")))
    }
}

pub fn parse_power_source(pmset_batt: &str) -> PowerSource {
    if pmset_batt.contains("'AC Power'") {
        PowerSource::Ac
    } else if pmset_batt.contains("'Battery Power'") {
        PowerSource::Battery
    } else {
        PowerSource::Unknown
    }
}

pub fn parse_has_default_route(route_output: &str) -> bool {
    route_output.lines().any(|line| line.trim().starts_with("interface:"))
}

pub fn parse_wifi_device(hardware_ports: &str) -> Option<String> {
    let mut lines = hardware_ports.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "Hardware Port: Wi-Fi" || line.trim() == "Hardware Port: AirPort" {
            return lines
                .next()
                .and_then(|next| next.trim().strip_prefix("Device: "))
                .map(str::to_string);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_power_source() {
        assert_eq!(parse_power_source("Now drawing from 'AC Power'\n"), PowerSource::Ac);
        assert_eq!(parse_power_source("Now drawing from 'Battery Power'\n"), PowerSource::Battery);
        assert_eq!(parse_power_source(""), PowerSource::Unknown);
    }

    #[test]
    fn parses_default_route() {
        let online = "   route to: default\ndestination: default\n  interface: en0\n";
        assert!(parse_has_default_route(online));
        assert!(!parse_has_default_route("route: writing to routing socket: not in table\n"));
    }

    #[test]
    fn finds_wifi_device() {
        let ports = "Hardware Port: Thunderbolt Bridge\nDevice: bridge0\n\nHardware Port: Wi-Fi\nDevice: en0\nEthernet Address: aa\n";
        assert_eq!(parse_wifi_device(ports).as_deref(), Some("en0"));
        assert_eq!(parse_wifi_device("Hardware Port: USB\nDevice: en5\n"), None);
    }
}
