use anyhow::{bail, Result};
use std::path::Path;
use tokio::process::{Child, Command};

/// Spawns the given binary with the given args, elevated when `elevated` is
/// true. TUN interfaces need root/admin on every desktop OS; Proxy mode
/// (plain HTTP/SOCKS listeners) does not, so callers only set `elevated`
/// for `RunMode::Tun`.
///
/// This is real, OS-appropriate elevation (pkexec/osascript/PowerShell
/// RunAs) rather than a stub, but it genuinely cannot be exercised from a
/// headless build sandbox: pkexec needs a polkit agent and a graphical
/// session, osascript needs macOS, and the Windows path needs an actual
/// Windows host. Test each path on its real OS before shipping.
pub fn spawn(binary: &Path, args: &[String], elevated: bool) -> Result<Child> {
    if !elevated {
        return Ok(Command::new(binary).args(args).kill_on_drop(true).spawn()?);
    }

    #[cfg(target_os = "linux")]
    {
        // Requires a polkit agent to be running in the user's session; falls
        // back to `sudo` (which needs a terminal, so it's a worse fallback
        // but still functional for a dev run from a terminal).
        if which_exists("pkexec") {
            return Ok(Command::new("pkexec")
                .arg(binary)
                .args(args)
                .kill_on_drop(true)
                .spawn()?);
        }
        if which_exists("sudo") {
            return Ok(Command::new("sudo")
                .arg(binary)
                .args(args)
                .kill_on_drop(true)
                .spawn()?);
        }
        bail!("neither pkexec nor sudo found; cannot elevate for TUN mode");
    }

    #[cfg(target_os = "macos")]
    {
        let joined = std::iter::once(binary.display().to_string())
            .chain(args.iter().cloned())
            .collect::<Vec<_>>()
            .join(" ");
        let script = format!("do shell script \"{joined}\" with administrator privileges");
        return Ok(Command::new("osascript")
            .arg("-e")
            .arg(script)
            .kill_on_drop(true)
            .spawn()?);
    }

    #[cfg(target_os = "windows")]
    {
        // Launches sing-box in a new elevated process via PowerShell's
        // Start-Process -Verb RunAs, which triggers the UAC prompt.
        let arg_list = args
            .iter()
            .map(|a| format!("'{}'", a.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(",");
        let ps_command = format!(
            "Start-Process -FilePath '{}' -ArgumentList {} -Verb RunAs",
            binary.display(),
            arg_list
        );
        return Ok(Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_command])
            .kill_on_drop(true)
            .spawn()?);
    }

    #[allow(unreachable_code)]
    {
        bail!("elevated TUN launch is not implemented for this OS");
    }
}

#[cfg(target_os = "linux")]
fn which_exists(bin: &str) -> bool {
    std::process::Command::new("which")
        .arg(bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
