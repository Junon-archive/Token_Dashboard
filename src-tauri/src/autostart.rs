use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

const APP_ID: &str = "io.github.junonarchive.token-dashboard";
const APP_NAME: &str = "Token Dashboard";

#[derive(Debug, Error)]
pub enum AutostartError {
    #[error("autostart is unsupported on this platform")]
    Unsupported,
    #[error("autostart path is unavailable")]
    PathUnavailable,
    #[error("autostart IO failed")]
    Io(#[from] std::io::Error),
}

pub fn set_autostart(enabled: bool) -> Result<(), AutostartError> {
    set_autostart_for_executable(enabled, &std::env::current_exe()?)
}

pub fn set_autostart_for_executable(
    enabled: bool,
    executable: &Path,
) -> Result<(), AutostartError> {
    #[cfg(target_os = "linux")]
    {
        let config_dir = dirs::config_dir().ok_or(AutostartError::PathUnavailable)?;
        return set_linux_autostart(enabled, executable, &config_dir);
    }

    #[cfg(target_os = "macos")]
    {
        let home_dir = dirs::home_dir().ok_or(AutostartError::PathUnavailable)?;
        return set_macos_autostart(enabled, executable, &home_dir);
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (enabled, executable);
        Err(AutostartError::Unsupported)
    }
}

#[cfg(target_os = "linux")]
fn set_linux_autostart(
    enabled: bool,
    executable: &Path,
    config_dir: &Path,
) -> Result<(), AutostartError> {
    let path = linux_autostart_path(config_dir);
    if !enabled {
        remove_if_exists(&path)?;
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, linux_desktop_entry(executable))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn linux_autostart_path(config_dir: &Path) -> PathBuf {
    config_dir
        .join("autostart")
        .join(format!("{APP_ID}.desktop"))
}

#[cfg(target_os = "linux")]
fn linux_desktop_entry(executable: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={APP_NAME}\nExec=\"{}\"\nTerminal=false\nX-GNOME-Autostart-enabled=true\n",
        desktop_exec_escape(executable)
    )
}

#[cfg(target_os = "linux")]
fn desktop_exec_escape(executable: &Path) -> String {
    executable
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
}

#[cfg(target_os = "macos")]
fn set_macos_autostart(
    enabled: bool,
    executable: &Path,
    home_dir: &Path,
) -> Result<(), AutostartError> {
    let path = macos_launch_agent_path(home_dir);
    if !enabled {
        remove_if_exists(&path)?;
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, macos_launch_agent_plist(executable))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_launch_agent_path(home_dir: &Path) -> PathBuf {
    home_dir
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{APP_ID}.plist"))
}

#[cfg(target_os = "macos")]
fn macos_launch_agent_plist(executable: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{APP_ID}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
</dict>
</plist>
"#,
        xml_escape(&executable.to_string_lossy())
    )
}

#[cfg(target_os = "macos")]
fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn remove_if_exists(path: &Path) -> Result<(), AutostartError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_desktop_entry_quotes_and_escapes_executable() {
        let entry = linux_desktop_entry(Path::new("/tmp/Token Dashboard/bin\"app"));

        assert!(entry.contains("Type=Application"));
        assert!(entry.contains("Name=Token Dashboard"));
        assert!(entry.contains("Exec=\"/tmp/Token Dashboard/bin\\\"app\""));
        assert!(!entry.contains("Bearer "));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_autostart_enable_and_disable_use_xdg_path() {
        let temp = tempfile::tempdir().unwrap();
        let path = linux_autostart_path(temp.path());

        set_linux_autostart(true, Path::new("/tmp/token-dashboard"), temp.path()).unwrap();
        assert!(path.exists());
        let contents = fs::read_to_string(&path).unwrap();
        assert!(contents.contains("X-GNOME-Autostart-enabled=true"));

        set_linux_autostart(false, Path::new("/tmp/token-dashboard"), temp.path()).unwrap();
        assert!(!path.exists());
        set_linux_autostart(false, Path::new("/tmp/token-dashboard"), temp.path()).unwrap();
    }
}
