# SystemSettings – Wiki

SystemSettings is the TontooOS Settings basis: a 900x600 TontooUI window
with a sidebar (Wi-Fi/WLAN and Network categories, blue CoreIcon SF
Symbols) and the selected detail page on the right. It follows the live
system color scheme and loads `en_us`/`de_de` strings from `lang/`.

- Repository: https://github.com/TontooOS/TontooOS
- License: TCL v26.1
- Version: 0.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| SystemSettings | [SystemSettings.md](SystemSettings.md) | Sidebar layout, Wi-Fi/Network pages, daemon backend and localization |

## Quick Start

Run the Settings window from the repository root:

```bash
cargo run
```

The window follows the GNOME system theme live (Dark `#1d1d1d`, Light
`#ececec`) and picks German strings when `LANG` starts with `de`.

See [SystemSettings.md](SystemSettings.md) for details.

## Changelog

- 2026-09-12: Wi-Fi header sits in a card (shared 12px style) like the
  other pages.
- 2026-09-12: Sign-in header scrolls with the sidebar content (first
  row of the scrollable list; only the search field stays sticky).
- 2026-09-12: Wi-Fi page wired to the daemon: Known Networks section
  from `wifi_known_list`, live scan, no-adapter note under both sections
  (`wifi.no_adapter`), header-only when the radio is off, join dialog
  stores known networks (daemon keeps the encrypted password
  system-wide and auto-joins at startup).
- 2026-09-12: Sign-in avatar as plain SF glyph on a fully round,
  slightly transparent disc (no tile, fixed 40px via exact-size thumb).
- 2026-09-12: Sign-in header below the sidebar search again (with
  the wider spacing kept).
- 2026-09-12: Sign-in header above the sidebar search with more
  spacing (avatar plus bold title and subtitle, display only for now;
  injected in front of the search container since the TontooUI Sidebar
  has no header slot).
- 2026-09-12: About app count covers `/Applications`, every
  `/Users/*/Applications` and `/System/Applications` (files and
  folders ending in `.app`, symlinks deduplicated).
- 2026-09-12: About images fixed-size with rounder corners (device
  tile, OS logo and drive icon render via fixed `Picture`, never
  scaling with the window; logo radius 24px).
- 2026-09-12: About detail page (hidden page behind the General About
  row, history navigation): live hostname, processor, memory, kernel
  and app count, TontooOS card with versioned CoreIcon logo and dynamic
  version/codename from the daemon (`get_os`), storage used/total, no
  buttons. General About row navigates there.
- 2026-09-12: About row uses the plain `questionmark` icon.
- 2026-09-11: General rows grouped into cards (first 3 together,
  AirDrop alone, next 7 together, last 2 alone).
- 2026-09-11: Displays without header (like Wallpaper), controls in a
  card (visible in Light Mode via white cards), brightness uses the
  TontooUI slider.
- 2026-09-11: Displays page rebuilt (output info, live brightness
  slider, night light toggle, refresh rate dropdown from the monitor's
  reported modes up to its max; daemon-wired via `display_get`/
  `display_set` with defaults).
- 2026-09-11: Fill mode applies live (daemon forwards the mode to the
  compositor on dropdown change; no app changes needed).
- 2026-09-11: Current wallpaper thumbnail enlarged (160x100, matches
  the premade thumbs).
- 2026-09-11: General page rebuilt (centered gear header plus one
  card per row: About, Software Update, Storage, AirDrop & Handoff,
  AutoFill & Passwords, Date & Time, Language & Region, Login Items &
  Extensions, Sharing, Startup Disk, Time Machine, Device Management,
  Transfer or Reset; display only, no click actions yet).
- 2026-09-11: Exact-size grid thumbnails (cover-crop from header
  dimensions plus center crop, so cells keep their width and rows flow
  with the window width instead of stacking one per row).
- 2026-09-11: Custom wallpapers back (Browse upload with image
  filters, horizontal custom row, click switches straight to the
  wallpaper with no popup; daemon converts to PNG with unique names).
- 2026-09-11: Instant popup open (Auto split composites from the
  cached small thumbnails instead of decoding the 4K/6K originals;
  single-variant packs show the plain image without a divider).
- 2026-09-11: Wallpaper popup reworked (borderless, centered on the
  app; Light/Auto/Dark previews side by side, all thumbnail-sized;
  click selects with an accent border) and grid cells keep their size
  so rows flow with the window width. The app only talks to the
  settings daemon, never to the compositor directly.
- 2026-09-11: Wallpaper apply popup (click a pack: preview with
  Light/Auto/Dark modes, diagonal split composite for Auto, Cancel/Set;
  Set applies to the desktop via `wallpaper_apply` and refreshes the
  current card).
- 2026-09-11: Wallpaper page rebuilt (current wallpaper card with
  rounded preview, name and fill mode dropdown; premade grid in macOS
  release order, cached thumbnails). Display only except fill mode;
  daemon-wired via `wallpaper_get`/`wallpaper_set_fill` with empty
  fallbacks. Custom uploads come later.
- 2026-09-11: Renamed Siri AI to Tinti AI (`src/views/tinti_ai.rs`,
  `tinti_ai.*` + `sidebar.tinti_ai` + `spotlight.tinti_suggestions` keys in
  all `lang` files, stale top-level `lang/` synced with `Resources/lang/`,
  Spotlight row key fixed).
- 2026-09-09: App Settings category (bundled `launchpad.png` icon after
  an empty gap, example page with Default/Auto Update rows) and
  Developer category (gray `hammer.fill` icon after another gap,
  example page with Mode/Logs rows).
- 2026-09-09: Printers category (gray `printer.fill` icon below Mouse &
  Trackpad, example page with Default/Double-Sided rows).
- 2026-09-09: Keyboard category (gray `keyboard.fill` icon) and Mouse &
  Trackpad category (gray `cursorarrow` icon) after an empty gap, both
  with example pages.
- 2026-09-09: Octo Cloud category (orange `icloud.fill` icon below
  Internet Accounts, example page with Storage/Sync rows).
- 2026-09-09: Internet Accounts category (blue `mail.stack.fill` icon
  after an empty gap, example page with Account/Mail rows).
- 2026-09-09: Touch ID & Password category (pink `touchid` icon) and
  Users & Groups category (gray `person.2.fill` icon) below Privacy,
  both with example pages.
- 2026-09-09: Lock Screen category (black `lock.fill` icon after an
  empty gap) and Privacy & Security category (blue `hand.raised.fill`
  icon), both with example pages.
- 2026-09-09: Focus category (indigo `moon.fill` icon) and Screen Time
  category (indigo `hourglass` icon) below Sound, both with example
  pages.
- 2026-09-09: Sound category (pink `speaker.wave.3.fill` icon below
  Notifications, example page with Output/Mute rows).
- 2026-09-09: Notifications category (red `bell.badge.fill` icon after
  an empty gap, example page with Allow/Sounds toggles).
- 2026-09-09: Wallpaper category (turquoise `atom` icon below Spotlight,
  example page with Current/Auto Change rows).
- 2026-09-09: Spotlight category (gray `magnifyingglass` icon below Siri
  AI, example page with Siri Suggestions/Recent Searches toggles).
- 2026-09-09: Siri AI category (bundled `siri.png` icon below Menu Bar,
  example page with Listen/AI Suggestions toggles).
- 2026-09-09: Fix missing `sidebar.bluetooth` key in all `lang` files
  (sidebar showed the raw key).
- 2026-09-09: Displays category (blue `sun.max.fill` icon) and Menu Bar
  category (gray `switch.2` icon) below Desktop & Dock, both with example
  pages.
- 2026-09-09: Accessibility category (blue `figure.wave.circle` icon
  under General, example page with Display/Reduce Motion rows) and
  Desktop & Dock icon switched to black.
- 2026-09-09: Battery category (green `battery.100` icon below Network,
  same group, example page with Charge/Condition rows).
- 2026-09-09: Detail toolbar (back/forward buttons with history plus the
  current page title above the content).
- 2026-09-09: Desktop & Dock category (blue `menubar.dock.rectangle`
  icon below Appearance, example page with Wallpaper/Show Dock/
  Magnification rows).
- 2026-09-09: Appearance category (bundled PNG icon below General,
  example page with Theme/Accent rows).
- 2026-09-09: General category (gray gear below an empty gap, example
  page with About/Software Update rows).
- 2026-09-09: Bluetooth category (blue antenna icon between Wi-Fi and
  Network, example page with master toggle and device toggles).
- 2026-09-09: Network category (blue `network` icon below Wi-Fi, example
  page with DNS/VPN/wired toggles, in-place detail swap, `src/views/`
  split into `root`/`wifi`/`network`).
- 2026-09-09: Daemon backend wiring (`src/daemon.rs`, public + private
  WiFi ops, no frontend use yet, UI unchanged).
- 2026-09-09: Initial Settings basis (TontooUI Sidebar + Wi-Fi example page, `lang/en_us.json` and `lang/de_de.json`).
