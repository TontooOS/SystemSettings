//! The SystemSettings window on the TontooUI renderer.
//!
//! One full-bleed `Sidebar` owns the navigation column and the detail
//! page of the selected row, so selecting a row swaps the page inside the
//! sidebar's page slot. The app keeps the `SharedPage` handle of every
//! page so it can theme them, route text, keys and hover into the visible
//! one, and swap in a fresh build when daemon-backed state changed.
//!
//! Modal work (joining a network, applying a wallpaper) is queued by the
//! pages through [`views::Nav`] and turned into a sheet by [`Sheets`].
//! The live Wi-Fi state is polled on an interval; only a coarse
//! fingerprint triggers a rebuild, so a live scan never resets the
//! scroll position.

use crate::lang;
use crate::views::{
  self, Nav, SheetAction, SheetKind, Sheets, SharedPage, Skin, PAGE_COUNT,
};
use crate::TontooUI::elements::{Sidebar, SidebarItem, TrafficAction, View};
use crate::TontooUI::renderer::window::{App, CursorKind, Key, Viewport, WindowCommand};
use crate::TontooUI::renderer::{FontSystem, ImageLoader};
use crate::TontooUI::theme::ThemeWatcher;
use crate::TontooUI::{Color, Scene};

/// Polling interval for the live Wi-Fi state in seconds.
const WIFI_POLL_SECONDS: f64 = 2.0;

/// The Settings app: the sidebar, its pages, the theme watcher and the
/// modal sheet host.
pub struct SettingsApp {
  sidebar: Sidebar,
  pages: Vec<SharedPage>,
  sheets: Sheets,
  nav: Nav,
  watcher: ThemeWatcher,
  skin: Skin,
  selected: usize,
  focused: bool,
  bg: Color,
  text_cursor: bool,
  page_backdrop: bool,
  wifi_mark: String,
  wifi_poll: f64,
  command: Option<WindowCommand>,
}

impl SettingsApp {
  pub fn new() -> Self {
    let mut watcher = ThemeWatcher::new();
    let theme = watcher.theme();
    let skin = Skin::from_theme(
      theme.mode,
      theme.accent.color(),
      theme.glass,
      &watcher.palette(0.0),
    );
    let nav = Nav::new();

    let mut app = Self {
      // Empty sidebar on the first pass: `rebuild` fills in every row and
      // its page, seeded with the shared column width.
      sidebar: Sidebar::new(Vec::new()).width(views::SIDEBAR_W),
      pages: Vec::new(),
      sheets: Sheets::new(),
      nav,
      watcher,
      skin,
      selected: 0,
      focused: true,
      bg: skin.bg,
      text_cursor: false,
      page_backdrop: false,
      wifi_mark: String::new(),
      wifi_poll: 0.0,
      command: None,
    };
    app.rebuild();
    app.wifi_mark = views::wifi::fingerprint(&views::wifi::resolve_state());
    app
  }

  /// Rebuild every page from the current state and hand them to a fresh
  /// sidebar. Keeps the column width, the collapse state and the search
  /// query.
  fn rebuild(&mut self) {
    let width = self.sidebar.width_value();
    let collapsed = self.sidebar.is_collapsed();
    let selected = self.sidebar.selected_index();
    let query = self.sidebar.search_text().to_string();

    let pages: Vec<SharedPage> = (0..PAGE_COUNT)
      .map(|index| SharedPage::new(views::build_page(index, &self.skin, &self.nav)))
      .collect();

    let items: Vec<SidebarItem> = (0..PAGE_COUNT).map(views::sidebar_item).collect();
    let mut sidebar = Sidebar::new(items)
      .width(width)
      .search_field(true)
      // The avatar pill replaces the old GTK header injection: a round
      // person glyph sitting next to the traffic lights.
      .left_button(0, "person.crop.circle.fill", || {})
      .toggle_button(false)
      .collapsible(false);
    for page in pages.iter() {
      sidebar = sidebar.page(page.clone());
    }
    sidebar.set_glass(self.skin.mode, self.skin.glass);
    sidebar.set_theme(self.skin.accent, self.skin.dark);
    sidebar.set_focused(self.focused);
    sidebar.set_collapsed(collapsed);
    sidebar.set_title(self.title(selected));
    // The selection always exists: the index came from the old sidebar.
    debug_assert!(sidebar.select(selected));
    if !query.is_empty() {
      sidebar.set_search_text(query);
    }

    self.sidebar = sidebar;
    self.pages = pages;
    self.selected = selected;
    self.nav.dirty.set(false);
  }

  /// Sidebar title for a navigation index: the selected row label, or the
  /// pushed detail title while a hidden page is open.
  fn title(&self, selected: usize) -> String {
    match self.nav.current() {
      Some(pushed) => lang::t(views::hidden_title(pushed)),
      None => views::page_label(selected),
    }
  }

  /// Poll the live Wi-Fi state and request a rebuild when it changed.
  fn poll_wifi(&mut self, now: f64) {
    if now - self.wifi_poll < WIFI_POLL_SECONDS {
      return;
    }
    self.wifi_poll = now;
    let mark = views::wifi::fingerprint(&views::wifi::resolve_state());
    if mark != self.wifi_mark {
      self.wifi_mark = mark;
      self.nav.touch();
    }
  }

  /// Turn the queued page requests into sheets and apply the result of a
  /// finished sheet to the settings daemon.
  fn pump(&mut self) {
    if let Some((ssid, secured)) = self.nav.take_join() {
      self.sheets
        .open(SheetKind::Join { ssid, secured }, &self.skin);
    }
    if let Some(entry) = self.nav.take_wallpaper() {
      self.sheets.open(SheetKind::Wallpaper { entry, variant: 1 }, &self.skin);
    }
    self.sheets.pump();
    let Some(action) = self.sheets.take_action() else {
      return;
    };
    match action {
      SheetAction::Join { ssid, password } => {
        let password = if password.is_empty() {
          None
        } else {
          Some(password.as_str())
        };
        match crate::daemon::connect(&ssid, password, false) {
          Ok(_) => println!("Wi-Fi joined: {ssid}"),
          Err(err) => {
            println!("Wi-Fi join failed: {err}");
            self.sheets.open(
              SheetKind::Notice {
                title: ssid,
                message: views::network::dns_error_text(&err),
              },
              &self.skin,
            );
          }
        }
        self.nav.touch();
      }
      SheetAction::Wallpaper { kind, id, variant } => {
        match crate::daemon::wallpaper_apply(&kind, &id, &variant) {
          Ok(applied) => println!("Wallpaper applied: {} ({variant})", applied.path),
          Err(err) => println!("Wallpaper apply failed: {err}"),
        }
        self.nav.touch();
      }
      SheetAction::Dismiss => {}
    }
  }

  /// Refresh the theme tokens and rebuild when the theme flipped.
  fn sync_theme(&mut self, now: f64, viewport: Viewport) -> bool {
    self.watcher.set_focused(self.focused, now);
    if self.watcher.poll(now) {
      let theme = self.watcher.theme();
      self.skin = Skin::from_theme(
        theme.mode,
        theme.accent.color(),
        theme.glass,
        &self.watcher.palette(now),
      );
      return true;
    }
    let palette = self.watcher.palette(now);
    self.bg = palette.bg;
    self.skin.bg = palette.bg;
    for page in self.pages.iter() {
      page.with(|page| page.theme(&self.skin, self.focused, viewport));
    }
    false
  }

  /// Route hover into the visible page, which the sidebar does not do.
  fn hover(&mut self, x: f64, y: f64) {
    self.sidebar.set_hover(x as f32, y as f32);
    if self.sheets.is_open() {
      self.sheets.hover(x, y);
      return;
    }
    if let Some(page) = self.pages.get(self.selected) {
      page.with(|page| page.hover(x, y));
    }
  }

  /// True while any element wants the text cursor.
  fn needs_text_cursor(&mut self) -> bool {
    if self.sheets.is_open() {
      return self.sheets.wants_text_cursor();
    }
    if self.sidebar.search_text_cursor() {
      return true;
    }
    self
      .pages
      .get(self.selected)
      .map(|page| page.with(|page| page.wants_text_cursor()))
      .unwrap_or(false)
  }

  /// True while any element needs the backdrop blur pass.
  fn needs_backdrop(&self) -> bool {
    self.page_backdrop || self.sidebar.wants_backdrop() || self.sheets.wants_backdrop()
  }
}

impl Default for SettingsApp {
  fn default() -> Self {
    Self::new()
  }
}

impl App for SettingsApp {
  fn draw(
    &mut self,
    scene: &mut Scene,
    fonts: &mut FontSystem,
    images: &mut ImageLoader<'_>,
    viewport: Viewport,
    time_secs: f64,
  ) {
    let theme_changed = self.sync_theme(time_secs, viewport);
    self.bg = self.skin.bg;
    self.poll_wifi(time_secs);
    self.pump();
    if theme_changed || self.nav.dirty.get() {
      self.rebuild();
      self.sidebar.set_title(self.title(self.selected));
    }

    self.sidebar.set_theme(self.skin.accent, self.skin.dark);
    self.sidebar.set_glass(self.skin.mode, self.skin.glass);
    self.sidebar.set_focused(self.focused);
    let selected = self.sidebar.selected_index();
    if selected != self.selected {
      self.selected = selected;
      self.nav.pushed.set(None);
      self.sidebar.set_title(self.title(selected));
    }

    // Full-bleed: the sidebar owns the traffic lights, so no titlebar.
    self.sidebar.place(
      fonts,
      viewport.x,
      viewport.y,
      viewport.width,
      viewport.height,
    );
    self.sidebar.draw(scene, fonts, images);

    if self.sheets.is_open() {
      self.sheets.draw(scene, fonts, images, viewport);
    }
    self.text_cursor = self.needs_text_cursor();
    self.page_backdrop = self
      .pages
      .get(self.selected)
      .map(|page| page.with(|page| page.wants_backdrop()))
      .unwrap_or(false);
  }

  fn background(&self) -> Color {
    self.bg
  }

  fn wants_backdrop(&self) -> bool {
    self.needs_backdrop()
  }

  fn drag_region(&self) -> Option<(f32, f32, f32, f32)> {
    Some(self.sidebar.drag_rect())
  }

  fn poll_window_command(&mut self) -> Option<WindowCommand> {
    self.command.take()
  }

  fn cursor(&self, x: f64, y: f64) -> CursorKind {
    if self.text_cursor {
      return CursorKind::Text;
    }
    if self.sheets.is_open() {
      return CursorKind::Default;
    }
    if self.sidebar.wants_resize_cursor(x, y) {
      return CursorKind::ResizeColumn;
    }
    CursorKind::Default
  }

  fn mouse_down(&mut self, x: f64, y: f64) {
    if self.sheets.is_open() {
      self.sheets.mouse_down(x, y);
      return;
    }
    // Traffic lights first: a hit must not reach the page.
    if let Some(action) = self.sidebar.press(x, y) {
      self.command = Some(match action {
        TrafficAction::Close => WindowCommand::Close,
        TrafficAction::Minimize => WindowCommand::Minimize,
        TrafficAction::Maximize => WindowCommand::ToggleMaximize,
      });
      return;
    }
    self.sidebar.mouse_down(x, y);
  }

  fn mouse_move(&mut self, x: f64, y: f64) {
    self.hover(x, y);
  }

  fn mouse_up(&mut self, x: f64, y: f64) {
    if self.sheets.is_open() {
      self.sheets.mouse_up(x, y);
      return;
    }
    self.sidebar.mouse_up(x, y);
  }

  fn mouse_wheel(&mut self, dx: f64, dy: f64) {
    if self.sheets.is_open() {
      return;
    }
    self.sidebar.mouse_wheel(dx, dy);
  }

  fn set_focused(&mut self, focused: bool) {
    self.focused = focused;
    self.sidebar.set_focused(focused);
  }

  fn text(&mut self, text: &str) {
    if self.sheets.is_open() {
      self.sheets.type_text(text);
      return;
    }
    self.sidebar.page_text(text);
  }

  fn key(&mut self, key: Key) {
    if self.sheets.is_open() {
      self.sheets.key(key);
      return;
    }
    // Escape leaves a pushed hidden detail before it reaches the page.
    if key == Key::Escape && self.nav.current().is_some() {
      self.nav.pop();
      self.sidebar.set_title(self.title(self.selected));
    }
    let _ = self.sidebar.page_key(key);
  }
}
