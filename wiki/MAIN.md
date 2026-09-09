# SystemSettings – Wiki

SystemSettings is the TontooOS Settings basis: a 900x600 TontooUI window
with a sidebar (single Wi-Fi/WLAN category, blue CoreIcon SF Symbol) and
an example Wi-Fi page on the right. It follows the live system color
scheme and loads `en_us`/`de_de` strings from `lang/`.

- Repository: https://github.com/TontooOS/TontooOS
- License: TCL v26.1
- Version: 0.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| SystemSettings | [SystemSettings.md](SystemSettings.md) | Sidebar layout, Wi-Fi page, daemon backend and localization |

## Quick Start

Run the Settings window from the repository root:

```bash
cargo run
```

The window follows the GNOME system theme live (Dark `#1d1d1d`, Light
`#ececec`) and picks German strings when `LANG` starts with `de`.

See [SystemSettings.md](SystemSettings.md) for details.

## Changelog

- 2026-09-09: Daemon backend wiring (`src/daemon.rs`, public + private
  WiFi ops, no frontend use yet, UI unchanged).
- 2026-09-09: Initial Settings basis (TontooUI Sidebar + Wi-Fi example page, `lang/en_us.json` and `lang/de_de.json`).
