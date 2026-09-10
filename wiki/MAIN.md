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
