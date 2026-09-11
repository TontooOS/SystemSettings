//! Settings daemon client (WiFi domain, backend wiring only).
//!
//! Covers the public read ops (`wifi_list`, `wifi_status`) and the private
//! write ops (`wifi_connect`, `wifi_disconnect`, `wifi_enable`,
//! `wifi_disable`, `wifi_forget`) reserved for this app
//! (`com.tontoo.systemsettings`). No UI code uses this module yet;
//! frontend wiring is a later step.
//!
//! The Wallpaper and Displays pages are daemon-wired: `wallpaper_get`
//! (public read) loads the full wallpaper state, `wallpaper_set_current`,
//! `wallpaper_set_fill` and `wallpaper_add` (private writes) persist the
//! selection and uploads; `display_get` (public read) loads outputs with
//! modes plus brightness and night light, `display_set` (private write)
//! applies settings live and persists them.
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

/// One listed wallpaper: a premade pack or a user custom file.
/// `path_dark` mirrors `path` when no dark variant exists.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct WallpaperEntry {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub path_dark: String,
}

/// Full wallpaper state behind `wallpaper_get`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(default)]
pub struct WallpaperState {
    pub current: Option<WallpaperEntry>,
    pub fill: String,
    pub customs: Vec<WallpaperEntry>,
    pub premade: Vec<WallpaperEntry>,
}

impl Default for WallpaperState {
    fn default() -> Self {
        Self {
            current: None,
            fill: "fill".to_string(),
            customs: Vec::new(),
            premade: Vec::new(),
        }
    }
}

/// Read the full wallpaper state (`wallpaper_get`, public).
pub fn wallpaper_get() -> Result<WallpaperState, String> {
    let result = call("wallpaper_get", serde_json::json!({}))?;
    serde_json::from_value(result).map_err(|e| format!("wallpaper get invalid: {}", e))
}

/// Persist the current wallpaper selection (`wallpaper_set_current`,
/// private). Desktop untouched.
pub fn wallpaper_set_current(kind: &str, id: &str) -> Result<Option<WallpaperEntry>, String> {
    let result = call(
        "wallpaper_set_current",
        serde_json::json!({"kind": kind, "id": id}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("wallpaper set invalid: {}", e))
}

/// Persist the fill mode (`wallpaper_set_fill`, private). Returns the
/// applied mode. Desktop untouched.
pub fn wallpaper_set_fill(fill: &str) -> Result<String, String> {
    let result = call("wallpaper_set_fill", serde_json::json!({"fill": fill}))?;
    result
        .get("fill")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| "wallpaper set invalid: missing fill".to_string())
}

/// Upload an image file to the user customs (`wallpaper_add`, private).
/// The daemon decodes it and stores a PNG. Returns the new entry.
pub fn wallpaper_add(path: &str, name: Option<&str>) -> Result<WallpaperEntry, String> {
    let result = call(
        "wallpaper_add",
        serde_json::json!({"path": path, "name": name}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("wallpaper add invalid: {}", e))
}

/// Apply a wallpaper to the desktop (`wallpaper_apply`, private).
/// `variant` is `light`, `dark` or `auto`. Returns the applied entry.
pub fn wallpaper_apply(kind: &str, id: &str, variant: &str) -> Result<WallpaperEntry, String> {
    let result = call(
        "wallpaper_apply",
        serde_json::json!({"kind": kind, "id": id, "variant": variant}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("wallpaper apply invalid: {}", e))
}

/// One output mode: resolution plus refresh rate in Hz.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct DisplayMode {
    #[serde(default)]
    pub width: i32,
    #[serde(default)]
    pub height: i32,
    #[serde(default)]
    pub refresh: u32,
}

/// One output with its modes and live current mode.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct DisplayOutput {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub modes: Vec<DisplayMode>,
    pub current: Option<DisplayMode>,
}

/// Effective display state behind `display_get`.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default)]
pub struct DisplayState {
    pub outputs: Vec<DisplayOutput>,
    pub brightness: u32,
    pub night_light: bool,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            outputs: Vec::new(),
            brightness: 100,
            night_light: false,
        }
    }
}

/// Read the effective display state (`display_get`, public).
pub fn display_get() -> Result<DisplayState, String> {
    let result = call("display_get", serde_json::json!({}))?;
    serde_json::from_value(result).map_err(|e| format!("display get invalid: {}", e))
}

/// Apply partial display settings live (`display_set`, private).
/// `None` leaves the field untouched. Returns the effective state.
pub fn display_set(
    output: Option<&str>,
    width: Option<i32>,
    height: Option<i32>,
    refresh: Option<u32>,
    brightness: Option<f64>,
    night_light: Option<bool>,
) -> Result<DisplayState, String> {
    let result = call(
        "display_set",
        serde_json::json!({"output": output, "width": width, "height": height,
            "refresh": refresh, "brightness": brightness, "night_light": night_light}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("display set invalid: {}", e))
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
    fn wallpaper_get_roundtrip_with_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("wallpaper-get");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result": {
                "current": null, "fill": "tile",
                "customs": [{"kind": "custom", "id": "mine", "name": "Mine", "path": "/tmp/mine.png"}],
                "premade": []}}),
        );
        let state = wallpaper_get().unwrap();
        assert!(state.current.is_none());
        assert_eq!(state.fill, "tile");
        assert_eq!(state.customs.len(), 1);
        assert_eq!(state.customs[0].id, "mine");
        assert!(state.premade.is_empty());
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn wallpaper_set_fill_roundtrip() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("wallpaper-fill");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result": {"fill": "center"}}),
        );
        assert_eq!(wallpaper_set_fill("center").unwrap(), "center");
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn wallpaper_add_roundtrip_and_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("wallpaper-add");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"kind": "custom", "id": "photo", "name": "Photo", "path": "/tmp/photo.png"}}),
        );
        let entry = wallpaper_add("/tmp/photo.jpg", None).unwrap();
        assert_eq!(entry.id, "photo");

        let path = unique_socket("wallpaper-add-error");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": false, "error": "wallpaper add failed: file not found"}),
        );
        let err = wallpaper_add("/tmp/missing.png", None).unwrap_err();
        assert!(err.contains("file not found"));
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn display_get_roundtrip_with_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("display-get");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result": {
                "outputs": [{"name": "HDMI-1",
                    "modes": [{"width": 1920, "height": 1080, "refresh": 60},
                              {"width": 1920, "height": 1080, "refresh": 120}],
                    "current": {"width": 1920, "height": 1080, "refresh": 60}}],
                "brightness": 80, "night_light": false}}),
        );
        let state = display_get().unwrap();
        assert_eq!(state.outputs.len(), 1);
        assert_eq!(state.outputs[0].modes.len(), 2);
        assert_eq!(state.brightness, 80);
        assert!(!state.night_light);
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn display_set_roundtrip_and_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("display-set");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"outputs": [], "brightness": 70, "night_light": true}}),
        );
        let state = display_set(None, None, None, None, Some(70.0), Some(true)).unwrap();
        assert_eq!(state.brightness, 70);
        assert!(state.night_light);

        let path = unique_socket("display-set-error");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": false, "error": "display set failed: invalid refresh rate: 5000"}),
        );
        let err = display_set(None, None, None, Some(5000), None, None).unwrap_err();
        assert!(err.contains("invalid refresh rate"));
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn wallpaper_apply_roundtrip() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("wallpaper-apply");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"kind": "premade", "id": "SONOMA", "name": "Sonoma",
                 "path": "/wp/sonoma.png", "path_dark": "/wp/sonoma-dark.png"}}),
        );
        let applied = wallpaper_apply("premade", "SONOMA", "auto").unwrap();
        assert_eq!(applied.id, "SONOMA");
        assert_eq!(applied.path_dark, "/wp/sonoma-dark.png");
        std::env::remove_var("SETTINGS_SOCKET");
    }
}
