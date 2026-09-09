# SystemSettings

TontooOS Settings basis: a 900x600 TontooUI window with a sidebar on the
left (single Wi-Fi/WLAN category, blue CoreIcon SF Symbol) and an example
Wi-Fi page on the right (title, toggle, example network list). Follows the
live system Dark/Light scheme and loads `en_us`/`de_de` strings from
`lang/`.

## Layout

From left to right the window contains:

1. Sidebar (`Sidebar`, 220px, traffic lights, search, one selectable row)
2. Wi-Fi example page (header row with icon, title, subtitle and toggle,
   plus an `InsetGrouped` list)

```rust
let mut app = App::with_delegate(lang::t("app.title"), 900, 600, SettingsDelegate);
app.auto_color_scheme(); // live Dark/Light follow
app.run();
```

## Sidebar

`TontooUI::Sidebar` with the `coreicon` feature (default). The single row
uses the SF Symbol `wifi.circle.fill` on a solid blue fill
(`Color::from_rgb(0, 122, 255)`), so CoreIcon generates the icon PNG at
render time.

| Method | Value |
|---|---|
| `item` | `lang::t("sidebar.wifi")` + `SidebarIcon::sf("wifi.circle.fill", blue)` |
| `selected` | `0` (single category, always highlighted) |
| `search_placeholder` | `lang::t("sidebar.search")` |
| `width` | `220.0` |

## Wi-Fi header

The detail page starts with a header row: the blue `wifi` SF Symbol icon
(rendered by `wifi_icon_path` with the exact sidebar artwork parameters:
solid `#007AFF` fill, white glyph, cached under the temp dir), the title
plus a two-line subtitle, and the toggle on the right. When icon
generation fails the row degrades to titles plus toggle.

| Key | en_us | de_de |
|---|---|---|
| `wifi.header.subtitle` | `Set up Wi-Fi to wirelessly connect your computer to the internet. Turn on Wi-Fi, then choose a network to join.` | `Richte WLAN ein, um deinen Computer drahtlos mit dem Internet zu verbinden. Schalte WLAN ein und wähle dann ein Netzwerk aus.` |

## Colors

All text uses the `SF Pro Display` family, resolved from the system font
paths (`/usr/share/fonts/OTF/SF-Pro-Display-Regular.otf`, etc.).

| Token | Dark | Light |
|---|---|---|
| Background | `#1d1d1d` | `#ececec` |
| Primary text | `#F5F5F7` | `#1E1E1E` |
| Secondary text | `#A1A1A6` | `#6E6E73` |
| Wi-Fi icon | `#007AFF` | `#007AFF` |

The scheme is read from `uikit::app::current_color_scheme()` with a
`ColorScheme::detect_system()` fallback, so the window matches the live
system theme on every rebuild.

## Localization

Strings live in `lang/en_us.json` and `lang/de_de.json` (only these
two). `src/lang.rs` detects German from `LANGUAGE`, `LC_ALL`, `LANG`
or `/etc/locale.conf` and falls back to `en_us`.

`Resources/lang/` holds copies of both files: TBuild copies only
`Resources/` into the `.app` bundle (root `lang/` is used just for the
localized `name` in `Info.tontoo`). Keep both locations in sync.

| Key | en_us | de_de |
|---|---|---|
| `app.title` | `Settings` | `Einstellungen` |
| `sidebar.search` | `Search` | `Suchen` |
| `sidebar.wifi` | `Wi-Fi` | `WLAN` |
| `wifi.title` | `Wi-Fi` | `WLAN` |
| `wifi.toggle` | `Wi-Fi` | `WLAN` |
| `wifi.networks.header` | `Known Networks` | `Bekannte Netzwerke` |
| `wifi.example.note` | `Example content: connect to a network to get started.` | `Beispielinhalt: Verbinde dich mit einem Netzwerk, um zu starten.` |
| `wifi.row.home` | `HomeNet` | `HeimNetz` |
| `wifi.row.home.detail` | `Connected` | `Verbunden` |
| `wifi.row.lab` | `TontooLab` | `TontooLabor` |
| `wifi.row.lab.detail` | `Secured` | `Gesichert` |

### `t(key)`

```rust
pub fn t(key: &str) -> String
```

Returns the localized string for `key`. Returns the key itself when the
locale file or key is missing, so the UI never renders empty text.

## Usage / Example

```bash
cargo run
LANG=de_DE.UTF-8 cargo run
```

The first command shows English strings (`Wi-Fi`), the second German
strings (`WLAN`).

## Daemon backend

`src/daemon.rs` wires the app to the settings daemon over its unix socket
(`SETTINGS_SOCKET` override, else `/run/tontoo-settings.sock`). It covers
the public read ops (`wifi_list`, `wifi_status`) and the private write ops
(`wifi_connect`, `wifi_disconnect`, `wifi_enable`, `wifi_disable`,
`wifi_forget`) reserved for this app (`com.tontoo.systemsettings`).

```rust
pub fn list() -> Result<Vec<WifiNetwork>, String>
pub fn status() -> Result<(bool, Option<WifiStatus>), String>
pub fn connect(ssid: &str, password: Option<&str>, hidden: bool) -> Result<WifiStatus, String>
pub fn disconnect() -> Result<(), String>
pub fn set_enabled(enabled: bool) -> Result<(), String>
pub fn forget(ssid: &str) -> Result<bool, String>
```

Rules:

- Backend wiring only: no UI code uses this module yet, the Wi-Fi page
  keeps showing example content until the frontend step connects it.
- Missing or unreachable sockets return `Err`, never partial data.

## Packaging

`tontoo.proj` (`bundle_id: com.tontoo.systemsettings`) lets TBuild
assemble the `.app` bundle:

```bash
tbuild app /path/to/SystemSettings
```

The bundle contains the release binary (`App/`), the icon
(`Resources/app_icon.png`) and `Resources/lang/` (`lang/`). The runtime
lookup covers the bundle layout
(`<Name>.app/Resources/lang`), dev checkouts (`lang/`,
`Resources/`) and installed files (`/usr/share/systemsettings/`).

## Cross References

- [MAIN.md](MAIN.md) -- wiki entry point
- TontooUI [Sidebar](https://github.com/TontooOS/TontooOS) -- sidebar with traffic lights and item list
- CoreIcon SF Symbol `wifi.circle.fill` -- blue Wi-Fi category icon
