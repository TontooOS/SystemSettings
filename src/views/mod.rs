//! Shared page scaffolding for SystemSettings.
//!
//! The window is one full-bleed TontooUI `Sidebar`: every navigation row
//! owns one detail page, so selecting a row swaps the page inside the
//! sidebar's page slot. All pages share the same shell built here:
//!
//! ```text
//! ScrollView
//! └── Padding(PAGE_MARGIN)
//!     └── VStack (gap HEADER_GAP)
//!         ├── header   HStack: back button? SF Symbol + subtitle
//!         └── content  VStack (gap BLOCK_GAP): Form / lists / blocks
//! ```
//!
//! `Skin` carries the theme tokens of the current frame. Only the
//! mandated TontooOS colors are used: background `#1b2022` dark and
//! `#ffffff` light, text `#d8d9d9` dark and `#272727` light. Every
//! divider and accent value comes from the TontooUI `ThemeWatcher`
//! palette, so the app never defines a secondary color of its own.

pub mod about;
pub mod appearance;
pub mod datetime;
pub mod displays;
pub mod general;
pub mod locale;
pub mod network;
pub mod sheet;
pub mod simple;
pub mod wifi;
pub mod wallpaper;

pub(crate) use sheet::{Action as SheetAction, Kind as SheetKind, Sheets};

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::lang;
use crate::TontooUI::elements::{
  Align, BasicList, BasicOutlineGroup, BasicText, Button, ButtonStyle, FileImage, Form, HStack,
  Padding, SFSymbolImage, ScrollView, SegmentedPicker, SidebarItem, Slider, TextAlignment, Toggle,
  VStack, View,
};
use crate::TontooUI::renderer::window::{Key, Viewport};
use crate::TontooUI::renderer::{FontSystem, ImageLoader};
use crate::TontooUI::theme::{GlassAmount, Palette, ThemeMode};
use crate::TontooUI::Color;

// ── geometry ────────────────────────────────────────────────────────

/// Window size in logical px.
pub(crate) const WINDOW_W: u32 = 900;
pub(crate) const WINDOW_H: u32 = 600;

/// Sidebar column width in logical px.
pub(crate) const SIDEBAR_W: f32 = 240.0;
/// Page padding on every side in logical px.
pub(crate) const PAGE_MARGIN: f32 = 28.0;
/// Gap between the page header and the page content in logical px.
pub(crate) const HEADER_GAP: f32 = 14.0;
/// Gap between two content blocks in logical px.
pub(crate) const BLOCK_GAP: f32 = 22.0;
/// Wrap width for subtitle and note text in logical px.
pub(crate) const TEXT_W: f32 = 420.0;
/// Header SF Symbol size in logical px.
pub(crate) const HEADER_SYMBOL_PX: f32 = 26.0;

// ── mandated TontooOS color tokens ──────────────────────────────────

/// Window background in dark mode.
pub(crate) const BG_DARK: Color = Color::from_rgb8(0x1b, 0x20, 0x22);
/// Window background in light mode.
pub(crate) const BG_LIGHT: Color = Color::from_rgb8(0xff, 0xff, 0xff);
/// Body text in dark mode.
pub(crate) const TEXT_DARK: Color = Color::from_rgb8(0xd8, 0xd9, 0xd9);
/// Body text in light mode.
pub(crate) const TEXT_LIGHT: Color = Color::from_rgb8(0x27, 0x27, 0x27);

// ── navigation table ────────────────────────────────────────────────

/// Sidebar index: Wi-Fi.
pub(crate) const WIFI: usize = 0;
/// Sidebar index: Bluetooth.
pub(crate) const BLUETOOTH: usize = 1;
/// Sidebar index: Network.
pub(crate) const NETWORK: usize = 2;
/// Sidebar index: Battery.
pub(crate) const BATTERY: usize = 3;
/// Sidebar index: General.
pub(crate) const GENERAL: usize = 4;
/// Sidebar index: Accessibility.
pub(crate) const ACCESSIBILITY: usize = 5;
/// Sidebar index: Appearance.
pub(crate) const APPEARANCE: usize = 6;
/// Sidebar index: Desktop & Dock.
pub(crate) const DESKTOP_DOCK: usize = 7;
/// Sidebar index: Displays.
pub(crate) const DISPLAYS: usize = 8;
/// Sidebar index: Menu Bar.
pub(crate) const MENU_BAR: usize = 9;
/// Sidebar index: Tinti AI.
pub(crate) const TINTI_AI: usize = 10;
/// Sidebar index: Spotlight.
pub(crate) const SPOTLIGHT: usize = 11;
/// Sidebar index: Wallpaper.
pub(crate) const WALLPAPER: usize = 12;
/// Sidebar index: Notifications.
pub(crate) const NOTIFICATIONS: usize = 13;
/// Sidebar index: Sound.
pub(crate) const SOUND: usize = 14;
/// Sidebar index: Focus.
pub(crate) const FOCUS: usize = 15;
/// Sidebar index: Screen Time.
pub(crate) const SCREEN_TIME: usize = 16;
/// Sidebar index: Lock Screen.
pub(crate) const LOCK_SCREEN: usize = 17;
/// Sidebar index: Privacy & Security.
pub(crate) const PRIVACY: usize = 18;
/// Sidebar index: Touch ID & Password.
pub(crate) const TOUCH_ID: usize = 19;
/// Sidebar index: Users & Groups.
pub(crate) const USERS: usize = 20;
/// Sidebar index: Internet Accounts.
pub(crate) const INTERNET_ACCOUNTS: usize = 21;
/// Sidebar index: Octo Cloud.
pub(crate) const OCTO_CLOUD: usize = 22;
/// Sidebar index: Keyboard.
pub(crate) const KEYBOARD: usize = 23;
/// Sidebar index: Mouse & Trackpad.
pub(crate) const MOUSE: usize = 24;
/// Sidebar index: Printers.
pub(crate) const PRINTERS: usize = 25;
/// Sidebar index: App Settings.
pub(crate) const APP_SETTINGS: usize = 26;
/// Sidebar index: Developer.
pub(crate) const DEVELOPER: usize = 27;
/// Number of sidebar rows.
pub(crate) const PAGE_COUNT: usize = 28;

/// Sidebar row: label lang key plus the SF Symbol used for both the row
/// icon and the page header.
pub(crate) struct Entry {
  pub label: &'static str,
  pub symbol: &'static str,
}

/// Navigation table in sidebar order.
pub(crate) const ENTRIES: [Entry; PAGE_COUNT] = [
  Entry { label: "sidebar.wifi", symbol: "wifi" },
  Entry { label: "sidebar.bluetooth", symbol: "antenna.radiowaves.left.and.right" },
  Entry { label: "sidebar.network", symbol: "network" },
  Entry { label: "sidebar.battery", symbol: "battery.100" },
  Entry { label: "sidebar.general", symbol: "gear" },
  Entry { label: "sidebar.accessibility", symbol: "figure.wave.circle" },
  Entry { label: "sidebar.appearance", symbol: "circle.lefthalf.filled" },
  Entry { label: "sidebar.desktop_dock", symbol: "menubar.dock.rectangle" },
  Entry { label: "sidebar.displays", symbol: "display" },
  Entry { label: "sidebar.menu_bar", symbol: "switch.2" },
  Entry { label: "sidebar.tinti_ai", symbol: "sparkles" },
  Entry { label: "sidebar.spotlight", symbol: "magnifyingglass" },
  Entry { label: "sidebar.wallpaper", symbol: "photo" },
  Entry { label: "sidebar.notifications", symbol: "bell.badge.fill" },
  Entry { label: "sidebar.sound", symbol: "speaker.wave.3.fill" },
  Entry { label: "sidebar.focus", symbol: "moon.fill" },
  Entry { label: "sidebar.screen_time", symbol: "hourglass" },
  Entry { label: "sidebar.lock_screen", symbol: "lock.fill" },
  Entry { label: "sidebar.privacy", symbol: "hand.raised.fill" },
  Entry { label: "sidebar.touch_id", symbol: "touchid" },
  Entry { label: "sidebar.users", symbol: "person.2.fill" },
  Entry { label: "sidebar.internet_accounts", symbol: "mail.stack.fill" },
  Entry { label: "sidebar.octo_cloud", symbol: "icloud.fill" },
  Entry { label: "sidebar.keyboard", symbol: "keyboard.fill" },
  Entry { label: "sidebar.mouse", symbol: "cursorarrow" },
  Entry { label: "sidebar.printers", symbol: "printer.fill" },
  Entry { label: "sidebar.app_settings", symbol: "square.grid.2x2.fill" },
  Entry { label: "sidebar.developer", symbol: "hammer.fill" },
];

/// Sidebar row for a navigation index, localized.
pub(crate) fn sidebar_item(index: usize) -> SidebarItem {
  let entry = ENTRIES.get(index).unwrap_or(&ENTRIES[0]);
  SidebarItem::new(lang::t(entry.label), entry.symbol)
}

/// Header SF Symbol for a page: the sidebar symbol, so row and page always
/// match.
pub(crate) fn header_symbol(index: usize) -> &'static str {
  ENTRIES
    .get(index)
    .map(|entry| entry.symbol)
    .unwrap_or("gear")
}

/// Localized sidebar label for a navigation index.
pub(crate) fn page_label(index: usize) -> String {
  let entry = ENTRIES.get(index).unwrap_or(&ENTRIES[0]);
  lang::t(entry.label)
}

/// Subtitle lang key of a page, following the `<page>.header.subtitle`
/// naming the pages use.
const fn subtitle_key(index: usize) -> &'static str {
  match index {
    WIFI => "wifi.header.subtitle",
    BLUETOOTH => "bluetooth.header.subtitle",
    NETWORK => "network.header.subtitle",
    BATTERY => "battery.header.subtitle",
    GENERAL => "general.header.subtitle",
    ACCESSIBILITY => "accessibility.header.subtitle",
    APPEARANCE => "appearance.header.subtitle",
    DESKTOP_DOCK => "desktop_dock.header.subtitle",
    DISPLAYS => "displays.header.subtitle",
    MENU_BAR => "menu_bar.header.subtitle",
    TINTI_AI => "tinti_ai.header.subtitle",
    SPOTLIGHT => "spotlight.header.subtitle",
    WALLPAPER => "wallpaper.header.subtitle",
    NOTIFICATIONS => "notifications.header.subtitle",
    SOUND => "sound.header.subtitle",
    FOCUS => "focus.header.subtitle",
    SCREEN_TIME => "screen_time.header.subtitle",
    LOCK_SCREEN => "lock_screen.header.subtitle",
    PRIVACY => "privacy.header.subtitle",
    TOUCH_ID => "touch_id.header.subtitle",
    USERS => "users.header.subtitle",
    INTERNET_ACCOUNTS => "internet_accounts.header.subtitle",
    OCTO_CLOUD => "octo_cloud.header.subtitle",
    KEYBOARD => "keyboard.header.subtitle",
    MOUSE => "mouse.header.subtitle",
    PRINTERS => "printers.header.subtitle",
    APP_SETTINGS => "app_settings.header.subtitle",
    _ => "developer.header.subtitle",
  }
}

/// Localized page subtitle for a navigation index.
pub(crate) fn header_subtitle(index: usize) -> String {
  lang::t(subtitle_key(index))
}

// ── hidden detail pages pushed from General ─────────────────────────

/// Hidden detail: About.
pub(crate) const ABOUT_HIDDEN: usize = 0;
/// Hidden detail: Date & Time.
pub(crate) const DATETIME_HIDDEN: usize = 1;
/// Hidden detail: Language & Region.
pub(crate) const LOCALE_HIDDEN: usize = 2;

/// Lang key of a hidden detail title.
pub(crate) fn hidden_title(pushed: usize) -> &'static str {
  match pushed {
    DATETIME_HIDDEN => "general.datetime",
    LOCALE_HIDDEN => "general.language",
    _ => "general.about",
  }
}

/// Shared navigation state: the hidden detail currently pushed on top of
/// the General page, the network or wallpaper a page asked the app to act
/// on, plus a rebuild flag the app watches each frame.
#[derive(Clone, Default)]
pub(crate) struct Nav {
  /// `None` shows the General list, `Some(index)` the pushed detail.
  pub pushed: Rc<Cell<Option<usize>>>,
  /// Network the Wi-Fi page asked the app to join: SSID plus whether a
  /// password is required.
  pub join: Rc<Cell<Option<(String, bool)>>>,
  /// Premade or custom wallpaper the Wallpaper page wants applied.
  pub wallpaper: Rc<Cell<Option<crate::daemon::WallpaperEntry>>>,
  /// Raised by page callbacks that change daemon-backed state, so the
  /// app rebuilds the visible page on the next frame.
  pub dirty: Rc<Cell<bool>>,
}

impl Nav {
  pub(crate) fn new() -> Self {
    Self::default()
  }

  /// Push a hidden detail page onto the General page.
  pub(crate) fn push(&self, index: usize) {
    self.pushed.set(Some(index));
    self.dirty.set(true);
  }

  /// Leave the pushed detail and show the General list again.
  pub(crate) fn pop(&self) {
    self.pushed.set(None);
    self.dirty.set(true);
  }

  /// Currently pushed detail, if any.
  pub(crate) fn current(&self) -> Option<usize> {
    self.pushed.get()
  }

  /// Ask the app to open the join sheet for a scanned network.
  pub(crate) fn request_join(&self, ssid: &str, secured: bool) {
    self.join.set(Some((ssid.to_string(), secured)));
  }

  /// Take the pending join request, if any.
  pub(crate) fn take_join(&self) -> Option<(String, bool)> {
    self.join.take()
  }

  /// Ask the app to open the wallpaper apply sheet.
  pub(crate) fn request_wallpaper(&self, entry: crate::daemon::WallpaperEntry) {
    self.wallpaper.set(Some(entry));
  }

  /// Take the pending wallpaper request, if any.
  pub(crate) fn take_wallpaper(&self) -> Option<crate::daemon::WallpaperEntry> {
    self.wallpaper.take()
  }

  /// Request a rebuild on the next frame.
  pub(crate) fn touch(&self) {
    self.dirty.set(true);
  }
}

// ── theme ───────────────────────────────────────────────────────────

/// Theme tokens of one frame, taken from the TontooUI `ThemeWatcher`
/// palette plus the two mandated TontooOS background/text tokens.
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
  ) -> Self {
    let dark = mode == ThemeMode::Dark;
    Self {
      bg: if dark { BG_DARK } else { BG_LIGHT },
      text: if dark { TEXT_DARK } else { TEXT_LIGHT },
      divider: palette.divider,
      accent,
      mode,
      glass,
      dark,
    }
  }
}

/// Parse a `#rrggbb` string into an opaque color, falling back to the dark
/// background token.
pub(crate) fn parse_color(hex: &str) -> Color {
  let strip = hex.trim_start_matches('#');
  if strip.len() == 6 {
    if let Ok(value) = u32::from_str_radix(strip, 16) {
      return Color::from_rgb8(
        ((value >> 16) & 0xff) as u8,
        ((value >> 8) & 0xff) as u8,
        (value & 0xff) as u8,
      );
    }
  }
  BG_DARK
}

/// Lowercase `#rrggbb` form of a color, for comparing against a token
/// table.
pub(crate) fn accent_hex(color: Color) -> String {
  let rgba = color.to_rgba8();
  format!("#{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b)
}

// ── shared building blocks ──────────────────────────────────────────

/// Page header row: SF Symbol plus the localized subtitle. The page
/// title itself lives in the sidebar toolbar, so it is not repeated.
pub(crate) fn page_header(symbol: &str, subtitle: &str) -> HStack {
  HStack::new()
    .spacing(HEADER_GAP)
    .align(Align::Leading)
    .child(SFSymbolImage::new(symbol).size(HEADER_SYMBOL_PX))
    .child(
      BasicText::new(subtitle)
        .size(13.0)
        .weight(400.0)
        .width(TEXT_W)
        .alignment(TextAlignment::Leading),
    )
}

/// Page header row for a hidden detail pushed from the General list: a
/// Back button in front of the usual symbol and subtitle.
pub(crate) fn detail_header(nav: &Nav, symbol: &str, subtitle: &str) -> HStack {
  let popped = nav.clone();
  HStack::new()
    .spacing(HEADER_GAP)
    .align(Align::Leading)
    .child(
      Button::new(lang::t("general.back"))
        .style(ButtonStyle::Bordered)
        .icon("chevron.backward")
        .on_press(move || popped.pop()),
    )
    .child(SFSymbolImage::new(symbol).size(HEADER_SYMBOL_PX))
    .child(
      BasicText::new(subtitle)
        .size(13.0)
        .weight(400.0)
        .width(TEXT_W)
        .alignment(TextAlignment::Leading),
    )
}

/// Wrapped secondary note below a section header.
pub(crate) fn note(text: &str) -> BasicText {
  BasicText::new(text)
    .size(12.0)
    .weight(400.0)
    .width(TEXT_W)
    .alignment(TextAlignment::Leading)
}

/// Section caption above a group of rows.
pub(crate) fn caption(text: &str) -> BasicText {
  BasicText::new(text)
    .size(12.0)
    .weight(600.0)
    .width(TEXT_W)
    .alignment(TextAlignment::Leading)
}

/// Wrap page content into the scrolling page shell. The shell is always
/// `Padding[ VStack[ header: HStack, content: VStack ] ]`, so
/// [`PageView::content_mut`] and the theme walker can rely on the shape.
pub(crate) fn page_shell(header: HStack, body: impl View + 'static) -> PageView {
  let content = VStack::new()
    .spacing(BLOCK_GAP)
    .align(Align::Leading)
    .child(body);
  let stack = VStack::new()
    .spacing(HEADER_GAP)
    .align(Align::Leading)
    .child(header)
    .child(content);
  PageView::bare(ScrollView::new(Padding::all(stack, PAGE_MARGIN)))
}

// ── theming ─────────────────────────────────────────────────────────

/// Minimal common surface of `VStack` and `HStack`, so the theme walker
/// can recurse through either container without duplicating itself.
trait Stack {
  /// Number of child slots.
  fn count(&self) -> usize;
  /// The child in `index`, when it is a `T`.
  fn slot<T: View + 'static>(&mut self, index: usize) -> Option<&mut T>;
}

impl Stack for VStack {
  fn count(&self) -> usize {
    self.len()
  }

  fn slot<T: View + 'static>(&mut self, index: usize) -> Option<&mut T> {
    self.child_mut::<T>(index)
  }
}

impl Stack for HStack {
  fn count(&self) -> usize {
    self.len()
  }

  fn slot<T: View + 'static>(&mut self, index: usize) -> Option<&mut T> {
    self.child_mut::<T>(index)
  }
}

/// Theme every direct child of a stack. Stacks only expose
/// `child_mut::<T>(index)`, so each element type is tried once per slot
/// and the first match claims it. Nested stacks recurse `depth` levels,
/// which covers every page shape in this app.
fn theme_stack<S: Stack>(
  stack: &mut S,
  skin: &Skin,
  focused: bool,
  viewport: Viewport,
  depth: usize,
) {
  for index in 0..stack.count() {
    if let Some(element) = stack.slot::<Form>(index) {
      element.set_theme(skin.accent, skin.dark);
      element.set_glass(skin.mode, skin.glass);
      element.set_focused(focused);
      element.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
      continue;
    }
    if let Some(element) = stack.slot::<BasicText>(index) {
      element.set_theme(skin.mode);
      element.set_focused(focused);
      continue;
    }
    if let Some(element) = stack.slot::<SFSymbolImage>(index) {
      element.set_theme(skin.text, skin.dark);
      element.set_focused(focused);
      continue;
    }
    if let Some(element) = stack.slot::<BasicOutlineGroup>(index) {
      element.set_theme(skin.accent, skin.dark);
      element.set_focused(focused);
      continue;
    }
    if let Some(element) = stack.slot::<BasicList>(index) {
      element.set_theme(skin.divider, skin.dark);
      element.set_focused(focused);
      continue;
    }
    if let Some(element) = stack.slot::<FileImage>(index) {
      element.set_theme(skin.dark);
      element.set_focused(focused);
      continue;
    }
    if let Some(element) = stack.slot::<Slider>(index) {
      element.set_theme(skin.accent, skin.dark, skin.glass);
      element.set_focused(focused);
      continue;
    }
    if let Some(element) = stack.slot::<SegmentedPicker>(index) {
      element.set_theme(skin.accent, skin.dark);
      element.set_focused(focused);
      continue;
    }
    if let Some(element) = stack.slot::<Toggle>(index) {
      element.set_theme(skin.accent, skin.dark);
      element.set_focused(focused);
      continue;
    }
    if let Some(element) = stack.slot::<Button>(index) {
      element.set_theme(skin.accent, skin.dark);
      element.set_focused(focused);
      continue;
    }
    if let Some(element) = stack.slot::<ScrollView>(index) {
      element.set_theme(skin.accent, skin.dark);
      element.set_focused(focused);
      continue;
    }
    if depth == 0 {
      continue;
    }
    if let Some(element) = stack.slot::<VStack>(index) {
      theme_stack(element, skin, focused, viewport, depth - 1);
      continue;
    }
    if let Some(element) = stack.slot::<HStack>(index) {
      theme_stack(element, skin, focused, viewport, depth - 1);
    }
  }
}

// ── page view ───────────────────────────────────────────────────────

/// One detail page: a scroll shell whose form stays reachable by index
/// so the app can theme it and forward text and keys into it.
pub(crate) struct PageView {
  scroll: ScrollView,
}

impl PageView {
  /// Shell around an already assembled scroll view.
  pub(crate) fn bare(scroll: ScrollView) -> Self {
    Self { scroll }
  }

  /// The shell stack, when the page uses the standard shape.
  fn shell_mut(&mut self) -> Option<&mut VStack> {
    self.scroll.child_mut::<Padding>()?.child_mut::<VStack>()
  }

  /// The page content stack (shell child 1).
  pub(crate) fn content_mut(&mut self) -> Option<&mut VStack> {
    self.shell_mut()?.child_mut::<VStack>(1)
  }

  /// The page form, when the content carries one. The slot is located
  /// first, because the borrow checker cannot return a `child_mut` borrow
  /// from inside a loop over the same stack.
  pub(crate) fn form_mut(&mut self) -> Option<&mut Form> {
    let content = self.content_mut()?;
    let mut found = None;
    for index in 0..content.len() {
      if content.child_mut::<Form>(index).is_some() {
        found = Some(index);
        break;
      }
    }
    content.child_mut::<Form>(found?)
  }

  /// Forward printable text into the form.
  pub(crate) fn type_text(&mut self, text: &str) {
    if let Some(form) = self.form_mut() {
      form.type_text(text);
    }
  }

  /// Forward a key into the form. Returns true when consumed.
  pub(crate) fn key(&mut self, key: Key) -> bool {
    match self.form_mut() {
      Some(form) => form.key(key),
      None => false,
    }
  }

  /// Apply the frame theme plus the viewport the picker panels need.
  pub(crate) fn theme(&mut self, skin: &Skin, focused: bool, viewport: Viewport) {
    self.scroll.set_theme(skin.accent, skin.dark);
    self.scroll.set_focused(focused);
    if let Some(shell) = self.shell_mut() {
      theme_stack(shell, skin, focused, viewport, 2);
    }
  }

  /// True while a picker panel inside the page form needs the blur pass.
  pub(crate) fn wants_backdrop(&mut self) -> bool {
    self
      .form_mut()
      .map(|form| form.wants_backdrop())
      .unwrap_or(false)
  }

  /// True while a text field inside the page form wants the I-beam
  /// cursor.
  pub(crate) fn wants_text_cursor(&mut self) -> bool {
    self
      .form_mut()
      .map(|form| form.wants_text_cursor())
      .unwrap_or(false)
  }

  /// Hover routing for controls that only implement `mouse_move`.
  pub(crate) fn hover(&mut self, x: f64, y: f64) {
    self.scroll.mouse_move(x, y);
  }

  pub(crate) fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
    self.scroll.measure(fonts)
  }

  pub(crate) fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32) {
    self.scroll.place(fonts, x, y, width, height);
  }

  pub(crate) fn draw(
    &mut self,
    scene: &mut crate::TontooUI::Scene,
    fonts: &mut FontSystem,
    images: &mut ImageLoader<'_>,
  ) {
    self.scroll.draw(scene, fonts, images);
  }
}

impl View for PageView {
  fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
    self.scroll.measure(fonts)
  }

  fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32) {
    self.scroll.place(fonts, x, y, width, height);
  }

  fn draw(
    &mut self,
    scene: &mut crate::TontooUI::Scene,
    fonts: &mut FontSystem,
    images: &mut ImageLoader<'_>,
  ) {
    self.scroll.draw(scene, fonts, images);
  }

  fn mouse_down(&mut self, x: f64, y: f64) {
    self.scroll.mouse_down(x, y);
  }

  fn mouse_up(&mut self, x: f64, y: f64) {
    self.scroll.mouse_up(x, y);
  }

  fn set_hover(&mut self, x: f32, y: f32) {
    self.scroll.set_hover(x, y);
  }

  fn set_focused(&mut self, focused: bool) {
    self.scroll.set_focused(focused);
    if let Some(form) = self.form_mut() {
      form.set_focused(focused);
    }
  }

  fn text(&mut self, text: &str) {
    PageView::type_text(self, text);
  }

  fn key(&mut self, key: Key) -> bool {
    PageView::key(self, key)
  }

  fn mouse_wheel(&mut self, dx: f64, dy: f64) {
    self.scroll.mouse_wheel(dx, dy);
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
}

/// A page shared between the app (which themes it and forwards text) and
/// the sidebar (which owns and draws it).
#[derive(Clone)]
pub(crate) struct SharedPage(Rc<RefCell<PageView>>);

impl SharedPage {
  pub(crate) fn new(page: PageView) -> Self {
    Self(Rc::new(RefCell::new(page)))
  }

  /// Borrow the page for a short mutation.
  pub(crate) fn with<R>(&self, f: impl FnOnce(&mut PageView) -> R) -> R {
    f(&mut self.0.borrow_mut())
  }
}

impl View for SharedPage {
  fn measure(&mut self, fonts: &mut FontSystem) -> (f32, f32) {
    self.0.borrow_mut().measure(fonts)
  }

  fn place(&mut self, fonts: &mut FontSystem, x: f32, y: f32, width: f32, height: f32) {
    self.0.borrow_mut().place(fonts, x, y, width, height);
  }

  fn draw(
    &mut self,
    scene: &mut crate::TontooUI::Scene,
    fonts: &mut FontSystem,
    images: &mut ImageLoader<'_>,
  ) {
    self.0.borrow_mut().draw(scene, fonts, images);
  }

  fn mouse_down(&mut self, x: f64, y: f64) {
    self.0.borrow_mut().mouse_down(x, y);
  }

  fn mouse_up(&mut self, x: f64, y: f64) {
    self.0.borrow_mut().mouse_up(x, y);
  }

  fn set_hover(&mut self, x: f32, y: f32) {
    self.0.borrow_mut().set_hover(x, y);
  }

  fn set_focused(&mut self, focused: bool) {
    self.0.borrow_mut().set_focused(focused);
  }

  fn text(&mut self, text: &str) {
    self.0.borrow_mut().type_text(text);
  }

  fn key(&mut self, key: Key) -> bool {
    self.0.borrow_mut().key(key)
  }

  fn mouse_wheel(&mut self, dx: f64, dy: f64) {
    self.0.borrow_mut().mouse_wheel(dx, dy);
  }

  fn as_any_mut(&mut self) -> &mut dyn Any {
    self
  }
}

// ── page dispatch ───────────────────────────────────────────────────

/// Build the detail page for a sidebar index.
pub(crate) fn build_page(index: usize, skin: &Skin, nav: &Nav) -> PageView {
  match index {
    WIFI => wifi::build(skin, nav),
    NETWORK => network::build(skin, nav),
    GENERAL => general::build(skin, nav),
    APPEARANCE => appearance::build(skin, nav),
    DISPLAYS => displays::build(skin, nav),
    WALLPAPER => wallpaper::build(skin, nav),
    _ => simple::build(index, skin),
  }
}

/// Build one of the hidden detail pages pushed from the General list.
pub(crate) fn build_hidden(pushed: usize, skin: &Skin, nav: &Nav) -> PageView {
  match pushed {
    DATETIME_HIDDEN => datetime::build(skin, nav),
    LOCALE_HIDDEN => locale::build(skin, nav),
    _ => about::build(skin, nav),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_entry_has_a_label_and_symbol() {
    assert_eq!(ENTRIES.len(), PAGE_COUNT);
    for (index, entry) in ENTRIES.iter().enumerate() {
      assert!(entry.label.starts_with("sidebar."), "index {index}");
      assert!(!entry.symbol.is_empty(), "index {index}");
    }
  }

  #[test]
  fn every_entry_label_resolves() {
    for entry in ENTRIES.iter() {
      assert_ne!(lang::t(entry.label), entry.label, "missing {}", entry.label);
    }
  }

  #[test]
  fn every_subtitle_key_resolves() {
    for index in 0..PAGE_COUNT {
      assert_ne!(
        lang::t(subtitle_key(index)),
        subtitle_key(index),
        "missing {}",
        subtitle_key(index)
      );
    }
  }

  #[test]
  fn dark_and_light_tokens_match_the_mandate() {
    assert_eq!(BG_DARK.to_rgba8().r, 0x1b);
    assert_eq!(BG_DARK.to_rgba8().g, 0x20);
    assert_eq!(BG_DARK.to_rgba8().b, 0x22);
    assert_eq!(TEXT_DARK.to_rgba8().r, 0xd8);
    assert_eq!(TEXT_DARK.to_rgba8().g, 0xd9);
    assert_eq!(TEXT_DARK.to_rgba8().b, 0xd9);
    assert_eq!(BG_LIGHT.to_rgba8().r, 0xff);
    assert_eq!(BG_LIGHT.to_rgba8().g, 0xff);
    assert_eq!(BG_LIGHT.to_rgba8().b, 0xff);
    assert_eq!(TEXT_LIGHT.to_rgba8().r, 0x27);
    assert_eq!(TEXT_LIGHT.to_rgba8().g, 0x27);
    assert_eq!(TEXT_LIGHT.to_rgba8().b, 0x27);
  }

  #[test]
  fn hidden_titles_are_the_general_rows() {
    assert_eq!(hidden_title(ABOUT_HIDDEN), "general.about");
    assert_eq!(hidden_title(DATETIME_HIDDEN), "general.datetime");
    assert_eq!(hidden_title(LOCALE_HIDDEN), "general.language");
  }

  #[test]
  fn nav_push_and_pop_round_trip() {
    let nav = Nav::new();
    assert_eq!(nav.current(), None);
    nav.push(ABOUT_HIDDEN);
    assert_eq!(nav.current(), Some(ABOUT_HIDDEN));
    assert!(nav.dirty.get());
    nav.dirty.set(false);
    nav.pop();
    assert_eq!(nav.current(), None);
    assert!(nav.dirty.get());
  }

  #[test]
  fn nav_queues_join_and_wallpaper_requests() {
    let nav = Nav::new();
    nav.request_join("HomeNet", true);
    assert_eq!(nav.take_join(), Some(("HomeNet".to_string(), true)));
    assert_eq!(nav.take_join(), None);

    let entry = crate::daemon::WallpaperEntry {
      kind: "premade".to_string(),
      id: "DUO".to_string(),
      name: "Duo".to_string(),
      path: "/tmp/light.png".to_string(),
      path_dark: "/tmp/dark.png".to_string(),
    };
    nav.request_wallpaper(entry.clone());
    assert_eq!(nav.take_wallpaper().map(|e| e.id), Some("DUO".to_string()));
    assert!(nav.take_wallpaper().is_none());
  }

  #[test]
  fn color_helpers_round_trip() {
    assert_eq!(parse_color("#007aff").to_rgba8().r, 0x00);
    assert_eq!(parse_color("#007aff").to_rgba8().g, 0x7a);
    assert_eq!(parse_color("#007aff").to_rgba8().b, 0xff);
    assert_eq!(accent_hex(parse_color("#007AFF")), "#007aff");
    // Unparsable input falls back to the dark background token.
    assert_eq!(parse_color("nope"), BG_DARK);
  }
}
