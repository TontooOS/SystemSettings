//! Settings daemon client (WiFi domain plus wallpaper, display and OS state).
//!
//! Covers the public read ops (`wifi_list`, `wifi_status`,
//! `wifi_known_list`, `dns_get`, `wired_list`, `datetime_get`,
//! `locale_get`, `locale_keymap_variants`) and the private write ops
//! (`wifi_connect`, `wifi_disconnect`, `wifi_enable`, `wifi_disable`,
//! `wifi_forget`, `dns_set`, `datetime_set_timezone`, `datetime_set_24h`,
//! `locale_set_language`, `locale_set_region`, `locale_set_keymap`,
//! `locale_set_auto_keymap`) reserved for this app
//! (`com.tontoo.systemsettings`).
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
#[derive(Debug, Clone)]
pub struct WifiState {
    pub enabled: bool,
    pub available: bool,
    pub current: Option<WifiStatus>,
}

/// One stored known network (`wifi_known_list`, public, no passwords).
#[derive(Debug, Clone, Deserialize)]
pub struct KnownNetwork {
    pub ssid: String,
    #[serde(default)]
    pub security: String,
    #[serde(default)]
    pub last_connected: i64,
    #[serde(default)]
    pub auto_join: bool,
}

/// Radio state plus the active connection (`wifi_status`, public).
pub fn status() -> Result<WifiState, String> {
    let result = call("wifi_status", serde_json::json!({}))?;
    let enabled = result
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let available = result
        .get("available")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let current: Option<WifiStatus> =
        serde_json::from_value(result.get("status").cloned().unwrap_or(serde_json::Value::Null))
            .map_err(|e| format!("wifi status invalid: {}", e))?;
    Ok(WifiState {
        enabled,
        available,
        current,
    })
}

/// Stored known networks, most recently connected first
/// (`wifi_known_list`, public).
pub fn known_list() -> Result<Vec<KnownNetwork>, String> {
    let result = call("wifi_known_list", serde_json::json!({}))?;
    serde_json::from_value(result.get("networks").cloned().unwrap_or(serde_json::Value::Null))
        .map_err(|e| format!("wifi known list invalid: {}", e))
}

/// Effective DNS state behind `dns_get` (public). Empty `servers` with
/// `manual == false` means DHCP (automatic).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DnsState {
    #[serde(default)]
    pub servers: Vec<String>,
    #[serde(default)]
    pub manual: bool,
}

/// Read the effective DNS state (`dns_get`, public).
pub fn dns_get() -> Result<DnsState, String> {
    let result = call("dns_get", serde_json::json!({}))?;
    serde_json::from_value(result).map_err(|e| format!("dns get invalid: {}", e))
}

/// Apply DNS servers (`dns_set`, private). Comma/whitespace separated
/// IPv4 addresses; empty clears back to DHCP. Returns the effective state.
pub fn dns_set(servers: &str) -> Result<DnsState, String> {
    let result = call("dns_set", serde_json::json!({"servers": servers}))?;
    serde_json::from_value(result).map_err(|e| format!("dns set invalid: {}", e))
}

/// One connected wired interface with details (`wired_list`, public).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WiredInfo {
    #[serde(default)]
    pub interface: String,
    #[serde(default)]
    pub connection: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub ipv4_addrs: Vec<String>,
    pub gateway: Option<String>,
    pub speed_mbps: Option<u32>,
    pub mtu: Option<u32>,
    pub driver: Option<String>,
    #[serde(default)]
    pub mac: String,
}

/// Connected Ethernet interfaces with details (`wired_list`, public).
pub fn wired_list() -> Result<Vec<WiredInfo>, String> {
    let result = call("wired_list", serde_json::json!({}))?;
    serde_json::from_value(
        result
            .get("interfaces")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .map_err(|e| format!("wired list invalid: {}", e))
}

/// Effective date & time state behind `datetime_get` (public).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DateTimeState {
    #[serde(default)]
    pub ntp: bool,
    #[serde(default)]
    pub timezone: String,
    #[serde(default)]
    pub use_24h: bool,
    #[serde(default)]
    pub timezones: Vec<String>,
}

impl Default for DateTimeState {
    fn default() -> Self {
        Self {
            ntp: true,
            timezone: "UTC".to_string(),
            use_24h: false,
            timezones: vec!["UTC".to_string()],
        }
    }
}

/// Read NTP state, timezone, 24h preference and the zone list
/// (`datetime_get`, public).
pub fn datetime_get() -> Result<DateTimeState, String> {
    let result = call("datetime_get", serde_json::json!({}))?;
    serde_json::from_value(result).map_err(|e| format!("datetime get invalid: {}", e))
}

/// Apply a timezone (`datetime_set_timezone`, private). Returns the
/// effective state.
pub fn datetime_set_timezone(timezone: &str) -> Result<DateTimeState, String> {
    let result = call(
        "datetime_set_timezone",
        serde_json::json!({"timezone": timezone}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("datetime set invalid: {}", e))
}

/// Persist the 24-hour preference (`datetime_set_24h`, private). Returns
/// the effective state.
pub fn datetime_set_24h(use_24h: bool) -> Result<DateTimeState, String> {
    let result = call(
        "datetime_set_24h",
        serde_json::json!({"use_24h": use_24h}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("datetime set invalid: {}", e))
}

/// One language option (`locale_get`, public).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LanguageEntry {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name: String,
}

/// One region option (`locale_get`, public).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RegionEntry {
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub name: String,
}

/// Effective language & region state behind `locale_get` (public).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LocaleState {
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub keymap: String,
    pub keymap_variant: Option<String>,
    #[serde(default)]
    pub auto_keymap: bool,
    #[serde(default)]
    pub languages: Vec<LanguageEntry>,
    #[serde(default)]
    pub regions: Vec<RegionEntry>,
    #[serde(default)]
    pub keymaps: Vec<String>,
}

impl Default for LocaleState {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            region: "US".to_string(),
            keymap: String::new(),
            keymap_variant: None,
            auto_keymap: true,
            languages: vec![
                LanguageEntry { code: "en".to_string(), name: "English".to_string() },
                LanguageEntry { code: "de".to_string(), name: "Deutsch".to_string() },
            ],
            regions: Vec::new(),
            keymaps: Vec::new(),
        }
    }
}

/// Read language, region, keyboard and option lists (`locale_get`, public).
pub fn locale_get() -> Result<LocaleState, String> {
    let result = call("locale_get", serde_json::json!({}))?;
    serde_json::from_value(result).map_err(|e| format!("locale get invalid: {}", e))
}

/// Set the system language (`locale_set_language`, private).
pub fn locale_set_language(language: &str) -> Result<LocaleState, String> {
    let result = call(
        "locale_set_language",
        serde_json::json!({"language": language}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("locale set invalid: {}", e))
}

/// Set the region formats (`locale_set_region`, private).
pub fn locale_set_region(region: &str) -> Result<LocaleState, String> {
    let result = call(
        "locale_set_region",
        serde_json::json!({"region": region}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("locale set invalid: {}", e))
}

/// Set the keyboard layout (`locale_set_keymap`, private). `variant`
/// `None` means the default variant.
pub fn locale_set_keymap(layout: &str, variant: Option<&str>) -> Result<LocaleState, String> {
    let result = call(
        "locale_set_keymap",
        serde_json::json!({"layout": layout, "variant": variant}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("locale set invalid: {}", e))
}

/// Switch keyboard auto-detect (`locale_set_auto_keymap`, private).
pub fn locale_set_auto_keymap(auto: bool) -> Result<LocaleState, String> {
    let result = call(
        "locale_set_auto_keymap",
        serde_json::json!({"auto": auto}),
    )?;
    serde_json::from_value(result).map_err(|e| format!("locale set invalid: {}", e))
}

/// Variants for one keyboard layout (`locale_keymap_variants`, public).
pub fn locale_keymap_variants(layout: &str) -> Result<Vec<String>, String> {
    let result = call(
        "locale_keymap_variants",
        serde_json::json!({"layout": layout}),
    )?;
    serde_json::from_value(
        result
            .get("variants")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .map_err(|e| format!("locale variants invalid: {}", e))
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

/// OS identity facts behind `get_os` (mirrors the daemon `OsInfo`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct OsInfo {
    #[serde(default = "default_os_name")]
    pub name: String,
    #[serde(default = "default_os_display_name")]
    pub display_name: String,
    #[serde(default = "default_os_codename")]
    pub codename: String,
    #[serde(default = "default_os_version")]
    pub version: String,
    #[serde(default)]
    pub beta: bool,
}

fn default_os_name() -> String {
    "TontooOS".to_string()
}

fn default_os_display_name() -> String {
    "TontooOS".to_string()
}

fn default_os_codename() -> String {
    "Seal".to_string()
}

fn default_os_version() -> String {
    "26.1.0".to_string()
}

impl Default for OsInfo {
    fn default() -> Self {
        Self {
            name: default_os_name(),
            display_name: default_os_display_name(),
            codename: default_os_codename(),
            version: default_os_version(),
            beta: false,
        }
    }
}

/// Read the OS identity (`get_os`, public). Falls back to compiled
/// defaults when the daemon is unreachable.
pub fn get_os() -> Result<OsInfo, String> {
    let result = call("get_os", serde_json::json!({}))?;
    serde_json::from_value(result.get("os").cloned().unwrap_or(serde_json::Value::Null))
        .map_err(|e| format!("os info invalid: {}", e))
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
        let state = status().unwrap();
        assert!(state.enabled);
        assert!(state.available);
        assert_eq!(state.current.unwrap().ssid.as_deref(), Some("HomeNet"));

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
    fn status_reports_no_adapter() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("status-no-adapter");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"enabled": false, "status": null, "available": false}}),
        );
        let state = status().unwrap();
        assert!(!state.available);
        assert!(!state.enabled);
        assert!(state.current.is_none());
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn known_list_roundtrip_without_passwords() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("known-list");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result": {"networks": [
                {"ssid": "HomeNet", "security": "WPA2",
                 "last_connected": 1726000000, "auto_join": true},
                {"ssid": "Cafe"},
            ]}}),
        );
        let known = known_list().unwrap();
        assert_eq!(known.len(), 2);
        assert_eq!(known[0].ssid, "HomeNet");
        assert_eq!(known[0].security, "WPA2");
        assert!(known[0].auto_join);
        assert_eq!(known[1].security, "");
        assert!(!known[1].auto_join);
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn dns_get_roundtrip_manual_and_dhcp() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("dns-get");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"servers": ["1.1.1.1", "8.8.8.8"], "manual": true}}),
        );
        let state = dns_get().unwrap();
        assert_eq!(state.servers, vec!["1.1.1.1", "8.8.8.8"]);
        assert!(state.manual);

        let path = unique_socket("dns-get-dhcp");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"servers": [], "manual": false}}),
        );
        let state = dns_get().unwrap();
        assert!(state.servers.is_empty());
        assert!(!state.manual);
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn dns_set_roundtrip_and_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("dns-set");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"servers": ["9.9.9.9"], "manual": true}}),
        );
        let state = dns_set("9.9.9.9").unwrap();
        assert_eq!(state.servers, vec!["9.9.9.9"]);

        let path = unique_socket("dns-set-error");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": false, "error": "dns set failed: invalid IPv4 address: nope"}),
        );
        let err = dns_set("nope").unwrap_err();
        assert!(err.contains("invalid IPv4"));
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn wired_list_roundtrip_with_optional_details() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("wired-list");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result": {"interfaces": [
                {"interface": "eth0", "connection": "Wired connection 1",
                 "state": "connected", "ipv4_addrs": ["192.168.1.5"],
                 "gateway": "192.168.1.1", "mac": "aa:bb:cc:dd:ee:ff",
                 "speed_mbps": 1000, "mtu": 1500, "driver": "e1000e"},
                {"interface": "eth1"},
            ]}}),
        );
        let interfaces = wired_list().unwrap();
        assert_eq!(interfaces.len(), 2);
        assert_eq!(interfaces[0].interface, "eth0");
        assert_eq!(interfaces[0].ipv4_addrs, vec!["192.168.1.5"]);
        assert_eq!(interfaces[0].speed_mbps, Some(1000));
        assert_eq!(interfaces[1].connection, "");
        assert_eq!(interfaces[1].gateway, None);
        assert_eq!(interfaces[1].mtu, None);
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn datetime_get_roundtrip_with_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("datetime-get");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"ntp": true, "timezone": "Europe/Berlin", "use_24h": false,
                 "timezones": ["Europe/Berlin", "UTC"]}}),
        );
        let state = datetime_get().unwrap();
        assert!(state.ntp);
        assert_eq!(state.timezone, "Europe/Berlin");
        assert!(!state.use_24h);
        assert_eq!(state.timezones.len(), 2);
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn datetime_set_roundtrips_and_errors() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("datetime-tz");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"ntp": true, "timezone": "UTC", "use_24h": false,
                 "timezones": ["UTC"]}}),
        );
        let state = datetime_set_timezone("UTC").unwrap();
        assert_eq!(state.timezone, "UTC");

        let path = unique_socket("datetime-24h");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"ntp": true, "timezone": "UTC", "use_24h": true,
                 "timezones": ["UTC"]}}),
        );
        assert!(datetime_set_24h(true).unwrap().use_24h);

        let path = unique_socket("datetime-tz-error");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": false, "error": "datetime set failed: unknown timezone"}),
        );
        let err = datetime_set_timezone("Mars/Olympus_Mons").unwrap_err();
        assert!(err.contains("unknown timezone"));
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn locale_get_roundtrip_with_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("locale-get");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"language": "de", "region": "DE", "keymap": "de",
                 "keymap_variant": null, "auto_keymap": true,
                 "languages": [{"code": "en", "name": "English"},
                               {"code": "de", "name": "Deutsch"}],
                 "regions": [{"code": "DE", "name": "Germany"}],
                 "keymaps": ["de", "us"]}}),
        );
        let state = locale_get().unwrap();
        assert_eq!(state.language, "de");
        assert_eq!(state.region, "DE");
        assert_eq!(state.keymap, "de");
        assert_eq!(state.keymap_variant, None);
        assert!(state.auto_keymap);
        assert_eq!(state.languages.len(), 2);
        assert_eq!(state.keymaps, vec!["de", "us"]);
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn locale_set_roundtrips_and_errors() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("locale-lang");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"language": "de", "region": "US", "keymap": "",
                 "auto_keymap": true, "languages": [], "regions": [],
                 "keymaps": []}}),
        );
        assert_eq!(locale_set_language("de").unwrap().language, "de");

        let path = unique_socket("locale-variants");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"variants": ["nodeadkeys", "deadtilde"]}}),
        );
        assert_eq!(
            locale_keymap_variants("de").unwrap(),
            vec!["nodeadkeys", "deadtilde"]
        );

        let path = unique_socket("locale-lang-error");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": false, "error": "locale set failed: unsupported language"}),
        );
        let err = locale_set_language("fr").unwrap_err();
        assert!(err.contains("unsupported language"));
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn daemon_error_frame_surfaces() {        let _guard = ENV_LOCK.lock().unwrap();
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
    fn get_os_roundtrip_with_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("get-os");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result":
                {"os": {"name": "TontooOS", "display_name": "TontooOS Seal",
                        "codename": "Seal", "version": "26.1.0", "beta": false}}}),
        );
        let info = get_os().unwrap();
        assert_eq!(info.display_name, "TontooOS Seal");
        assert_eq!(info.codename, "Seal");
        assert_eq!(info.version, "26.1.0");
        std::env::remove_var("SETTINGS_SOCKET");
    }

    #[test]
    fn get_os_missing_fields_fall_back_to_defaults() {
        let _guard = ENV_LOCK.lock().unwrap();
        let path = unique_socket("get-os-partial");
        std::env::set_var("SETTINGS_SOCKET", &path);
        serve_once(
            path.clone(),
            serde_json::json!({"id": 1, "ok": true, "result": {"os": {"version": "27.0.0"}}}),
        );
        let info = get_os().unwrap();
        assert_eq!(info.version, "27.0.0");
        assert_eq!(info.codename, "Seal");
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
