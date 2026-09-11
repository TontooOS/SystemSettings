//! Settings daemon client (WiFi domain, backend wiring only).
//!
//! Covers the public read ops (`wifi_list`, `wifi_status`) and the private
//! write ops (`wifi_connect`, `wifi_disconnect`, `wifi_enable`,
//! `wifi_disable`, `wifi_forget`) reserved for this app
//! (`com.tontoo.systemsettings`). No UI code uses this module yet;
//! frontend wiring is a later step.
//!
//! The Customize page is the first daemon-wired UI: `customize_get`
//! (public read) loads the effective wallpaper pack, accent color and
//! theme, `customize_set` (private write) persists partial updates.
//!
//! The protocol is newline-delimited JSON over a unix socket:
//! `{"id": 1, "op": ..., "params": {...}}` with replies shaped
//! `{"id": 1, "ok": bool, "result": ...}` or `{"id": 1, "ok": false,
//! "error": ...}`.

// Backend wiring without frontend use yet; the UI keeps showing example
// content until the WiFi page is connected in a later step.
#![allow(dead_code)]

use serde::Deserialize;
use std::path::PathBuf;

pub const DEFAULT_SOCKET_PATH: &str = "/run/tontoo-settings.sock";

/// One nearby network as reported by `wifi_list`.
#[derive(Debug, Clone, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: Option<String>,
    pub signal_pct: i32,
    pub frequency_mhz: Option<u32>,
    pub security: String,
    #[serde(default)]
    pub known: bool,
}

/// Active connection details as reported by `wifi_status`.
#[derive(Debug, Clone, Deserialize)]
pub struct WifiStatus {
    pub interface: String,
    pub ssid: Option<String>,
    pub bssid: Option<String>,
    pub signal_pct: i32,
    pub frequency_mhz: Option<u32>,
    pub security: String,
    pub state: String,
    pub ipv4: Option<String>,
}

/// Daemon socket path (`SETTINGS_SOCKET` override, else the default).
pub fn socket_path() -> PathBuf {
    std::env::var("SETTINGS_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(DEFAULT_SOCKET_PATH))
}

/// True when the daemon socket file exists.
pub fn daemon_available() -> bool {
    socket_path().exists()
}

/// Send one request frame, return the `result` of a success frame.
fn call(op: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
    let reply = transact(op, params)?;
    if reply
        .get("ok")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        Ok(reply
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null))
    } else {
        Err(reply
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("daemon error")
            .to_string())
    }
}

#[cfg(unix)]
fn transact(op: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let path = socket_path();
    if !path.exists() {
        return Err("settings daemon unreachable".to_string());
    }
    let mut stream = UnixStream::connect(&path)
        .map_err(|e| format!("settings daemon unreachable: {}", e))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(15)));

    let mut line = serde_json::json!({"id": 1, "op": op, "params": params}).to_string();
    line.push('\n');
    stream
        .write_all(line.as_bytes())
        .map_err(|e| format!("daemon write failed: {}", e))?;
    stream.flush().map_err(|e| format!("daemon write failed: {}", e))?;

    let mut reader = BufReader::new(&stream);
    let mut reply = String::new();
    reader
        .read_line(&mut reply)
        .map_err(|e| format!("daemon read failed: {}", e))?;
    serde_json::from_str(&reply).map_err(|e| format!("daemon reply invalid: {}", e))
}

#[cfg(not(unix))]
fn transact(_op: &str, _params: serde_json::Value) -> Result<serde_json::Value, String> {
    Err("settings daemon is Linux-only".to_string())
}

/// Nearby networks with the known flag (`wifi_list`, public).
pub fn list() -> Result<Vec<WifiNetwork>, String> {
    let result = call("wifi_list", serde_json::json!({}))?;
    serde_json::from_value(result.get("networks").cloned().unwrap_or(serde_json::Value::Null))
        .map_err(|e| format!("wifi list invalid: {}", e))
}

/// Radio state plus the active connection (`wifi_status`, public).
/// Returns `(enabled, status)`.
pub fn status() -> Result<(bool, Option<WifiStatus>), String> {
    let result = call("wifi_status", serde_json::json!({}))?;
    let enabled = result
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let current: Option<WifiStatus> =
        serde_json::from_value(result.get("status").cloned().unwrap_or(serde_json::Value::Null))
            .map_err(|e| format!("wifi status invalid: {}", e))?;
    Ok((enabled, current))
}

/// Connect to a network (`wifi_connect`, private). Open networks take
/// `password: None`. Returns the verified connection status.
pub fn connect(
    ssid: &str,
    password: Option<&str>,
    hidden: bool,
) -> Result<WifiStatus, String> {
    let result = call(
        "wifi_connect",
        serde_json::json!({"ssid": ssid, "password": password, "hidden": hidden}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("wifi connect invalid: {}", e))
}

/// Disconnect the wireless interface (`wifi_disconnect`, private).
pub fn disconnect() -> Result<(), String> {
    call("wifi_disconnect", serde_json::json!({}))?;
    Ok(())
}

/// Switch the WLAN radio on or off (`wifi_enable`/`wifi_disable`, private).
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    let op = if enabled { "wifi_enable" } else { "wifi_disable" };
    call(op, serde_json::json!({}))?;
    Ok(())
}

/// Forget a known network (`wifi_forget`, private). Returns true when a
/// profile or store entry was removed.
pub fn forget(ssid: &str) -> Result<bool, String> {
    let result = call("wifi_forget", serde_json::json!({"ssid": ssid}))?;
    Ok(result
        .get("forgotten")
        .and_then(|v| v.as_bool())
        .unwrap_or(false))
}

/// Effective desktop customization as reported by `customize_get`.
/// Missing fields fall back to the daemon defaults (`THAOELAKE`,
/// `orange`, `dark`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(default)]
pub struct CustomizeSettings {
    pub wallpaper: String,
    pub accent: String,
    pub theme: String,
}

impl Default for CustomizeSettings {
    fn default() -> Self {
        Self {
            wallpaper: "THAOELAKE".to_string(),
            accent: "orange".to_string(),
            theme: "dark".to_string(),
        }
    }
}

/// Read the effective customization (`customize_get`, public).
pub fn customize_get() -> Result<CustomizeSettings, String> {
    let result = call("customize_get", serde_json::json!({}))?;
    serde_json::from_value(result).map_err(|e| format!("customize get invalid: {}", e))
}

/// Apply a partial customization update (`customize_set`, private).
/// `None` leaves the key untouched. Returns the effective settings.
pub fn customize_set(
    wallpaper: Option<&str>,
    accent: Option<&str>,
    theme: Option<&str>,
) -> Result<CustomizeSettings, String> {
    let result = call(
        "customize_set",
        serde_json::json!({"wallpaper": wallpaper, "accent": accent, "theme": theme}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("customize set invalid: {}", e))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::net::UnixListener;

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn unique_socket(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "systemsettings-{}-{}.sock",
            name,
            std::process::id()
        ))
    }

    /// Serve exactly one canned reply frame, then stop.
    fn serve_once(path: PathBuf, reply: serde_json::Value) {
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                use std::io::{BufRead, BufReader, Write};
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                let _ = reader.read_line(&mut line);
                let mut out = reply.to_string();
                out.push('\n');
                let _ = stream.write_all(out.as_bytes());
            }
            let _ = std::fs::remove_file(&path);
        });
    }

    #[test]
    fn socket_override_and_unreachable() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("SETTINGS_SOCKET", "/tmp/custom-settings-test.sock");
        assert_eq!(socket_path(), PathBuf::from("/tmp/custom-settings-test.sock"));
        assert!(!daemon_available());
        std::env::set_var(
            "SETTINGS_SOCKET",
            "/nonexistent-systemsettings-test.sock",
        );
        assert!(list().is_err());
        assert!(status().is_err());
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn list_roundtrip_marks_known() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("list");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result": {"networks": [
                {"ssid": "HomeNet", "bssid": null, "signal_pct": 80,
                 "frequency_mhz": 2437, "security": "WPA2", "known": true},
                {"ssid": "Cafe", "bssid": null, "signal_pct": 40,
                 "frequency_mhz": 5180, "security": "OPEN", "known": false},
            ]}}),
        );
        let networks = list().unwrap();
        assert_eq!(networks.len(), 2);
        assert!(networks[0].known);
        assert!(!networks[1].known);
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn status_and_private_ops_roundtrip() {
        let _guard = ENV_LOCK.lock().unwrap();
        let reply = serde_json::json!({"id": 1, "ok": true, "result":
            {"enabled": true, "status": {"interface": "wlan0", "ssid": "HomeNet",
             "bssid": null, "signal_pct": 80, "frequency_mhz": 2437,
             "security": "WPA2", "state": "connected", "ipv4": "192.168.1.5"}}});

        let path = unique_socket("status");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(path.clone(), reply.clone());
        let (enabled, current) = status().unwrap();
        assert!(enabled);
        assert_eq!(current.unwrap().ssid.as_deref(), Some("HomeNet"));

        let path = unique_socket("connect");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result": reply["result"]["status"]}),
        );
        let connected = connect("HomeNet", None, false).unwrap();
        assert_eq!(connected.ssid.as_deref(), Some("HomeNet"));

        let path = unique_socket("forget");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result": {"forgotten": true}}),
        );
        assert!(forget("HomeNet").unwrap());
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn daemon_error_frame_surfaces() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("error");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": false, "error": "wifi connect failed: nope"}),
        );
        let err = connect("Nope", None, false).unwrap_err();
        assert!(err.contains("wifi connect failed"));
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn customize_get_roundtrip() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("customize-get");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"wallpaper": "SONOMA", "accent": "blue", "theme": "light"}}),
        );
        let settings = customize_get().unwrap();
        assert_eq!(
            settings,
            CustomizeSettings {
                wallpaper: "SONOMA".to_string(),
                accent: "blue".to_string(),
                theme: "light".to_string(),
            }
        );
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn customize_get_missing_fields_fall_back_to_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("customize-partial");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result": {"theme": "light"}}),
        );
        let settings = customize_get().unwrap();
        assert_eq!(settings.wallpaper, "THAOELAKE");
        assert_eq!(settings.accent, "orange");
        assert_eq!(settings.theme, "light");
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn customize_set_roundtrip_and_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("customize-set");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"wallpaper": "VENTURA", "accent": "orange", "theme": "dark"}}),
        );
        let applied = customize_set(Some("VENTURA"), None, None).unwrap();
        assert_eq!(applied.wallpaper, "VENTURA");

        let path = unique_socket("customize-set-error");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": false, "error": "customize set failed: unknown theme"}),
        );
        let err = customize_set(None, None, Some("sepia")).unwrap_err();
        assert!(err.contains("unknown theme"));
        std::env::remove_var("SETTINGS_SOCKET");
    }
}
