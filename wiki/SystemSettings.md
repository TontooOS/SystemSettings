# SystemSettings

TontooOS Settings app on the TontooUI renderer: a 900x600 window whose
navigation column and detail pages are both owned by one `Sidebar`. All
pages follow the same shell, follow the live system color scheme through
the `ThemeWatcher`, use the mandated TontooOS background/text tokens and
load `en_us`/`de_de` strings from `lang/`.

## Architecture

```
src/
├── main.rs          entry point: lang::init() + TontooUI::renderer::window::run
├── app.rs           SettingsApp: Sidebar, pages, ThemeWatcher, sheet host
├── daemon.rs        JSON-over-unix-socket client for the settings daemon
├── lang.rs          locale store for lang/en_us.json and lang/de_de.json
└── views/
    ├── mod.rs       page shell, Skin, navigation table, Nav, SharedPage
    ├── sheet.rs     one modal host: Join, Wallpaper, Notice
    ├── simple.rs    22 table-driven placeholder categories
    ├── wifi.rs      radio switch, known networks, live scan, join sheet
    ├── network.rs   DNS field plus wired interface groups
    ├── general.rs   navigation list plus the three pushed details
    ├── appearance.rs theme/style/color previews (display only)
    ├── displays.rs  output info, brightness slider, refresh rate
    ├── wallpaper.rs current wallpaper, fill mode, apply sheet
    ├── about.rs     device, TontooOS and storage groups
    ├── datetime.rs  date/time, 24-hour switch, timezone dropdown
    └── locale.rs    languages, region, keyboard layout and variants
```

The old GTK build needed a page-swap poller, per-page `timeout_add_local`
timers and a `Send + Sync` dance for every widget callback. The Vello
renderer has none of that: callbacks are `Box<dyn FnMut>`, the frame
clock is `App::draw(..., time_secs)`, and the `Sidebar` already draws the
selected page. That removed roughly 6,500 lines of plumbing.

## Window

The window is full bleed. There is no titlebar: the `Sidebar` owns the
traffic lights and draws them over its own column, exactly like Apple
Settings. The page title therefore lives in the sidebar toolbar row,
which TontooUI fills with the selected row's label (or whatever
`set_title` overrides).

```rust
// src/main.rs
mod app;
mod daemon;
mod lang;
mod views;

sdk::preinclude!();

use TontooUI::renderer::window::run;

fn main() {
  lang::init();
  if let Err(err) = run(
    &lang::t("app.title"),
    views::WINDOW_W,
    views::WINDOW_H,
    app::SettingsApp::new(),
  ) {
    eprintln!("systemsettings: {err}");
    std::process::exit(1);
  }
}
```

### `SettingsApp`

```rust
pub struct SettingsApp { /* private */ }

impl SettingsApp {
    pub fn new() -> Self
}

impl TontooUI::renderer::window::App for SettingsApp {
    fn draw(&mut self, scene, fonts, images, viewport, time_secs);
    fn background(&self) -> Color;
    fn wants_backdrop(&self) -> bool;
    fn drag_region(&self) -> Option<(f32, f32, f32, f32)>;
    fn cursor(&self, x: f64, y: f64) -> CursorKind;
    fn mouse_down(&mut self, x: f64, y: f64);
    fn mouse_up(&mut self, x: f64, y: f64);
    fn mouse_move(&mut self, x: f64, y: f64);
    fn mouse_wheel(&mut self, dx: f64, dy: f64);
    fn set_focused(&mut self, focused: bool);
    fn text(&mut self, text: &str);
    fn key(&mut self, key: Key);
}
```

`draw` runs the whole per-frame contract:

1. `ThemeWatcher::poll` / `palette` refresh the `Skin`; a theme flip
   schedules a rebuild.
2. `poll_wifi` reads the live radio state every 2 seconds and compares a
   coarse fingerprint (SSID sets, no signal percentages), so a running
   scan never resets the scroll position.
3. `pump` turns queued page requests into a sheet and applies a finished
   sheet to the daemon.
4. `rebuild` re-runs every page builder and hands the pages to a fresh
   `Sidebar`, keeping the column width, the collapse state and the search
   query.
5. `Sidebar::place` + `Sidebar::draw` draw the column and the selected
   page; the sheet draws last when open.

Input order matters and matches the TontooUI contract: while a sheet is
open it consumes every event, then the traffic lights, then the sidebar
(which forwards the rest into the page).

## Page shell

Every page is built by `page_shell`, so `PageView::form_mut` and the
theme walker can rely on the shape:

```text
ScrollView
└── Padding(PAGE_MARGIN)
    └── VStack (gap HEADER_GAP)
        ├── header   HStack: Back button? + SFSymbolImage + BasicText
        └── content  VStack (gap BLOCK_GAP): Form / lists / blocks
```

### `page_shell`

```rust
pub(crate) fn page_shell(header: HStack, body: impl View + 'static) -> PageView
```

Wraps `body` in the standard content stack, then header plus content in
the shell stack, then the shell in a `Padding` inside a `ScrollView`. The
page title is not repeated inside the page: the sidebar toolbar already
shows it.

### `PageView`

```rust
pub(crate) struct PageView { /* private */ }

impl PageView {
    pub(crate) fn bare(scroll: ScrollView) -> Self
    pub(crate) fn content_mut(&mut self) -> Option<&mut VStack>
    pub(crate) fn form_mut(&mut self) -> Option<&mut Form>
    pub(crate) fn type_text(&mut self, text: &str)
    pub(crate) fn key(&mut self, key: Key) -> bool
    pub(crate) fn theme(&mut self, skin: &Skin, focused: bool, viewport: Viewport)
    pub(crate) fn wants_backdrop(&mut self) -> bool
    pub(crate) fn wants_text_cursor(&mut self) -> bool
    pub(crate) fn hover(&mut self, x: f64, y: f64)
}
```

`PageView` is the app's only handle into a built page, because the
`Sidebar` boxes its pages as `Box<dyn View>`. `form_mut` finds the first
`Form` in the content stack so `App::text` and `App::key` can reach the
page's inline fields, and so the picker panels get `set_viewport`.

`theme` walks the stack by index. `VStack` and `HStack` expose
`child_mut::<T>(index)`, so the walker tries each supported element type
once per slot and the first match claims it; nested stacks recurse two
levels, which covers every page shape in this app.

### `SharedPage`

```rust
#[derive(Clone)]
pub(crate) struct SharedPage(Rc<RefCell<PageView>>);

impl SharedPage {
    pub(crate) fn new(page: PageView) -> Self
    pub(crate) fn with<R>(&self, f: impl FnOnce(&mut PageView) -> R) -> R
}
```

A page handle shared by the app (which themes it and forwards input) and
the `Sidebar` (which owns and draws it). The same `SharedPage` clone goes
into `Sidebar::page`, so `draw` and `theme` always see the same tree.

## Navigation

```rust
pub(crate) struct Entry {
    pub label: &'static str,
    pub symbol: &'static str,
}

pub(crate) const ENTRIES: [Entry; PAGE_COUNT] = [/* 28 rows */];

pub(crate) fn sidebar_item(index: usize) -> SidebarItem
pub(crate) fn header_symbol(index: usize) -> &'static str
pub(crate) fn page_label(index: usize) -> String
pub(crate) fn header_subtitle(index: usize) -> String
```

The table is the single source of truth for the sidebar labels, the row
icons and the page header symbols, so a row and its page can never drift
apart. Indices are named constants (`WIFI`, `NETWORK`, `GENERAL`, ...)
because pages dispatch on them.

| Index | Category | Module |
|---|---|---|
| 0 | Wi-Fi | `wifi` |
| 1 | Bluetooth | `simple` |
| 2 | Network | `network` |
| 3 | Battery | `simple` |
| 4 | General | `general` |
| 5 | Accessibility | `simple` |
| 6 | Appearance | `appearance` |
| 7 | Desktop & Dock | `simple` |
| 8 | Displays | `displays` |
| 9 | Menu Bar | `simple` |
| 10 | Tinti AI | `simple` |
| 11 | Spotlight | `simple` |
| 12 | Wallpaper | `wallpaper` |
| 13 | Notifications | `simple` |
| 14 | Sound | `simple` |
| 15 | Focus | `simple` |
| 16 | Screen Time | `simple` |
| 17 | Lock Screen | `simple` |
| 18 | Privacy & Security | `simple` |
| 19 | Touch ID & Password | `simple` |
| 20 | Users & Groups | `simple` |
| 21 | Internet Accounts | `simple` |
| 22 | Octo Cloud | `simple` |
| 23 | Keyboard | `simple` |
| 24 | Mouse & Trackpad | `simple` |
| 25 | Printers | `simple` |
| 26 | App Settings | `simple` |
| 27 | Developer | `simple` |

Three bundled PNGs that used to be sidebar icons (Appearance, Tinti AI,
Launchpad for App Settings) are now SF Symbols
(`circle.lefthalf.filled`, `sparkles`, `square.grid.2x2.fill`): the
`Sidebar` resolves row icons through `SFSymbolImage`, which does not take
file paths.

### Hidden detail pages

About, Date & Time and Language & Region stay out of the sidebar, as in
the GTK build. Their General rows push the detail instead:

```rust
pub(crate) const ABOUT_HIDDEN: usize = 0;
pub(crate) const DATETIME_HIDDEN: usize = 1;
pub(crate) const LOCALE_HIDDEN: usize = 2;

pub(crate) fn hidden_title(pushed: usize) -> &'static str
```

`general::build` returns the pushed detail when `nav.current()` is set,
and the detail header carries a `general.back` button; `App::key`
handles `Escape` as well.

### `Nav`

```rust
#[derive(Clone, Default)]
pub(crate) struct Nav {
    pub pushed: Rc<Cell<Option<usize>>>,
    pub join: Rc<Cell<Option<(String, bool)>>>,
    pub wallpaper: Rc<Cell<Option<crate::daemon::WallpaperEntry>>>,
    pub dirty: Rc<Cell<bool>>,
}

impl Nav {
    pub(crate) fn new() -> Self
    pub(crate) fn push(&self, index: usize)
    pub(crate) fn pop(&self)
    pub(crate) fn current(&self) -> Option<usize>
    pub(crate) fn request_join(&self, ssid: &str, secured: bool)
    pub(crate) fn take_join(&self) -> Option<(String, bool)>
    pub(crate) fn request_wallpaper(&self, entry: crate::daemon::WallpaperEntry)
    pub(crate) fn take_wallpaper(&self) -> Option<crate::daemon::WallpaperEntry>
    pub(crate) fn touch(&self)
}
```

The one channel between pages and the app. A widget callback captures a
`Nav` clone and reports through it; the app drains the queue once per
frame and turns it into a sheet, a daemon call or a rebuild. Nothing in a
callback touches the element tree.

## Theme

```rust
#[derive(Clone, Copy)]
pub(crate) struct Skin {
    pub bg: Color,
    pub text: Color,
    pub divider: Color,
    pub accent: Color,
    pub mode: ThemeMode,
    pub glass: GlassAmount,
    pub dark: bool,
}

impl Skin {
    pub(crate) fn from_theme(
        mode: ThemeMode,
        accent: Color,
        glass: GlassAmount,
        palette: &Palette,
    ) -> Self
}
```

### Colors

Only the mandated TontooOS tokens are hard coded. Everything else comes
from the TontooUI `ThemeWatcher` palette, so the app defines no secondary
color of its own.

| Token | Dark | Light | Source |
|---|---|---|---|
| Window background | `#1b2022` | `#ffffff` | `BG_DARK` / `BG_LIGHT` |
| Body text | `#d8d9d9` | `#272727` | `TEXT_DARK` / `TEXT_LIGHT` |
| Group body, dividers, dim labels | — | — | `palette.divider`, `Form` tokens |
| Accent | — | — | `theme.accent.color()` |

The GTK build had its own `Palette { fg, secondary, card }` and hand-picked
sidebar tile colors per category. All of that is gone; `Form` already
paints its group bodies and dividers from the theme, and the sidebar tints
row icons with the accent.

Text uses SF Pro through TontooUI's CoreText `FontSystem`; the app no
longer names a font family or builds Pango markup.

| Helper | Signature |
|---|---|
| `parse_color` | `pub(crate) fn parse_color(hex: &str) -> Color` |
| `accent_hex` | `pub(crate) fn accent_hex(color: Color) -> String` |

## Pages

### `simple` — the placeholder categories

```rust
pub(crate) enum Row {
    Info(&'static str, &'static str),
    Toggle(&'static str, bool),
}

pub(crate) struct SimplePage {
    pub index: usize,
    pub master: Option<(&'static str, bool)>,
    pub caption: Option<&'static str>,
    pub rows: &'static [Row],
}
```

Twenty-two categories had near-identical GTK modules (about 130 lines
each of `info_row`, `toggle_row`, `markup_label` and margin juggling).
They are now one table plus one `build`, rendered from `Form`,
`FormRow::text`, `FormRow::toggle`, `Toggle` and `Spacer`. The switches
still only report to stdout, exactly like the GTK build, until the
category gets a real backend.

`every_index_is_covered_exactly_once` keeps the table and the dedicated
modules in sync, and `every_row_key_resolves` fails the build when a lang
key goes missing.

### `wifi`

```rust
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Radio {
    Unreachable,
    NoAdapter,
    Off,
    On { known: Vec<Row>, networks: Vec<Row> },
}

pub(crate) fn resolve_state() -> Radio
pub(crate) fn fingerprint(state: &Radio) -> String
pub(crate) fn is_secured(security: &str) -> bool
```

The header row carries the radio switch pinned to the trailing edge via a
`Spacer`. The two network lists are `BasicOutlineGroup` rows: picking one
calls `nav.request_join(ssid, secured)`, and the app opens the join
sheet. Rows show the SSID, the signal percentage for a live scan and the
localized `wifi.row.locked` marker for a secured network.

`fingerprint` deliberately leaves the signal percentage out, so a scan in
progress does not rebuild the page (and reset the scroll position) every
two seconds.

### `network`

DNS is a `FormRow::text` (a native borderless right-aligned field) plus an
Apply button. The field only feeds a draft `Rc<RefCell<String>>`; the
button commits it through the daemon, so a half-typed address never
reaches NetworkManager. Empty input means DHCP. `dns_error_text`
classifies failures: an `invalid IPv4` error gets the format hint, a
missing NetworkManager or active connection gets the unavailable note.

Each connected interface gets its own titled `FormSection` with the known
fields; unknown ones are skipped.

### `displays`

Output info, night light and refresh rate are `FormRow` values; the
brightness slider is a native `Slider` that dims the desktop live. The
refresh rate options are the standard rates up to the monitor max plus
every reported rate, capped at 1000 Hz, sorted and deduplicated by
`refresh_rates`.

### `general`

Ten rows in four groups (`[3, 1, 4, 2]`), rendered as tappable
`BasicOutlineGroup` blocks with a trailing chevron. A group with no
navigable row is not selectable at all. `target_for` maps the About,
Date & Time and Language & Region rows onto the hidden details; every
other row is display only.

### `appearance`

Three theme thumbnails from `Resources/`, a row of accent color dots and
the four icon and widget style previews rendered through the CoreIcon
`AppIcon` pipeline (`Default`, `Dark`, `TintedLight`, `TintedDark`,
cached per style in the temp dir). Display only. The accent dot matching
the live theme accent gets a stroke.

### `wallpaper`

The current wallpaper preview plus the fill mode dropdown, then one
`BasicOutlineGroup` per group. Picking a row calls
`nav.request_wallpaper(entry)` and the app opens the apply sheet with the
Light/Auto/Dark variant.

```rust
pub(crate) const FILL_ORDER: &[&str] = &["fill", "fit", "stretch", "center", "tile"];
pub(crate) const PREVIEW_ORDER: &[&str] = &["light", "auto", "dark"];

pub(crate) fn thumb_cache_path(cache_dir: &Path, source: &Path) -> Option<PathBuf>
pub(crate) fn cached_thumb(source: &Path) -> Option<PathBuf>
pub(crate) fn split_preview(light: &Path, dark: &Path, width: u32, height: u32) -> Option<PathBuf>
pub(crate) fn preview_file(entry: &daemon::WallpaperEntry, variant: &str) -> Option<PathBuf>
```

`gdk-pixbuf` and the `image` crate are gone; CoreImage does the work.
`FileImage` downsamples on load, so the grid does not need pre-scaled
thumbnails. The one case that still needs an offline composite is the
Auto preview: `split_preview` loads two cached thumbnails through
CoreImage, walks the pixels once (the divider drifts right going down, so
left stays light and right goes dark) and writes a PNG next to the other
thumbnails. That keeps a 6K original out of the popup's critical path.

> **Note:** the Browse... button is gone. It needs an `NSOpenPanel`
> equivalent, and TontooLibs does not have a file-open element yet, so
> custom uploads are unreachable until one exists.

### `about`

Device, TontooOS and storage groups, all read live with the localized
Unknown fallback: `/etc/hostname`, `/proc/cpuinfo`, `/proc/meminfo`,
`/proc/sys/kernel/osrelease`, the daemon `get_os`, and `df -B1` for
storage. The app count walks `/Applications`, every user's
`~/Applications` and `/System/Applications`, deduplicating symlinked
bundles by canonical path. The OS logo prefers the CoreIcon versioned
asset and falls back to the bundled `app_icon.png`.

### `datetime`

Date and time from the Unix epoch through a civil-from-days conversion
(no date crate), the 24-hour switch through the daemon, and a timezone
dropdown built from the zone list the daemon reports. Automatic time
stays locked on: the daemon enforces NTP at startup and the UI offers no
way to turn it off.

### `locale`

System language list with a checkmark on the active entry, a region
dropdown with the full country list, and a keyboard group with the Auto
Detect switch plus the layout and variant dropdowns. Picking a layout
clears the variant and asks for a rebuild, so the variant list follows
the picked layout. Everything applies through the daemon `localectl`
backend.

Long searchable lists use the native `FormRow::picker` dropdown rather
than a custom search sheet, which is why the old popovers with
`gtk::SearchEntry` plus `gtk::ListBox` are gone.

## Sheets

```rust
pub(crate) enum Kind {
    Join { ssid: String, secured: bool },
    Wallpaper { entry: daemon::WallpaperEntry, variant: usize },
    Notice { title: String, message: String },
}

pub(crate) enum Action {
    Join { ssid: String, password: String },
    Wallpaper { kind: String, id: String, variant: String },
    Dismiss,
}

pub(crate) struct Sheets { /* private */ }

impl Sheets {
    pub(crate) fn new() -> Self
    pub(crate) fn is_open(&self) -> bool
    pub(crate) fn open(&mut self, kind: Kind, skin: &Skin)
    pub(crate) fn close(&mut self)
    pub(crate) fn pump(&mut self)
    pub(crate) fn take_action(&mut self) -> Option<Action>
}
```

One `BasicSheet` host for every modal, so the app routes input to and
drains a single overlay. `open` builds the content and resets the shared
flags; `pump` keeps the wallpaper preview in sync with the segmented
picker, then turns the button flags into one `Action`. The app drains
that action and talks to the daemon, so a failed join shows up as a
`Notice` sheet with the daemon message instead of a silent no-op.

`Enter` submits, `Escape` cancels. While a sheet is open it swallows every
event, which is what keeps the page behind it from receiving presses.

## Localization

Strings live in `lang/en_us.json` and `lang/de_de.json` (only these two).
`src/lang.rs` detects German from `LANGUAGE`, `LC_ALL`, `LANG` or
`/etc/locale.conf` and falls back to `en_us`.

`Resources/lang/` holds copies of both files: TBuild copies only
`Resources/` into the `.app` bundle (root `lang/` is used just for the
localized `name` in `Info.tontoo`). Keep both locations in sync.

### `t(key)`

```rust
pub fn t(key: &str) -> String
```

Returns the localized string, or the key itself when it is missing, so a
gap shows up as a raw key rather than an empty label.

### Keys added by the port

| Key | en_us | de_de |
|---|---|---|
| `appearance.header.subtitle` | `Choose the theme, accent color and icon style.` | `Design, Akzentfarbe und Symbolstil wählen.` |
| `general.back` | `Back` | `Zurück` |
| `network.dns.apply` | `Apply` | `Anwenden` |
| `wifi.no_networks` | `No networks found.` | `Keine Netzwerke gefunden.` |
| `wifi.row.locked` | `Secured` | `Gesichert` |

## Daemon

`src/daemon.rs` is unchanged by the port: newline-delimited JSON over a
`UnixStream` at `/run/tontoo-settings.sock`, overridable with
`SETTINGS_SOCKET`. It is pure data, so it never touched GTK and needed no
work. It covers Wi-Fi, DNS, wired interfaces, date and time, locale,
wallpaper, display and OS info.

## Errors

| Where | Behavior |
|---|---|
| Daemon socket unreachable | Every call returns `Err(String)`; pages fall back to a placeholder state (Wi-Fi shows example rows, wallpaper shows the "no wallpaper" note, wired shows the empty note) |
| `dns_set` validation | `dns_error_text` prefixes the format hint |
| `dns_set` without NetworkManager | `dns_error_text` prefixes the unavailable note |
| `wallpaper_apply` fails | Logged; the sheet stays out of the way and the page is not touched |
| Wi-Fi join fails | Logged plus a `Notice` sheet carrying the daemon message |
| CoreImage cannot decode a wallpaper | `preview_file` and `cached_thumb` return `None`; `FileImage` draws its theme placeholder |
| Missing `Resources/*.png` | Appearance and About fall back to the SF Symbol arm |

## Tests

77 unit tests, no GTK or windowing fixture required:

| Module | Coverage |
|---|---|
| `daemon` | Protocol roundtrips against a `UnixListener` mock |
| `lang` | Locale detection, missing-key fallback |
| `views` | Navigation table, token values, `Nav` queues, color helpers |
| `simple` | Index coverage, row and subtitle keys resolve |
| `wifi` | `is_secured`, signal clamping, row labels, fingerprint stability |
| `network` | DNS error classification, value formatting, detail lines |
| `displays` | Refresh rate options, mode and output text |
| `general` | Row table, group bounds, navigation targets |
| `about` | `/proc` parsing, unit formatting, app counting, `df` parsing |
| `datetime` | Civil date conversion, both clock modes |
| `wallpaper` | Fill order, thumb cache keys, split preview pixels |

Run them with:

```bash
cargo test
```

## Cross References

- [MAIN.md](MAIN.md) – wiki index and changelog
- TontooUI `wiki/MAIN.md` – the renderer, `Sidebar`, `Form`, `Sheet` and
  `ThemeWatcher` pages this app is built on
