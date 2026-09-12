# SystemSettings

TontooOS Settings basis: a 900x600 TontooUI window with a sidebar on the
left (Wi-Fi/WLAN and Network categories, blue CoreIcon SF Symbols) and
the selected detail page on the right. Follows the
live system Dark/Light scheme and loads `en_us`/`de_de` strings from
`lang/`.

## Layout

From left to right the window contains:

1. Sidebar (`Sidebar`, 220px, traffic lights, search, two selectable rows)
2. Detail page (Wi-Fi page or Network example page, swapped in place on
   selection, no app rebuild)

```rust
let mut app = App::with_delegate(lang::t("app.title"), 900, 600, SettingsDelegate);
app.auto_color_scheme(); // live Dark/Light follow
app.run();
```

## Sidebar

`TontooUI::Sidebar` with the `coreicon` feature (default). Both rows use
SF Symbols on a solid blue fill (`Color::from_rgb(0, 122, 255)`), so
CoreIcon generates the icon PNGs at render time. A sign-in header
(avatar plus `sidebar.signin.title`/`sidebar.signin.subtitle`, display
only) is prepended to the scrollable sidebar list at `to_gtk` time, so
it scrolls away with the content while the search field stays sticky;
the Sidebar widget has no header slot.

| Method | Value |
|---|---|
| `item` | `lang::t("sidebar.wifi")` + `SidebarIcon::sf("wifi", blue)` |
| `item` | `lang::t("sidebar.bluetooth")` + `SidebarIcon::sf("antenna.radiowaves.left.and.right", blue)` |
| `item` | `lang::t("sidebar.network")` + `SidebarIcon::sf("network", blue)` |
| `item` | `lang::t("sidebar.battery")` + `SidebarIcon::sf("battery.100", green)` |
| `section` | `""` (empty gap between Network and General) |
| `item` | `lang::t("sidebar.general")` + `SidebarIcon::sf("gear", gray)` |
| `item` | `lang::t("sidebar.accessibility")` + `SidebarIcon::sf("figure.wave.circle", blue)` |
| `item` | `lang::t("sidebar.appearance")` + `SidebarIcon::file(...)` (bundled PNG, used as-is) |
| `item` | `lang::t("sidebar.desktop_dock")` + `SidebarIcon::sf("menubar.dock.rectangle", black)` |
| `item` | `lang::t("sidebar.displays")` + `SidebarIcon::sf("sun.max.fill", blue)` |
| `item` | `lang::t("sidebar.menu_bar")` + `SidebarIcon::sf("switch.2", gray)` |
| `item` | `lang::t("sidebar.tinti_ai")` + `SidebarIcon::file(...)` (bundled PNG, used as-is) |
| `item` | `lang::t("sidebar.spotlight")` + `SidebarIcon::sf("magnifyingglass", gray)` |
| `item` | `lang::t("sidebar.wallpaper")` + `SidebarIcon::sf("atom", teal)` |
| `section` | `""` (empty gap before the Lock Screen group) |
| `item` | `lang::t("sidebar.lock_screen")` + `SidebarIcon::sf("lock.fill", black)` |
| `item` | `lang::t("sidebar.privacy")` + `SidebarIcon::sf("hand.raised.fill", blue)` |
| `item` | `lang::t("sidebar.touch_id")` + `SidebarIcon::sf("touchid", pink)` |
| `item` | `lang::t("sidebar.users")` + `SidebarIcon::sf("person.2.fill", gray)` |
| `section` | `""` (empty gap before the Internet Accounts group) |
| `item` | `lang::t("sidebar.internet_accounts")` + `SidebarIcon::sf("mail.stack.fill", blue)` |
| `item` | `lang::t("sidebar.octo_cloud")` + `SidebarIcon::sf("icloud.fill", orange)` |
| `section` | `""` (empty gap before the Keyboard group) |
| `item` | `lang::t("sidebar.keyboard")` + `SidebarIcon::sf("keyboard.fill", gray)` |
| `item` | `lang::t("sidebar.mouse")` + `SidebarIcon::sf("cursorarrow", gray)` |
| `item` | `lang::t("sidebar.printers")` + `SidebarIcon::sf("printer.fill", gray)` |
| `section` | `""` (empty gap before the App Settings group) |
| `item` | `lang::t("sidebar.app_settings")` + `SidebarIcon::file(...)` (bundled PNG, used as-is) |
| `section` | `""` (empty gap before the Developer group) |
| `item` | `lang::t("sidebar.developer")` + `SidebarIcon::sf("hammer.fill", gray)` |
| `section` | `""` (empty gap before the Notifications group) |
| `item` | `lang::t("sidebar.notifications")` + `SidebarIcon::sf("bell.badge.fill", red)` |
| `item` | `lang::t("sidebar.sound")` + `SidebarIcon::sf("speaker.wave.3.fill", pink)` |
| `item` | `lang::t("sidebar.focus")` + `SidebarIcon::sf("moon.fill", indigo)` |
| `item` | `lang::t("sidebar.screen_time")` + `SidebarIcon::sf("hourglass", indigo)` |
| `selected` | Stored index (survives rebuilds) |
| `search_placeholder` | `lang::t("sidebar.search")` |
| `width` | `220.0` |

`SettingsRoot` (`src/views/root.rs`) stores the sidebar and all detail
pages. GTK widgets are not `Send + Sync`, so the `on_select` handler
(which must be both) only records the index in shared navigation state
(`NavState`: selection plus back/forward history with branching); a
lightweight main-thread poller (100ms) swaps the page, refreshes the
toolbar title and the button sensitivity, and stops itself once its
containers leave the window. The sidebar keeps its own blue highlight.

## Toolbar

Above the detail content sits a toolbar: one joined segment of two
TontooUI `Button`s (text chevrons `‹`/`›`, `Glass` style, half-capsule
each via per-button `.seg-first`/`.seg-last` CSS attached directly to
the buttons, no wrapper background), a separator and
the current page title (bold 15pt). Back walks
the selection history, forward re-enters branched entries; both disable
at the history ends. Layout only, no app design dependency.

## Wi-Fi header

The detail page starts with a header card directly on the screen: the blue
`wifi` SF Symbol icon (rendered by `wifi_icon_path` with the exact sidebar
artwork parameters: solid `#007AFF` fill, white glyph, cached under the
temp dir), the title plus a two-line subtitle, and the toggle pinned to
the top right. The card uses the shared style (`pal.card` background,
12px radius, `12px 16px` padding, like General/About). When icon
generation fails the row degrades to titles plus toggle.

| Key | en_us | de_de |
|---|---|---|
| `wifi.header.subtitle` | `Set up Wi-Fi to wirelessly connect your computer to the internet. Turn on Wi-Fi, then choose a network to join.` | `Richte WLAN ein, um deinen Computer drahtlos mit dem Internet zu verbinden. Schalte WLAN ein und wähle dann ein Netzwerk aus.` |
| `wifi.join.password` | `Password` | `Passwort` |
| `wifi.join.connect` | `Connect` | `Verbinden` |
| `wifi.join.cancel` | `Cancel` | `Abbrechen` |
| `wifi.join.password_required` | `Password required.` | `Passwort erforderlich.` |
| `wifi.known.header` | `Known Networks` | `Bekannte Netzwerke` |
| `wifi.networks.header` | `Networks` | `Netzwerke` |
| `wifi.no_adapter` | `Your Computer doesn't have Wi-Fi` | `Dein Computer hat kein WLAN` |

## Network list

`resolve_state` reads the radio state from the daemon backend
(`src/daemon.rs`, `wifi_status` with `available`/`enabled`):

- No adapter (`available: false`): the Known Networks and Networks
  sections each show the `wifi.no_adapter` note, and the header toggle
  is off and insensitive.
- Radio off: only the header with the toggle stays visible, no sections
  below.
- Radio on: the Known Networks section (from `wifi_known_list`, most
  recently connected first, no signal bars) plus the live scan list
  (`wifi_list`).
- Daemon unreachable: example rows under the Networks header.

The header toggle applies `wifi_enable`/`wifi_disable` and raises a
refresh flag; a `timeout_add_local` poller re-renders the page on the
main thread (the TontooUI toggle handler must be Send + Sync and cannot
touch GTK directly).

Each compact row shows the blue `wifi` icon
(22px), four signal bars for scan rows (filled count from `signal_pct`,
hidden for known rows without a live signal), the SSID
(13pt) and a small gray `lock.fill` badge (14px) when the network is
secured (anything but `OPEN`). Rows sit directly on the screen with a
thin separator, no card behind them. Clicking a row
opens the join dialog: password entry for secured networks (error label
for empty passwords and failed connects), direct connect for open ones.
A successful connect closes the dialog and re-renders the page; the
daemon stores the network as known (encrypted password, system-wide)
and auto-joins it at startup.

## Window bar

There is no system decoration bar: `SettingsRoot` stores the `Sidebar`
and exposes it via `children()`, so UIKit's `hides_window_bar_recursive`
finds it and hides the bar. The traffic lights render directly on the
sidebar, like Apple Settings.

## Colors

All text uses the `SF Pro Display` family, resolved from the system font
paths (`/usr/share/fonts/OTF/SF-Pro-Display-Regular.otf`, etc.). The
content area follows Apple Settings: gray sidebar, contrasting content.

| Token | Dark | Light |
|---|---|---|
| Sidebar | `#1C1C1E` | `#EBEBF0` |
| Content | `#1d1d1d` | `#FFFFFF` |
| Cards | `#2C2C2E` | `#F5F5F7` |
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
| `wifi.networks.header` | `Networks` | `Netzwerke` |
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

## Bluetooth page

Example content (`src/views/bluetooth.rs`): header with the blue
`antenna.radiowaves.left.and.right` icon (no plain `bluetooth` symbol
exists in CoreIcon, same convention as the TontooUI demo), title,
subtitle and a master toggle, then a Devices section (Tontoo Buds on,
Tontoo Mouse off, each with an on/off toggle).

| Key | en_us | de_de |
|---|---|---|
| `sidebar.bluetooth` | `Bluetooth` | `Bluetooth` |
| `bluetooth.title` | `Bluetooth` | `Bluetooth` |
| `bluetooth.header.subtitle` | `Turn on Bluetooth to connect keyboards, headphones and other devices.` | `Schalte Bluetooth ein, um Tastaturen, Kopfhörer und andere Geräte zu verbinden.` |
| `bluetooth.devices.header` | `Devices` | `Geräte` |
| `bluetooth.device.buds` | `Tontoo Buds` | `Tontoo Buds` |
| `bluetooth.device.mouse` | `Tontoo Mouse` | `Tontoo-Maus` |

## Network page

Example content (`src/views/network.rs`): header card with the blue `network`
icon, title, subtitle and a master toggle on the top right (same 1:1
header layout as Wi-Fi and Bluetooth), then the DNS card,
a VPN card with an on/off toggle and a Wired Networks card (Ethernet
on, iPhone USB off, each with an on/off toggle, divider between the
rows). All cards use the shared style (`pal.card` background, 12px
radius, `12px 16px` padding).

The DNS card shows the single big `network.dns` title with the effective
servers (or `network.dns.automatic`) below. Clicking the value turns it
into a text field prefilled with the current servers (`1.1.1.1, 8.8.8.8`
when on DHCP); Enter or leaving the field saves through the daemon
(`dns_set`, empty means DHCP) and the card shows the effective state.
Invalid input shows `network.dns.invalid` plus the daemon error. The
daemon applies the servers to the active NetworkManager connection and
reactivates it, so the change takes effect system-wide immediately.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.network` | `Network` | `Netzwerk` |
| `network.title` | `Network` | `Netzwerk` |
| `network.header.subtitle` | `Manage DNS, VPN and wired connections such as Ethernet or a phone over USB-C.` | `DNS, VPN und kabelgebundene Verbindungen wie Ethernet oder ein Telefon über USB-C verwalten.` |
| `network.dns` | `DNS Server` | `DNS-Server` |
| `network.dns.detail` | `192.168.1.1` | `192.168.1.1` |
| `network.dns.automatic` | `Automatic` | `Automatisch` |
| `network.dns.hint` | `Empty means automatic (DHCP).` | `Leer bedeutet automatisch (DHCP).` |
| `network.dns.invalid` | `Enter valid IPv4 addresses, separated by commas.` | `Gültige IPv4-Adressen eingeben, mit Kommas getrennt.` |
| `network.vpn` | `VPN` | `VPN` |
| `network.wired.header` | `Wired Networks` | `Kabelnetzwerke` |
| `network.wired.ethernet` | `Ethernet` | `Ethernet` |
| `network.wired.iphone` | `iPhone USB` | `iPhone-USB` |

## Battery page

Example content (`src/views/battery.rs`): header with the green
`battery.100` icon (Apple green `(52, 199, 89)`, same as the TontooUI
demo), title and subtitle, then example rows (Charge, Condition with
details).

| Key | en_us | de_de |
|---|---|---|
| `sidebar.battery` | `Battery` | `Batterie` |
| `battery.title` | `Battery` | `Batterie` |
| `battery.header.subtitle` | `Charge level and battery condition.` | `Ladestand und Batteriezustand.` |
| `battery.charge` | `Charge` | `Ladestand` |
| `battery.charge.detail` | `100%` | `100 %` |
| `battery.condition` | `Condition` | `Zustand` |
| `battery.condition.detail` | `Normal` | `Normal` |

## General page

`src/views/general.rs`: centered header card (gear tile, title,
subtitle) plus grouped row cards — first 3 together (About, Software
Update, Storage), AirDrop & Handoff alone, next 7 together (AutoFill &
Passwords through Time Machine), last 2 alone (Device Management,
Transfer or Reset) — each row with a CoreIcon tile, label and chevron.
The About row navigates to the hidden About detail page (history
navigation, back button works); every other row is display only.

## About page

`src/views/about.rs`: hidden detail page behind the General About row
(index 28, reached via history only, no sidebar entry). Device header
(laptop tile plus hostname), device card (Name, Chip from
`/proc/cpuinfo`, Memory from `/proc/meminfo`, Linux kernel release,
installed app count from `/Applications`, every
`/Users/*/Applications` and `/System/Applications` — files and folders
ending in `.app`, symlinks deduplicated), TontooOS card (versioned OS
logo from CoreIcon `OSVersionAssets/<version>/TontooOS_Icon.png` with
rounded corners, daemon display name, dynamic `Version <version>`) and
a storage card (device plus used/total from `df`, no buttons). All values
read live with "Unknown" fallbacks; version and codename come from the
daemon (`get_os`, dynamic, never hardcoded).

| Key | en_us | de_de |
|---|---|---|
| `about.title` | `About` | `Über` |
| `about.name` | `Name` | `Name` |
| `about.chip` | `Chip` | `Chip` |
| `about.memory` | `Memory` | `Arbeitsspeicher` |
| `about.kernel` | `Linux Kernel` | `Linux-Kernel` |
| `about.apps` | `Apps` | `Apps` |
| `about.storage` | `Storage` | `Speicher` |
| `about.unknown` | `Unknown` | `Unbekannt` |
| `about.version` | `Version` | `Version` |

| Key | en_us | de_de |
|---|---|---|
| `sidebar.general` | `General` | `General` |
| `general.title` | `General` | `Allgemein` |
| `general.header.subtitle` | `Manage your overall setup and preferences for TontooOS, such as software updates, device language, AirDrop, and more.` | `Verwalte dein gesamtes Setup und deine Einstellungen für TontooOS, wie Softwareupdates, Gerätesprache, AirDrop und mehr.` |
| `general.about` | `About` | `Info` |
| `general.software_update` | `Software Update` | `Softwareupdate` |
| `general.storage` | `Storage` | `Speicher` |
| `general.airdrop` | `AirDrop & Handoff` | `AirDrop & Handoff` |
| `general.autofill` | `AutoFill & Passwords` | `AutoFill & Passwörter` |
| `general.datetime` | `Date & Time` | `Datum & Uhrzeit` |
| `general.language` | `Language & Region` | `Sprache & Region` |
| `general.login_items` | `Login Items & Extensions` | `Anmeldeobjekte & Erweiterungen` |
| `general.sharing` | `Sharing` | `Freigaben` |
| `general.startup_disk` | `Startup Disk` | `Startvolume` |
| `general.time_machine` | `Time Machine` | `Time Machine` |
| `general.device_management` | `Device Management` | `Geräteverwaltung` |
| `general.transfer_reset` | `Transfer or Reset` | `Übertragen oder Zurücksetzen` |

## Accessibility page

Example content (`src/views/accessibility.rs`): header with the blue
`figure.wave.circle` icon, title and subtitle, then a Display row with
an example value and a Reduce Motion toggle.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.accessibility` | `Accessibility` | `Bedienungshilfen` |
| `accessibility.title` | `Accessibility` | `Bedienungshilfen` |
| `accessibility.header.subtitle` | `Make the screen easier to see and use.` | `Den Bildschirm leichter sehen und bedienen.` |
| `accessibility.display` | `Display` | `Anzeige` |
| `accessibility.display.detail` | `Default` | `Standard` |
| `accessibility.reduce_motion` | `Reduce Motion` | `Bewegung reduzieren` |

## Appearance page

Example content (`src/views/appearance.rs`): header with the bundled PNG
icon (`Resources/mf4of5ol1b5a1inx0512nn8mq6wd.png`, sidebar via
`SidebarIcon::file`, header via `gtk::Image` directly), title and
subtitle, then example rows (Theme, Accent Color with details).

| Key | en_us | de_de |
|---|---|---|
| `sidebar.appearance` | `Appearance` | `Erscheinungsbild` |
| `appearance.title` | `Appearance` | `Erscheinungsbild` |
| `appearance.header.subtitle` | `Choose the look of windows, buttons and controls.` | `Wähle das Aussehen von Fenstern, Knöpfen und Bedienelementen.` |
| `appearance.theme` | `Theme` | `Farbschema` |
| `appearance.theme.detail` | `Automatic` | `Automatisch` |
| `appearance.accent` | `Accent Color` | `Akzentfarbe` |
| `appearance.accent.detail` | `Orange` | `Orange` |

## Desktop & Dock page

Example content (`src/views/desktop_dock.rs`): header with the blue
`menubar.dock.rectangle` icon, title and subtitle, then a Wallpaper row
with an example name plus Show Dock (on) and Magnification (off) toggles.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.desktop_dock` | `Desktop & Dock` | `Schreibtisch & Dock` |
| `desktop_dock.title` | `Desktop & Dock` | `Schreibtisch & Dock` |
| `desktop_dock.header.subtitle` | `Manage the wallpaper and the Dock.` | `Hintergrundbild und Dock verwalten.` |
| `desktop_dock.wallpaper` | `Wallpaper` | `Hintergrundbild` |
| `desktop_dock.wallpaper.detail` | `Tontoo Reef` | `Tontoo-Riff` |
| `desktop_dock.show_dock` | `Show Dock` | `Dock einblenden` |
| `desktop_dock.magnification` | `Magnification` | `Vergrößerung` |

## Displays page

`src/views/displays.rs`: one card with output info plus live controls
— brightness slider (TontooUI `Slider`, dims the whole desktop in the
compositor), night light toggle (warm overlay) and a refresh rate
dropdown built from the monitor's reported modes. No title header (like
the Wallpaper page, the toolbar shows the title). All values come from
`display_get` with defaults when the daemon is unreachable; every
change applies live via `display_set` and persists there.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.displays` | `Displays` | `Monitore` |
| `displays.title` | `Displays` | `Monitore` |
| `displays.header.subtitle` | `Adjust brightness, refresh rate and night light.` | `Helligkeit, Bildwiederholrate und Night Light anpassen.` |
| `displays.output` | `Display` | `Monitor` |
| `displays.no_output` | `No display found` | `Kein Monitor gefunden` |
| `displays.brightness` | `Brightness` | `Helligkeit` |
| `displays.night_light` | `Night Light` | `Night Light` |
| `displays.refresh_rate` | `Refresh Rate` | `Bildwiederholrate` |

## Menu Bar page

Example content (`src/views/menu_bar.rs`): header with the gray
`switch.2` icon (Apple-gray like General), title and subtitle, then
Clock (on) and Spotlight (off) toggles.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.menu_bar` | `Menu Bar` | `Menüleiste` |
| `menu_bar.title` | `Menu Bar` | `Menüleiste` |
| `menu_bar.header.subtitle` | `Customize the menu bar.` | `Menüleiste anpassen.` |
| `menu_bar.clock` | `Clock` | `Uhr` |
| `menu_bar.spotlight` | `Spotlight` | `Spotlight` |

## Tinti AI page

Example content (`src/views/tinti_ai.rs`): header with the bundled PNG
icon (`Resources/tinti.png`, sidebar via `SidebarIcon::file`, header via
`gtk::Image` directly), title and subtitle, then Listen for Tinti (on)
and AI Suggestions (on) toggles.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.tinti_ai` | `Tinti AI` | `Tinti AI` |
| `tinti_ai.title` | `Tinti AI` | `Tinti AI` |
| `tinti_ai.header.subtitle` | `Talk to Tinti and get intelligent suggestions.` | `Sprich mit Tinti und erhalte intelligente Vorschläge.` |
| `tinti_ai.listen` | `Listen for Tinti` | `Auf Tinti hören` |
| `tinti_ai.suggestions` | `AI Suggestions` | `KI-Vorschläge` |

## Spotlight page

Example content (`src/views/spotlight.rs`): header with the gray
`magnifyingglass` icon (Apple-gray like General), title and subtitle,
then Tinti Suggestions (on) and Recent Searches (off) toggles.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.spotlight` | `Spotlight` | `Spotlight` |
| `spotlight.title` | `Spotlight` | `Spotlight` |
| `spotlight.header.subtitle` | `Search apps, files and the web.` | `Apps, Dateien und das Web durchsuchen.` |
| `spotlight.tinti_suggestions` | `Tinti Suggestions` | `Tinti-Vorschläge` |
| `spotlight.recents` | `Recent Searches` | `Letzte Suchanfragen` |

## Wallpaper page

`src/views/wallpaper.rs`: current wallpaper card (rounded preview
thumbnail, name, fill mode dropdown) plus an available wallpapers card
(Browse button, horizontal custom row, clickable premade grid in macOS
release order; cells keep their size so rows flow with the window
width). Rounded thumbnails come from small cached files
(`gdk-pixbuf` scale-on-load into the temp dir), the 4K/6K originals are
never loaded into the UI. Custom cells switch straight to the wallpaper
on click (no popup; Browse uploads convert to PNG with unique names);
premade cells open a borderless popup centered on the Settings window
with Light/Auto/Dark previews side by side, click selects with an
orange accent border, Cancel and Set at the bottom. Set applies to the
desktop via `wallpaper_apply` and refreshes the current card. The app
only ever talks to the settings daemon, never to the compositor
directly; the daemon persists the selection. All data comes from
`wallpaper_get` with empty fallbacks when the daemon is unreachable.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.wallpaper` | `Wallpaper` | `Hintergrundsbild` |
| `wallpaper.title` | `Wallpaper` | `Hintergrundsbild` |
| `wallpaper.header.subtitle` | `Choose the desktop background.` | `Schreibtischhintergrund wählen.` |
| `wallpaper.current` | `Current Wallpaper` | `Aktuelles Hintergrundbild` |
| `wallpaper.no_wallpaper` | `No wallpaper set` | `Kein Hintergrundbild festgelegt` |
| `wallpaper.fill_mode` | `Fill Mode` | `Füllmodus` |
| `wallpaper.fill.fill` | `Fill screen` | `Bildschirm füllen` |
| `wallpaper.fill.fit` | `Fit to screen` | `An Bildschirm anpassen` |
| `wallpaper.fill.stretch` | `Stretch to Fill Screen` | `Auf Bildschirm strecken` |
| `wallpaper.fill.center` | `Center` | `Zentrieren` |
| `wallpaper.fill.tile` | `Tile` | `Kacheln` |
| `wallpaper.available` | `Available Wallpapers` | `Verfügbare Hintergrundbilder` |
| `wallpaper.browse` | `Browse...` | `Durchsuchen …` |
| `wallpaper.open` | `Open` | `Öffnen` |
| `wallpaper.all_images` | `All images` | `Alle Bilder` |
| `wallpaper.no_wallpapers` | `No wallpapers found.` | `Keine Hintergrundbilder gefunden.` |
| `wallpaper.custom` | `Custom` | `Eigene` |
| `wallpaper.premade` | `Premade` | `Vorinstalliert` |
| `wallpaper.set` | `Set` | `Setzen` |
| `wallpaper.cancel` | `Cancel` | `Abbrechen` |
| `wallpaper.mode.light` | `Light` | `Hell` |
| `wallpaper.mode.auto` | `Auto` | `Auto` |
| `wallpaper.mode.dark` | `Dark` | `Dunkel` |

## Notifications page

Example content (`src/views/notifications.rs`): header with the red
`bell.badge.fill` icon (`(255, 69, 58)`), title and subtitle, then Allow
Notifications (on) and Sounds (on) toggles.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.notifications` | `Notifications` | `Mitteilungen` |
| `notifications.title` | `Notifications` | `Mitteilungen` |
| `notifications.header.subtitle` | `Choose which apps notify you and how.` | `Wähle, welche Apps dich benachrichtigen und wie.` |
| `notifications.allow` | `Allow Notifications` | `Mitteilungen erlauben` |
| `notifications.sounds` | `Sounds` | `Töne` |

## Sound page

Example content (`src/views/sound.rs`): header with the pink
`speaker.wave.3.fill` icon (`(255, 45, 85)`), title and subtitle, then
an Output row with an example device plus a Mute toggle.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.sound` | `Sound` | `Ton` |
| `sound.title` | `Sound` | `Ton` |
| `sound.header.subtitle` | `Adjust output volume and alerts.` | `Lautstärke und Hinweistöne anpassen.` |
| `sound.output` | `Output` | `Ausgabe` |
| `sound.output.detail` | `Speakers` | `Lautsprecher` |
| `sound.mute` | `Mute` | `Stumm` |

## Focus page

Example content (`src/views/focus.rs`): header with the indigo
`moon.fill` icon (`(88, 86, 214)`), title and subtitle, then Do Not
Disturb (on) and Sleep (off) toggles.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.focus` | `Focus` | `Fokus` |
| `focus.title` | `Focus` | `Fokus` |
| `focus.header.subtitle` | `Silence notifications when you need to concentrate.` | `Mitteilungen stummschalten, wenn du dich konzentrieren musst.` |
| `focus.dnd` | `Do Not Disturb` | `Nicht stören` |
| `focus.sleep` | `Sleep` | `Schlaf` |

## Screen Time page

Example content (`src/views/screen_time.rs`): header with the indigo
`hourglass` icon (`(88, 86, 214)`), title and subtitle, then a Downtime
toggle (off) plus an App Limits row.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.screen_time` | `Screen Time` | `Bildschirmzeit` |
| `screen_time.title` | `Screen Time` | `Bildschirmzeit` |
| `screen_time.header.subtitle` | `See app usage and set limits.` | `App-Nutzung sehen und Limits setzen.` |
| `screen_time.downtime` | `Downtime` | `Auszeit` |
| `screen_time.app_limits` | `App Limits` | `App-Limits` |
| `screen_time.app_limits.detail` | `None` | `Keine` |

## Lock Screen page

Example content (`src/views/lock_screen.rs`): header with the black
`lock.fill` icon (`(0, 0, 0)`), title and subtitle, then a Require
Password toggle (on) plus a Screen Saver row.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.lock_screen` | `Lock Screen` | `Sperrbildschirm` |
| `lock_screen.title` | `Lock Screen` | `Sperrbildschirm` |
| `lock_screen.header.subtitle` | `Secure your computer when you step away.` | `Computer sichern, wenn du weggehst.` |
| `lock_screen.require_password` | `Require Password` | `Passwort anfordern` |
| `lock_screen.screen_saver` | `Screen Saver` | `Bildschirmschoner` |
| `lock_screen.screen_saver.detail` | `5 Minutes` | `5 Minuten` |

## Privacy & Security page

Example content (`src/views/privacy.rs`): header with the blue
`hand.raised.fill` icon, title and subtitle, then a Location Services
toggle (on) plus an App Permissions row.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.privacy` | `Privacy & Security` | `Datenschutz & Sicherheit` |
| `privacy.title` | `Privacy & Security` | `Datenschutz & Sicherheit` |
| `privacy.header.subtitle` | `Control how apps access your data.` | `Steuern, wie Apps auf deine Daten zugreifen.` |
| `privacy.location` | `Location Services` | `Ortungsdienste` |
| `privacy.permissions` | `App Permissions` | `App-Berechtigungen` |
| `privacy.permissions.detail` | `12 Apps` | `12 Apps` |

## Touch ID & Password page

Example content (`src/views/touch_id.rs`): header with the pink
`touchid` icon (`(255, 45, 85)`), title and subtitle, then an Unlock
with Touch ID toggle (on) plus a Passwords row.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.touch_id` | `Touch ID & Password` | `Touch ID & Passwort` |
| `touch_id.title` | `Touch ID & Password` | `Touch ID & Passwort` |
| `touch_id.header.subtitle` | `Unlock with your fingerprint and manage passwords.` | `Mit Fingerabdruck entsperren und Passwörter verwalten.` |
| `touch_id.unlock` | `Unlock with Touch ID` | `Mit Touch ID entsperren` |
| `touch_id.passwords` | `Passwords` | `Passwörter` |
| `touch_id.passwords.detail` | `3 Saved` | `3 gespeichert` |

## Users & Groups page

Example content (`src/views/users.rs`): header with the gray
`person.2.fill` icon (Apple-gray like General), title and subtitle,
then a Current User row plus a Guest User toggle (off).

| Key | en_us | de_de |
|---|---|---|
| `sidebar.users` | `Users & Groups` | `Benutzer & Gruppen` |
| `users.title` | `Users & Groups` | `Benutzer & Gruppen` |
| `users.header.subtitle` | `Manage the users of this computer.` | `Benutzer dieses Computers verwalten.` |
| `users.current` | `Current User` | `Aktueller Benutzer` |
| `users.current.detail` | `Admin` | `Admin` |
| `users.guest` | `Guest User` | `Gastbenutzer` |

## Internet Accounts page

Example content (`src/views/internet_accounts.rs`): header with the blue
`mail.stack.fill` icon, title and subtitle, then a Tontoo Account row
plus a Mail toggle (on).

| Key | en_us | de_de |
|---|---|---|
| `sidebar.internet_accounts` | `Internet Accounts` | `Internetaccounts` |
| `internet_accounts.title` | `Internet Accounts` | `Internetaccounts` |
| `internet_accounts.header.subtitle` | `Connect mail, contacts and calendars.` | `Mail, Kontakte und Kalender verbinden.` |
| `internet_accounts.account` | `Tontoo Account` | `Tontoo-Account` |
| `internet_accounts.account.detail` | `Signed In` | `Angemeldet` |
| `internet_accounts.mail` | `Mail` | `Mail` |

## Octo Cloud page

Example content (`src/views/octo_cloud.rs`): header with the orange
`icloud.fill` icon (`(255, 107, 43)`), title and subtitle, then a
Storage row plus a Sync toggle (on).

| Key | en_us | de_de |
|---|---|---|
| `sidebar.octo_cloud` | `Octo Cloud` | `Octo Cloud` |
| `octo_cloud.title` | `Octo Cloud` | `Octo Cloud` |
| `octo_cloud.header.subtitle` | `Sync notes, podcasts and more across devices.` | `Notizen, Podcasts und mehr geräteübergreifend synchronisieren.` |
| `octo_cloud.storage` | `Storage` | `Speicher` |
| `octo_cloud.storage.detail` | `128 GB of 1 TB` | `128 GB von 1 TB` |
| `octo_cloud.sync` | `Sync` | `Synchronisieren` |

## Keyboard page

Example content (`src/views/keyboard.rs`): header with the gray
`keyboard.fill` icon (Apple-gray like General), title and subtitle,
then a Key Repeat toggle (on) plus a Shortcuts row.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.keyboard` | `Keyboard` | `Tastatur` |
| `keyboard.title` | `Keyboard` | `Tastatur` |
| `keyboard.header.subtitle` | `Set typing behavior and shortcuts.` | `Tippverhalten und Kurzbefehle festlegen.` |
| `keyboard.key_repeat` | `Key Repeat` | `Tastenwiederholung` |
| `keyboard.shortcuts` | `Shortcuts` | `Kurzbefehle` |
| `keyboard.shortcuts.detail` | `12 Defined` | `12 definiert` |

## Mouse & Trackpad page

Example content (`src/views/mouse.rs`): header with the gray
`cursorarrow` icon (Apple-gray like General), title and subtitle, then
a Tracking Speed row plus a Natural Scroll toggle (on).

| Key | en_us | de_de |
|---|---|---|
| `sidebar.mouse` | `Mouse & Trackpad` | `Maus & Trackpad` |
| `mouse.title` | `Mouse & Trackpad` | `Maus & Trackpad` |
| `mouse.header.subtitle` | `Adjust pointing and clicking.` | `Zeigen und Klicken anpassen.` |
| `mouse.tracking` | `Tracking Speed` | `Zeigergeschwindigkeit` |
| `mouse.tracking.detail` | `Fast` | `Schnell` |
| `mouse.natural_scroll` | `Natural Scroll` | `Natürliches Scrollen` |

## Printers page

Example content (`src/views/printers.rs`): header with the gray
`printer.fill` icon (Apple-gray like General), title and subtitle, then
a Default Printer row plus a Double-Sided toggle (on).

| Key | en_us | de_de |
|---|---|---|
| `sidebar.printers` | `Printers` | `Drucker` |
| `printers.title` | `Printers` | `Drucker` |
| `printers.header.subtitle` | `Add printers and manage print jobs.` | `Drucker hinzufügen und Druckaufträge verwalten.` |
| `printers.default` | `Default Printer` | `Standarddrucker` |
| `printers.default.detail` | `Tontoo Laser` | `Tontoo Laser` |
| `printers.double_sided` | `Double-Sided` | `Beidseitig` |

## App Settings page

Example content (`src/views/app_settings.rs`): header with the bundled
Launchpad PNG (`Resources/launchpad.png`, sidebar via
`SidebarIcon::file`, header via `gtk::Image` directly), title and
subtitle, then a Default Apps row plus an Auto Update toggle (on).

| Key | en_us | de_de |
|---|---|---|
| `sidebar.app_settings` | `App Settings` | `App-Einstellungen` |
| `app_settings.title` | `App Settings` | `App-Einstellungen` |
| `app_settings.header.subtitle` | `Manage installed apps and defaults.` | `Installierte Apps und Standards verwalten.` |
| `app_settings.default_apps` | `Default Apps` | `Standard-Apps` |
| `app_settings.default_apps.detail` | `Tontoo Apps` | `Tontoo-Apps` |
| `app_settings.auto_update` | `Auto Update` | `Automatische Updates` |

## Developer page

Example content (`src/views/developer.rs`): header with the gray
`hammer.fill` icon (Apple-gray like General), title and subtitle, then
a Developer Mode toggle (off) plus an API Logs row.

| Key | en_us | de_de |
|---|---|---|
| `sidebar.developer` | `Developer` | `Entwickler` |
| `developer.title` | `Developer` | `Entwickler` |
| `developer.header.subtitle` | `Tools for developers.` | `Werkzeuge für Entwickler.` |
| `developer.mode` | `Developer Mode` | `Entwicklermodus` |
| `developer.api_logs` | `API Logs` | `API-Protokolle` |
| `developer.api_logs.detail` | `Minimal` | `Minimal` |

## Daemon backend

`src/daemon.rs` wires the app to the settings daemon over its unix socket
(`SETTINGS_SOCKET` override, else `/run/tontoo-settings.sock`). It covers
the public read ops (`wifi_list`, `wifi_status`, `wifi_known_list`,
`dns_get`, `wallpaper_get`,
`display_get`, `get_os`) and the
private write ops (`wifi_connect`, `wifi_disconnect`, `wifi_enable`,
`wifi_disable`, `wifi_forget`, `dns_set`, `wallpaper_set_current`,
`wallpaper_set_fill`, `wallpaper_add`) reserved for this app
(`com.tontoo.systemsettings`).

```rust
pub fn list() -> Result<Vec<WifiNetwork>, String>
pub fn status() -> Result<WifiState, String>
pub fn known_list() -> Result<Vec<KnownNetwork>, String>
pub fn connect(ssid: &str, password: Option<&str>, hidden: bool) -> Result<WifiStatus, String>
pub fn disconnect() -> Result<(), String>
pub fn set_enabled(enabled: bool) -> Result<(), String>
pub fn forget(ssid: &str) -> Result<bool, String>
pub fn dns_get() -> Result<DnsState, String>
pub fn dns_set(servers: &str) -> Result<DnsState, String>
pub fn wallpaper_get() -> Result<WallpaperState, String>
pub fn wallpaper_set_current(kind: &str, id: &str) -> Result<Option<WallpaperEntry>, String>
pub fn wallpaper_set_fill(fill: &str) -> Result<String, String>
pub fn wallpaper_add(path: &str, name: Option<&str>) -> Result<WallpaperEntry, String>
pub fn wallpaper_apply(kind: &str, id: &str, variant: &str) -> Result<WallpaperEntry, String>
pub fn get_os() -> Result<OsInfo, String>
pub fn display_get() -> Result<DisplayState, String>
pub fn display_set(output: Option<&str>, width: Option<i32>, height: Option<i32>, refresh: Option<u32>, brightness: Option<f64>, night_light: Option<bool>) -> Result<DisplayState, String>
```

Rules:

- The Wallpaper, Displays and Wi-Fi pages are daemon-wired; `WifiState`
  carries `enabled`, `available` (false without a wireless adapter) and
  the current connection. Known networks come from `wifi_known_list`
  (never passwords).
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
