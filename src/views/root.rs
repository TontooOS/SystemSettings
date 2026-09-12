//! Settings root for SystemSettings.
//!
//! The `Sidebar` (Wi-Fi, Bluetooth, Network, General, Appearance,
//! Desktop & Dock) is stored and exposed via `children()` so UIKit's
//! `hides_window_bar_recursive` finds it: the system decoration bar stays
//! hidden and the traffic lights render directly on the sidebar, like
//! Apple Settings.
//!
//! Above the detail content sits a toolbar (back/forward buttons plus the
//! current page title). Selecting a row swaps the detail page in place.
//! GTK widgets are not `Send + Sync`, so the `on_select` handler (which
//! must be both) only records the index in shared navigation state; a
//! lightweight main-thread poller (100ms, TabView pattern) swaps the page,
//! refreshes the title and the button sensitivity, and stops itself once
//! its containers leave the window (e.g. after a rebuild).

use super::{WIFI_BLUE, is_dark};
use crate::lang;
use crate::TontooUI::{Button, ButtonStyle, Sidebar, SidebarIcon};
use crate::UIKit::apply_css;
use crate::UIKit::prelude::*;
use crate::UIKit::widget::{WidgetId, next_widget_id};
use gtk::prelude::*;
use std::sync::{Arc, Mutex};

/// Sidebar index order: 0 Wi-Fi, 1 Bluetooth, 2 Network, 3 Battery,
/// 4 General, 5 Accessibility, 6 Appearance, 7 Desktop & Dock,
/// 8 Displays, 9 Menu Bar, 10 Tinti AI, 11 Spotlight, 12 Wallpaper,
/// 13 Notifications, 14 Sound, 15 Focus, 16 Screen Time, 17 Lock Screen,
/// 18 Privacy & Security, 19 Touch ID & Password, 20 Users & Groups,
/// 21 Internet Accounts, 22 Octo Cloud, 23 Keyboard, 24 Mouse & Trackpad,
/// 25 Printers, 26 App Settings, 27 Developer, 28 About (hidden detail
/// page behind the General About row, reached via history only).
const PAGE_TITLES: [&str; 29] = [
  "wifi.title",
  "bluetooth.title",
  "network.title",
  "battery.title",
  "general.title",
  "accessibility.title",
  "appearance.title",
  "desktop_dock.title",
  "displays.title",
  "menu_bar.title",
  "tinti_ai.title",
  "spotlight.title",
  "wallpaper.title",
  "notifications.title",
  "sound.title",
  "focus.title",
  "screen_time.title",
  "lock_screen.title",
  "privacy.title",
  "touch_id.title",
  "users.title",
  "internet_accounts.title",
  "octo_cloud.title",
  "keyboard.title",
  "mouse.title",
  "printers.title",
  "app_settings.title",
  "developer.title",
  "about.title",
];

/// Shared back/forward navigation state (Send + Sync for `on_select`).
/// Page 28 is the hidden About detail (General row, back/forward only).
#[derive(Debug, Default)]
pub(crate) struct NavState {
  selected: usize,
  history: Vec<usize>,
  pos: usize,
}

impl NavState {
  pub(crate) fn go(&mut self, index: usize) {
    self.history.truncate(self.pos + 1);
    self.history.push(index);
    self.pos = self.history.len() - 1;
    self.selected = index;
  }

  fn back(&mut self) -> bool {
    if self.pos > 0 {
      self.pos -= 1;
      self.selected = self.history[self.pos];
      true
    } else {
      false
    }
  }

  fn forward(&mut self) -> bool {
    if self.pos + 1 < self.history.len() {
      self.pos += 1;
      self.selected = self.history[self.pos];
      true
    } else {
      false
    }
  }

  fn can_back(&self) -> bool {
    self.pos > 0
  }

  fn can_forward(&self) -> bool {
    self.pos + 1 < self.history.len()
  }
}

/// Page title markup for the toolbar.
fn title_markup(index: usize, fg: &str) -> String {
  let key = PAGE_TITLES.get(index).copied().unwrap_or(PAGE_TITLES[0]);
  let text = lang::t(key);
  format!(
    "<span font_desc=\"{} bold 15\" foreground=\"{}\">{}</span>",
    super::SF_PRO,
    fg,
    glib::markup_escape_text(&text),
  )
}

pub struct SettingsRoot {
  id: WidgetId,
  sidebar: Sidebar,
  wifi_page: gtk::Widget,
  bluetooth_page: gtk::Widget,
  network_page: gtk::Widget,
  battery_page: gtk::Widget,
  general_page: gtk::Widget,
  accessibility_page: gtk::Widget,
  appearance_page: gtk::Widget,
  desktop_dock_page: gtk::Widget,
  displays_page: gtk::Widget,
  menu_bar_page: gtk::Widget,
  tinti_ai_page: gtk::Widget,
  spotlight_page: gtk::Widget,
  wallpaper_page: gtk::Widget,
  notifications_page: gtk::Widget,
  sound_page: gtk::Widget,
  focus_page: gtk::Widget,
  screen_time_page: gtk::Widget,
  lock_screen_page: gtk::Widget,
  privacy_page: gtk::Widget,
  touch_id_page: gtk::Widget,
  users_page: gtk::Widget,
  internet_accounts_page: gtk::Widget,
  octo_cloud_page: gtk::Widget,
  keyboard_page: gtk::Widget,
  mouse_page: gtk::Widget,
  printers_page: gtk::Widget,
  app_settings_page: gtk::Widget,
  developer_page: gtk::Widget,
  about_page: gtk::Widget,
  nav: Arc<Mutex<NavState>>,
}

/// Pick the detail page for a sidebar index.
fn page_for<'a>(
  index: usize,
  wifi: &'a gtk::Widget,
  bluetooth: &'a gtk::Widget,
  network: &'a gtk::Widget,
  battery: &'a gtk::Widget,
  general: &'a gtk::Widget,
  accessibility: &'a gtk::Widget,
  appearance: &'a gtk::Widget,
  desktop_dock: &'a gtk::Widget,
  displays: &'a gtk::Widget,
  menu_bar: &'a gtk::Widget,
  tinti_ai: &'a gtk::Widget,
  spotlight: &'a gtk::Widget,
  wallpaper: &'a gtk::Widget,
  notifications: &'a gtk::Widget,
  sound: &'a gtk::Widget,
  focus: &'a gtk::Widget,
  screen_time: &'a gtk::Widget,
  lock_screen: &'a gtk::Widget,
  privacy: &'a gtk::Widget,
  touch_id: &'a gtk::Widget,
  users: &'a gtk::Widget,
  internet_accounts: &'a gtk::Widget,
  octo_cloud: &'a gtk::Widget,
  keyboard: &'a gtk::Widget,
  mouse: &'a gtk::Widget,
  printers: &'a gtk::Widget,
  app_settings: &'a gtk::Widget,
  developer: &'a gtk::Widget,
  about: &'a gtk::Widget,
) -> &'a gtk::Widget {
  match index {
    0 => wifi,
    1 => bluetooth,
    2 => network,
    3 => battery,
    4 => general,
    5 => accessibility,
    6 => appearance,
    7 => desktop_dock,
    8 => displays,
    9 => menu_bar,
    10 => tinti_ai,
    11 => spotlight,
    12 => wallpaper,
    13 => notifications,
    14 => sound,
    15 => focus,
    16 => screen_time,
    17 => lock_screen,
    18 => privacy,
    19 => touch_id,
    20 => users,
    21 => internet_accounts,
    22 => octo_cloud,
    23 => keyboard,
    24 => mouse,
    25 => printers,
    26 => app_settings,
    27 => developer,
    28 => about,
    _ => developer,
  }
}

impl SettingsRoot {
  pub fn new() -> Self {
    // Navigation state first: General rows navigate programmatically.
    let nav: Arc<Mutex<NavState>> = Arc::new(Mutex::new(NavState::default()));
    {
      let mut state = nav.lock().unwrap();
      state.go(0);
    }

    let wifi_page = super::wifi::build_page();
    let bluetooth_page = super::bluetooth::build_page();
    let network_page = super::network::build_page();
    let battery_page = super::battery::build_page();
    let general_page = super::general::build_page(&nav);
    let accessibility_page = super::accessibility::build_page();
    let appearance_page = super::appearance::build_page();
    let desktop_dock_page = super::desktop_dock::build_page();
    let displays_page = super::displays::build_page();
    let menu_bar_page = super::menu_bar::build_page();
    let tinti_ai_page = super::tinti_ai::build_page();
    let spotlight_page = super::spotlight::build_page();
    let wallpaper_page = super::wallpaper::build_page();
    let notifications_page = super::notifications::build_page();
    let sound_page = super::sound::build_page();
    let focus_page = super::focus::build_page();
    let screen_time_page = super::screen_time::build_page();
    let lock_screen_page = super::lock_screen::build_page();
    let privacy_page = super::privacy::build_page();
    let touch_id_page = super::touch_id::build_page();
    let users_page = super::users::build_page();
    let internet_accounts_page = super::internet_accounts::build_page();
    let octo_cloud_page = super::octo_cloud::build_page();
    let keyboard_page = super::keyboard::build_page();
    let mouse_page = super::mouse::build_page();
    let printers_page = super::printers::build_page();
    let app_settings_page = super::app_settings::build_page();
    let developer_page = super::developer::build_page();
    let about_page = super::about::build_page();

    // Send + Sync only: records the index, the poller in `to_gtk` swaps.
    let nav_cb = nav.clone();
    // Apple sidebar fill (explicit, TontooUI default is lighter):
    // dark `#1C1C1E` (darker than the content), light `#EBEBF0`.
    let dark = is_dark();
    let sidebar_bg = if dark {
      Color::from_rgb(28, 28, 30)
    } else {
      Color::from_rgb(235, 235, 240)
    };
    let sidebar = Sidebar::new()
      .item(
        lang::t("sidebar.wifi"),
        SidebarIcon::sf(
          "wifi",
          Color::from_rgb(WIFI_BLUE.0, WIFI_BLUE.1, WIFI_BLUE.2),
        ),
      )
      .item(
        lang::t("sidebar.bluetooth"),
        SidebarIcon::sf(
          "antenna.radiowaves.left.and.right",
          Color::from_rgb(WIFI_BLUE.0, WIFI_BLUE.1, WIFI_BLUE.2),
        ),
      )
      .item(
        lang::t("sidebar.network"),
        SidebarIcon::sf(
          "network",
          Color::from_rgb(WIFI_BLUE.0, WIFI_BLUE.1, WIFI_BLUE.2),
        ),
      )
      .item(
        lang::t("sidebar.battery"),
        SidebarIcon::sf(
          "battery.100",
          Color::from_rgb(
            super::BATTERY_GREEN.0,
            super::BATTERY_GREEN.1,
            super::BATTERY_GREEN.2,
          ),
        ),
      )
      .section("")
      .item(
        lang::t("sidebar.general"),
        SidebarIcon::sf(
          "gear",
          Color::from_rgb(142, 142, 147),
        ),
      )
      .item(
        lang::t("sidebar.accessibility"),
        SidebarIcon::sf(
          "figure.wave.circle",
          Color::from_rgb(WIFI_BLUE.0, WIFI_BLUE.1, WIFI_BLUE.2),
        ),
      )
      .item(
        lang::t("sidebar.appearance"),
        SidebarIcon::file(super::appearance::appearance_png()),
      )
      .item(
        lang::t("sidebar.desktop_dock"),
        SidebarIcon::sf(
          "menubar.dock.rectangle",
          Color::from_rgb(0, 0, 0),
        ),
      )
      .item(
        lang::t("sidebar.displays"),
        SidebarIcon::sf(
          "sun.max.fill",
          Color::from_rgb(WIFI_BLUE.0, WIFI_BLUE.1, WIFI_BLUE.2),
        ),
      )
      .item(
        lang::t("sidebar.menu_bar"),
        SidebarIcon::sf(
          "switch.2",
          Color::from_rgb(142, 142, 147),
        ),
      )
      .item(
        lang::t("sidebar.tinti_ai"),
        SidebarIcon::file(super::tinti_ai::tinti_png()),
      )
      .item(
        lang::t("sidebar.spotlight"),
        SidebarIcon::sf(
          "magnifyingglass",
          Color::from_rgb(142, 142, 147),
        ),
      )
      .item(
        lang::t("sidebar.wallpaper"),
        SidebarIcon::sf(
          "atom",
          Color::from_rgb(48, 176, 199),
        ),
      )
      .section("")
      .item(
        lang::t("sidebar.notifications"),
        SidebarIcon::sf(
          "bell.badge.fill",
          Color::from_rgb(255, 69, 58),
        ),
      )
      .item(
        lang::t("sidebar.sound"),
        SidebarIcon::sf(
          "speaker.wave.3.fill",
          Color::from_rgb(255, 45, 85),
        ),
      )
      .item(
        lang::t("sidebar.focus"),
        SidebarIcon::sf(
          "moon.fill",
          Color::from_rgb(88, 86, 214),
        ),
      )
      .item(
        lang::t("sidebar.screen_time"),
        SidebarIcon::sf(
          "hourglass",
          Color::from_rgb(88, 86, 214),
        ),
      )
      .section("")
      .item(
        lang::t("sidebar.lock_screen"),
        SidebarIcon::sf(
          "lock.fill",
          Color::from_rgb(0, 0, 0),
        ),
      )
      .item(
        lang::t("sidebar.privacy"),
        SidebarIcon::sf(
          "hand.raised.fill",
          Color::from_rgb(WIFI_BLUE.0, WIFI_BLUE.1, WIFI_BLUE.2),
        ),
      )
      .item(
        lang::t("sidebar.touch_id"),
        SidebarIcon::sf(
          "touchid",
          Color::from_rgb(255, 45, 85),
        ),
      )
      .item(
        lang::t("sidebar.users"),
        SidebarIcon::sf(
          "person.2.fill",
          Color::from_rgb(142, 142, 147),
        ),
      )
      .section("")
      .item(
        lang::t("sidebar.internet_accounts"),
        SidebarIcon::sf(
          "mail.stack.fill",
          Color::from_rgb(WIFI_BLUE.0, WIFI_BLUE.1, WIFI_BLUE.2),
        ),
      )
      .item(
        lang::t("sidebar.octo_cloud"),
        SidebarIcon::sf(
          "icloud.fill",
          Color::from_rgb(255, 107, 43),
        ),
      )
      .section("")
      .item(
        lang::t("sidebar.keyboard"),
        SidebarIcon::sf(
          "keyboard.fill",
          Color::from_rgb(142, 142, 147),
        ),
      )
      .item(
        lang::t("sidebar.mouse"),
        SidebarIcon::sf(
          "cursorarrow",
          Color::from_rgb(142, 142, 147),
        ),
      )
      .item(
        lang::t("sidebar.printers"),
        SidebarIcon::sf(
          "printer.fill",
          Color::from_rgb(142, 142, 147),
        ),
      )
      .section("")
      .item(
        lang::t("sidebar.app_settings"),
        SidebarIcon::file(super::app_settings::launchpad_png()),
      )
      .section("")
      .item(
        lang::t("sidebar.developer"),
        SidebarIcon::sf(
          "hammer.fill",
          Color::from_rgb(142, 142, 147),
        ),
      )
      .selected(0)
      .search_placeholder(lang::t("sidebar.search"))
      .background_color(sidebar_bg)
      .width(220.0)
      .on_select(move |i| {
        nav_cb.lock().unwrap().go(i);
        println!("Settings selected: {}", i);
      });

    Self {
      id: next_widget_id(),
      sidebar,
      wifi_page,
      bluetooth_page,
      network_page,
      battery_page,
      general_page,
      accessibility_page,
      appearance_page,
      desktop_dock_page,
      displays_page,
      menu_bar_page,
      tinti_ai_page,
      spotlight_page,
      wallpaper_page,
      notifications_page,
      sound_page,
      focus_page,
      screen_time_page,
      lock_screen_page,
      privacy_page,
      touch_id_page,
      users_page,
      internet_accounts_page,
      octo_cloud_page,
      keyboard_page,
      mouse_page,
      printers_page,
      app_settings_page,
      developer_page,
      about_page,
      nav,
    }
  }
}

impl Default for SettingsRoot {
  fn default() -> Self {
    Self::new()
  }
}

/// Sign-in header below the sidebar search: avatar plus bold title and
/// subtitle. Display only for now (no navigation target yet).
fn signin_row(fg: &str, secondary: &str) -> gtk::Box {
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  row.set_hexpand(true);
  row.set_margin_top(12);
  row.set_margin_bottom(10);
  row.set_margin_start(10);
  row.set_margin_end(10);
  if let Some(avatar_path) = super::avatar_icon_path("person.crop.circle.fill", "signin") {
    // Exact 40px file: the raw tile texture must never reach layout,
    // or the avatar renders giant.
    let file = super::wallpaper::cached_thumb_exact(
      std::path::Path::new(&avatar_path),
      40,
      40,
    )
    .unwrap_or_else(|| std::path::PathBuf::from(&avatar_path));
    if let Some(file) = file.to_str() {
      let avatar = gtk::Picture::for_filename(file);
      avatar.set_content_fit(gtk::ContentFit::Cover);
      avatar.set_hexpand(false);
      avatar.set_vexpand(false);
      avatar.set_can_shrink(true);
      avatar.set_size_request(40, 40);
      avatar.set_valign(gtk::Align::Center);
      row.append(&avatar);
    }
  }
  let texts = gtk::Box::new(gtk::Orientation::Vertical, 2);
  texts.set_hexpand(true);
  texts.set_valign(gtk::Align::Center);
  let title = super::markup_label(&lang::t("sidebar.signin.title"), 13, "bold", fg);
  title.set_halign(gtk::Align::Start);
  title.set_xalign(0.0);
  texts.append(&title);
  let subtitle = super::markup_label(&lang::t("sidebar.signin.subtitle"), 12, "normal", secondary);
  subtitle.set_halign(gtk::Align::Start);
  subtitle.set_xalign(0.0);
  subtitle.set_ellipsize(gtk::pango::EllipsizeMode::End);
  texts.append(&subtitle);
  row.append(&texts);
  row
}

/// Depth-first search for the sidebar search field.
fn find_search_entry(widget: &gtk::Widget) -> Option<gtk::SearchEntry> {
  let mut child = widget.first_child();
  while let Some(current) = child {
    if let Ok(entry) = current.clone().downcast::<gtk::SearchEntry>() {
      return Some(entry);
    }
    if let Some(found) = find_search_entry(&current) {
      return Some(found);
    }
    child = current.next_sibling();
  }
  None
}

/// Insert the sign-in header right above the sidebar search field. The
/// TontooUI Sidebar has no header slot, so the row goes into the built
/// GTK tree in front of the search container; when the structure is
/// unexpected the row is skipped.
fn inject_signin(sidebar_gtk: &gtk::Widget, fg: &str, secondary: &str) {
  let Some(entry) = find_search_entry(sidebar_gtk) else {
    println!("Sign-in header skipped: search field not found");
    return;
  };
  let Some(search_box) = entry.parent() else {
    println!("Sign-in header skipped: search field has no parent");
    return;
  };
  let Some(container) = search_box
    .parent()
    .and_then(|w| w.downcast::<gtk::Box>().ok())
  else {
    println!("Sign-in header skipped: search container has no box parent");
    return;
  };
  let prev = search_box.prev_sibling();
  container.insert_child_after(&signin_row(fg, secondary), prev.as_ref());
}

impl Widget for SettingsRoot {
  fn id(&self) -> WidgetId {
    self.id
  }

  fn children(&self) -> Vec<&dyn Widget> {
    vec![&self.sidebar as &dyn Widget]
  }

  fn to_gtk(&self) -> gtk::Widget {
    let dark = is_dark();
    let fg = if dark { "#F5F5F7" } else { "#1E1E1E" };
    let secondary = if dark { "#A1A1A6" } else { "#6E6E73" };

    let outer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    outer.set_hexpand(true);
    outer.set_vexpand(true);

    let sidebar_gtk = self.sidebar.to_gtk();
    inject_signin(&sidebar_gtk, fg, secondary);
    outer.append(&sidebar_gtk);

    // Detail column: toolbar (back/forward + page title) above the page.
    let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
    column.set_hexpand(true);
    column.set_vexpand(true);

    let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    toolbar.set_hexpand(true);
    toolbar.set_margin_top(16);
    toolbar.set_margin_bottom(8);
    toolbar.set_margin_start(24);
    toolbar.set_margin_end(24);

    // Back/forward segment: two TontooUI Glass buttons joined into one
    // filled control. The radius override is attached to each button
    // directly (ancestor providers do not reach descendants here).
    let chevron_tint = || {
      if dark {
        Color::from_rgb(245, 245, 247)
      } else {
        Color::from_rgb(30, 30, 30)
      }
    };
    let seg = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    seg.set_valign(gtk::Align::Center);
    let nav_back = self.nav.clone();
    let back_button = Button::new("‹")
      .style(ButtonStyle::Glass)
      .tint(chevron_tint())
      .on_click(move || {
        nav_back.lock().unwrap().back();
      });
    let nav_forward = self.nav.clone();
    let forward_button = Button::new("›")
      .style(ButtonStyle::Glass)
      .tint(chevron_tint())
      .on_click(move || {
        nav_forward.lock().unwrap().forward();
      });
    let back = back_button.to_gtk();
    let forward = forward_button.to_gtk();
    apply_css(
      &back,
      "button.seg-first { border-top-right-radius: 0px; border-bottom-right-radius: 0px; border-right: none; }",
    );
    apply_css(
      &forward,
      "button.seg-last { border-top-left-radius: 0px; border-bottom-left-radius: 0px; }",
    );
    back.add_css_class("seg-first");
    forward.add_css_class("seg-last");
    back.set_sensitive(false);
    forward.set_sensitive(false);
    back.set_valign(gtk::Align::Center);
    forward.set_valign(gtk::Align::Center);
    seg.append(&back);
    seg.append(&forward);
    toolbar.append(&seg);

    let separator = gtk::Separator::new(gtk::Orientation::Vertical);
    toolbar.append(&separator);

    let title = gtk::Label::new(None);
    title.set_use_markup(true);
    title.set_halign(gtk::Align::Start);
    title.set_markup(&title_markup(
      self.nav.lock().unwrap().selected,
      fg,
    ));
    toolbar.append(&title);
    column.append(&toolbar);

    let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
    detail.set_hexpand(true);
    detail.set_vexpand(true);
    let mut shown = self.nav.lock().unwrap().selected;
    detail.append(page_for(
      shown,
      &self.wifi_page,
      &self.bluetooth_page,
      &self.network_page,
      &self.battery_page,
      &self.general_page,
      &self.accessibility_page,
      &self.appearance_page,
      &self.desktop_dock_page,
      &self.displays_page,
      &self.menu_bar_page,
      &self.tinti_ai_page,
      &self.spotlight_page,
      &self.wallpaper_page,
      &self.notifications_page,
      &self.sound_page,
      &self.focus_page,
      &self.screen_time_page,
      &self.lock_screen_page,
      &self.privacy_page,
      &self.touch_id_page,
      &self.users_page,
      &self.internet_accounts_page,
      &self.octo_cloud_page,
      &self.keyboard_page,
      &self.mouse_page,
      &self.printers_page,
      &self.app_settings_page,
      &self.developer_page,
      &self.about_page,
    ));
    column.append(&detail);
    outer.append(&column);

    // Main-thread poller: swaps the page, refreshes the title and the
    // button sensitivity when the selection changes; exits once these
    // containers leave the window.
    let column_poller = column.clone();
    let detail_poller = detail.clone();
    let title_poller = title.clone();
    let back_poller = back.clone();
    let forward_poller = forward.clone();
    let wifi_poller = self.wifi_page.clone();
    let bluetooth_poller = self.bluetooth_page.clone();
    let network_poller = self.network_page.clone();
    let battery_poller = self.battery_page.clone();
    let general_poller = self.general_page.clone();
    let accessibility_poller = self.accessibility_page.clone();
    let appearance_poller = self.appearance_page.clone();
    let desktop_dock_poller = self.desktop_dock_page.clone();
    let displays_poller = self.displays_page.clone();
    let menu_bar_poller = self.menu_bar_page.clone();
    let tinti_ai_poller = self.tinti_ai_page.clone();
    let spotlight_poller = self.spotlight_page.clone();
    let wallpaper_poller = self.wallpaper_page.clone();
    let notifications_poller = self.notifications_page.clone();
    let sound_poller = self.sound_page.clone();
    let focus_poller = self.focus_page.clone();
    let screen_time_poller = self.screen_time_page.clone();
    let lock_screen_poller = self.lock_screen_page.clone();
    let privacy_poller = self.privacy_page.clone();
    let touch_id_poller = self.touch_id_page.clone();
    let users_poller = self.users_page.clone();
    let internet_accounts_poller = self.internet_accounts_page.clone();
    let octo_cloud_poller = self.octo_cloud_page.clone();
    let keyboard_poller = self.keyboard_page.clone();
    let mouse_poller = self.mouse_page.clone();
    let printers_poller = self.printers_page.clone();
    let app_settings_poller = self.app_settings_page.clone();
    let developer_poller = self.developer_page.clone();
    let about_poller = self.about_page.clone();
    let nav_poller = self.nav.clone();
    let fg_poller = fg;
    glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
      if column_poller.root().is_none() {
        return glib::ControlFlow::Break;
      }
      let state = nav_poller.lock().unwrap();
      let want = state.selected;
      let can_back = state.can_back();
      let can_forward = state.can_forward();
      drop(state);
      if want != shown {
        shown = want;
        if let Some(child) = detail_poller.first_child() {
          detail_poller.remove(&child);
        }
        detail_poller.append(page_for(
          want,
          &wifi_poller,
          &bluetooth_poller,
          &network_poller,
          &battery_poller,
          &general_poller,
          &accessibility_poller,
          &appearance_poller,
          &desktop_dock_poller,
          &displays_poller,
          &menu_bar_poller,
          &tinti_ai_poller,
          &spotlight_poller,
          &wallpaper_poller,
          &notifications_poller,
          &sound_poller,
          &focus_poller,
          &screen_time_poller,
          &lock_screen_poller,
          &privacy_poller,
          &touch_id_poller,
          &users_poller,
          &internet_accounts_poller,
          &octo_cloud_poller,
          &keyboard_poller,
          &mouse_poller,
          &printers_poller,
          &app_settings_poller,
          &developer_poller,
          &about_poller,
        ));
        title_poller.set_markup(&title_markup(want, fg_poller));
      }
      back_poller.set_sensitive(can_back);
      forward_poller.set_sensitive(can_forward);
      glib::ControlFlow::Continue
    });

    outer.upcast()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn nav_history_go_back_forward() {
    let mut nav = NavState::default();
    assert!(!nav.can_back());
    assert!(!nav.can_forward());
    nav.go(0);
    nav.go(2);
    nav.go(5);
    assert!(nav.can_back());
    assert!(!nav.can_forward());
    assert!(nav.back());
    assert_eq!(nav.selected, 2);
    assert!(nav.forward());
    assert_eq!(nav.selected, 5);
  }

  #[test]
  fn nav_branch_truncates_forward() {
    let mut nav = NavState::default();
    nav.go(0);
    nav.go(1);
    nav.go(2);
    nav.back();
    nav.go(5);
    assert!(!nav.can_forward());
    assert_eq!(nav.selected, 5);
  }
}
