# SystemSettings – Wiki

SystemSettings is the TontooOS Settings app: a 900x600 TontooUI window
built on the Vello renderer. One full-bleed `Sidebar` owns the navigation
column (traffic lights, avatar pill, search field, 28 category rows) and
the detail page of the selected row. Every page is a `ScrollView` over a
header row plus native TontooUI elements (`Form`, `BasicOutlineGroup`,
`Slider`, `BasicSheet`). Strings come from `lang/en_us.json` and
`lang/de_de.json`.

- Repository: https://github.com/TontooOS/TontooOS
- License: TCL v27.0
- Version: 27.0.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| SystemSettings | [SystemSettings.md](SystemSettings.md) | Sidebar, page shell, the 31 pages, sheets and localization |

## Quick Start

Run the Settings window from the repository root:

```bash
cargo run
```

The window follows the live system color scheme through the TontooUI
`ThemeWatcher` and picks German strings when `LANG` starts with `de`.

```rust
// src/main.rs
sdk::preinclude!();

use TontooUI::renderer::window::run;

fn main() {
  lang::init();
  if let Err(err) = run(&lang::t("app.title"), 900, 600, app::SettingsApp::new()) {
    eprintln!("systemsettings: {err}");
    std::process::exit(1);
  }
}
```

See [SystemSettings.md](SystemSettings.md) for details.

## Changelog

- 2026-10-02: Ported to the new TontooUI API. GTK4, UIKit, the
  GTK-backed TontooUI widgets, `gdk-pixbuf` and the `image` crate are
  gone; the app now renders through the Vello scene
  (`run`/`App`/`ThemeWatcher`) with the mandated TontooOS color tokens
  only (`#1b2022`/`#d8d9d9` dark, `#ffffff`/`#272727` light) and native
  `Form` groups instead of hand-built cards. `src/views/root.rs` and the
  22 near-identical GTK page modules collapsed into `src/app.rs`,
  `src/views/mod.rs`, `src/views/simple.rs` and `src/views/sheet.rs`
  (36 files, ~9,500 lines down to 16 files, ~3,000 lines). Navigation
  moved into the `Sidebar` page slots, so the back/forward toolbar is
  gone (the page title lives in the sidebar toolbar row) and the
  sign-in header injection became a native `left_button` avatar pill.
  New: `general.back`, `network.dns.apply`, `wifi.row.locked`,
  `wifi.no_networks` and `appearance.header.subtitle` in both locales.
  Dropped: the Wallpaper "Browse..." button, because TontooLibs has no
  file-open panel yet.
- 2026-09-15: Appearance icon & widget style via the CoreIcon `AppIcon`
  pipeline (Default/Dark/Tinted Light/Tinted Dark with full Liquid
  Glass finish instead of a colored border; new `tinted_light` /
  `tinted_dark` strings in `en_us` / `de_de`).
- 2026-09-12: Appearance page: three theme thumbnails, CoreIcon-rendered
  style icons (Default/Dark/Tinted, no Clear), color picker, all
  display only.
- 2026-09-12: General tab without AutoFill & Passwords, Login Items &
  Extensions and Time Machine (10 rows, groups [3,1,4,2]).
- 2026-09-12: Language & Region page (General row, hidden detail):
  system language EN/DE plus "More soon", full country list, keyboard
  layouts with variants plus Auto Detect, all applied system-wide
  through the daemon `localectl` backend.
- 2026-09-12: Automatic time toggle always on and locked (daemon
  enforces NTP at startup).
- 2026-09-12: About icons fixed at display size; device header uses the
  bundled `Resources/laptop.png` artwork.
- 2026-09-12: Real wired networks: live connected Ethernet list from
  `wired_list` with a "..." info popover.
- 2026-09-12: DNS really works: click-to-edit DNS card applied
  system-wide through the daemon (`dns_get`/`dns_set`).
- 2026-09-12: Wi-Fi header sits in a card like the other pages.
- 2026-09-12: Wi-Fi page wired to the daemon: Known Networks section from
  `wifi_known_list`, live scan, join dialog stores known networks.
- 2026-09-12: About detail page: live hostname, processor, memory,
  kernel and app count, versioned CoreIcon logo and storage usage.
- 2026-09-11: General page rebuilt: centered gear header plus one card
  per row; Displays page rebuilt with brightness, night light and refresh
  rate; Wallpaper page rebuilt with premade grid and apply popup.
- 2026-09-09: The remaining categories (Bluetooth, Battery,
  Accessibility, Desktop & Dock, Menu Bar, Tinti AI, Spotlight,
  Notifications, Sound, Focus, Screen Time, Lock Screen, Privacy, Touch
  ID, Users, Internet Accounts, Octo Cloud, Keyboard, Mouse, Printers,
  App Settings, Developer) added as example pages.
- 2026-09-09: Daemon backend wiring (`src/daemon.rs`).
- 2026-09-09: Initial Settings basis (TontooUI Sidebar + Wi-Fi example
  page, `lang/en_us.json` and `lang/de_de.json`).
