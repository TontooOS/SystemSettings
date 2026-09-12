//! Wi-Fi settings page for SystemSettings.
//!
//! Header with a radio toggle, a Known Networks section backed by the
//! daemon system store and a live scan list. Without a wireless adapter
//! both sections show the no-hardware note; with the radio off only the
//! header stays visible. Clicking a row opens the join dialog (password
//! entry for secured networks); a successful connect is stored as known
//! by the daemon (encrypted, system-wide) and auto-joined at startup.
//! All text uses SF Pro Display and both `en_us` and `de_de` strings.

use super::{BADGE_GRAY, WIFI_BLUE, Palette, is_dark, markup_label, palette, sidebar_style_icon_path};
use crate::daemon;
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::apply_css;
use crate::UIKit::prelude::*;
use gtk::prelude::*;
use std::rc::Rc;
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

const HEADER_ICON_PX: i32 = 32;
const ROW_ICON_PX: i32 = 22;
const LOCK_ICON_PX: i32 = 14;

/// Blue `wifi` icon, same artwork as the sidebar row icon.
fn wifi_icon_path() -> Option<String> {
  sidebar_style_icon_path("wifi", "wifi", WIFI_BLUE)
}

/// Small gray badge icon (sidebar style) for the `lock.fill` glyph on
/// secured networks.
fn badge_icon_path() -> Option<String> {
  sidebar_style_icon_path("lock.fill", "badge_lock", BADGE_GRAY)
}

/// Four signal bars, filled according to `signal_pct` (0-100).
fn signal_bars(signal_pct: i32, filled_hex: &str, empty_hex: &str) -> gtk::Widget {
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
  row.set_valign(gtk::Align::Center);
  let filled = ((signal_pct.clamp(0, 100) + 24) / 25).clamp(0, 4);
  for (i, height) in [5, 9, 13, 17].iter().enumerate() {
    let bar = gtk::Box::new(gtk::Orientation::Vertical, 0);
    bar.set_size_request(4, *height);
    bar.set_valign(gtk::Align::End);
    let color = if (i as i32) < filled {
      filled_hex
    } else {
      empty_hex
    };
    apply_css(
      &bar,
      &format!("box {{ background-color: {}; border-radius: 1px; }}", color),
    );
    row.append(&bar);
  }
  row.upcast()
}

/// One network row for the list. Known networks carry no live signal, so
/// `signal_pct` is `None` and the bars are hidden.
struct NetworkRow {
  ssid: String,
  signal_pct: Option<i32>,
  secured: bool,
}

fn is_secured(security: &str) -> bool {
  !security.trim().is_empty() && security.trim().to_uppercase() != "OPEN"
}

/// Page state resolved from the daemon.
enum PageState {
  /// Daemon unreachable: header on plus example rows (dev fallback).
  Unreachable,
  /// No wireless adapter: both sections with the no-hardware note.
  NoAdapter,
  /// Radio off: header with the toggle off, no sections below.
  Off,
  /// Radio on: known networks plus the live scan.
  On {
    known: Vec<NetworkRow>,
    networks: Vec<NetworkRow>,
  },
}

fn example_rows() -> Vec<NetworkRow> {
  vec![
    NetworkRow {
      ssid: lang::t("wifi.row.home"),
      signal_pct: Some(82),
      secured: true,
    },
    NetworkRow {
      ssid: lang::t("wifi.row.lab"),
      signal_pct: Some(64),
      secured: true,
    },
  ]
}

fn resolve_state() -> PageState {
  let state = match daemon::status() {
    Ok(state) => state,
    Err(_) => return PageState::Unreachable,
  };
  if !state.available {
    return PageState::NoAdapter;
  }
  if !state.enabled {
    return PageState::Off;
  }
  let known = daemon::known_list()
    .unwrap_or_default()
    .into_iter()
    .map(|k| NetworkRow {
      ssid: k.ssid,
      signal_pct: None,
      secured: is_secured(&k.security),
    })
    .collect();
  let networks = match daemon::list() {
    Ok(networks) => networks
      .into_iter()
      .map(|n| NetworkRow {
        ssid: n.ssid,
        signal_pct: Some(n.signal_pct),
        secured: is_secured(&n.security),
      })
      .collect(),
    Err(_) => example_rows(),
  };
  PageState::On { known, networks }
}

/// Join dialog for one network: password entry for secured networks,
/// direct connect for open ones. `refresh` re-renders the page after a
/// successful connect (the daemon stored the network as known).
fn open_join_window(
  ssid: &str,
  secured: bool,
  fg: &str,
  card: &str,
  refresh: &Rc<dyn Fn()>,
) {
  let win = gtk::Window::new();
  win.set_title(Some(ssid));
  win.set_modal(true);
  win.set_resizable(false);
  win.set_default_size(320, 0);
  apply_css(&win, &format!("window {{ background-color: {}; }}", card));

  let body = gtk::Box::new(gtk::Orientation::Vertical, 10);
  body.set_margin_top(16);
  body.set_margin_bottom(16);
  body.set_margin_start(16);
  body.set_margin_end(16);

  let name = markup_label(ssid, 15, "bold", fg);
  name.set_halign(gtk::Align::Start);
  body.append(&name);

  let entry = gtk::Entry::new();
  if secured {
    entry.set_placeholder_text(Some(&lang::t("wifi.join.password")));
    entry.set_visibility(false);
    entry.set_input_purpose(gtk::InputPurpose::Password);
    body.append(&entry);
  }

  let error = markup_label("", 12, "normal", "#FF453A");
  error.set_halign(gtk::Align::Start);
  error.set_visible(false);
  body.append(&error);

  let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  buttons.set_halign(gtk::Align::End);
  let cancel = gtk::Button::with_label(&lang::t("wifi.join.cancel"));
  let connect_btn = gtk::Button::with_label(&lang::t("wifi.join.connect"));
  connect_btn.add_css_class("suggested-action");
  buttons.append(&cancel);
  buttons.append(&connect_btn);
  body.append(&buttons);

  win.set_child(Some(&body));

  let win_cancel = win.clone();
  cancel.connect_clicked(move |_| win_cancel.close());

  let win_connect = win.clone();
  let error_connect = error.clone();
  let ssid_owned = ssid.to_string();
  let refreshed = Rc::clone(refresh);
  connect_btn.connect_clicked(move |_| {
    let password = if secured {
      let text = entry.text().to_string();
      if text.is_empty() {
        error_connect.set_markup(&format!(
          "<span font_desc=\"{} normal 12\" foreground=\"#FF453A\">{}</span>",
          super::SF_PRO,
          glib::markup_escape_text(&lang::t("wifi.join.password_required")),
        ));
        error_connect.set_visible(true);
        return;
      }
      Some(text)
    } else {
      None
    };
    match daemon::connect(&ssid_owned, password.as_deref(), false) {
      Ok(_) => {
        refreshed();
        win_connect.close();
      }
      Err(e) => {
        error_connect.set_markup(&format!(
          "<span font_desc=\"{} normal 12\" foreground=\"#FF453A\">{}</span>",
          super::SF_PRO,
          glib::markup_escape_text(&e),
        ));
        error_connect.set_visible(true);
      }
    }
  });

  win.present();
}

/// Small section header (e.g. "Known Networks").
fn section_header(title: &str, pal: &Palette) -> gtk::Widget {
  let section = markup_label(title, 12, "normal", pal.secondary);
  section.set_halign(gtk::Align::Start);
  section.set_margin_bottom(2);
  section.upcast()
}

/// Wrapped secondary note (e.g. the no-adapter message).
fn note_label(text: &str, pal: &Palette) -> gtk::Widget {
  let note = markup_label(text, 13, "normal", pal.secondary);
  note.set_halign(gtk::Align::Start);
  note.set_xalign(0.0);
  note.set_wrap(true);
  note.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  note.set_margin_top(4);
  note.set_margin_bottom(8);
  note.upcast()
}

/// Append clickable network rows to `list_box`. `refresh` re-renders the
/// page after a successful join.
fn append_network_rows(
  list_box: &gtk::Box,
  rows: &[NetworkRow],
  pal: &Palette,
  refresh: &Rc<dyn Fn()>,
) {
  let last = rows.len().saturating_sub(1);
  for (index, row) in rows.iter().enumerate() {
    let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row_box.set_hexpand(true);
    row_box.set_margin_top(5);
    row_box.set_margin_bottom(5);
    row_box.set_focusable(true);
    if let Some(cursor) = gtk::gdk::Cursor::from_name("pointer", None) {
      row_box.set_cursor(Some(&cursor));
    }
    if index != last {
      apply_css(
        &row_box,
        "box { border-bottom: 1px solid rgba(128,128,128,0.25); }",
      );
    }

    if let Some(icon_path) = wifi_icon_path() {
      let icon = gtk::Image::from_file(&icon_path);
      icon.set_pixel_size(ROW_ICON_PX);
      icon.set_valign(gtk::Align::Center);
      row_box.append(&icon);
    }

    if let Some(signal_pct) = row.signal_pct {
      row_box.append(&signal_bars(signal_pct, pal.fg, pal.secondary));
    }

    let name = markup_label(&row.ssid, 13, "normal", pal.fg);
    name.set_halign(gtk::Align::Start);
    name.set_xalign(0.0);
    name.set_hexpand(true);
    name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    row_box.append(&name);

    if row.secured {
      if let Some(lock_path) = badge_icon_path() {
        let lock = gtk::Image::from_file(&lock_path);
        lock.set_pixel_size(LOCK_ICON_PX);
        lock.set_valign(gtk::Align::Center);
        row_box.append(&lock);
      }
    }

    let ssid = row.ssid.clone();
    let secured = row.secured;
    let fg = pal.fg;
    let card = pal.card;
    let refreshed = Rc::clone(refresh);
    let click = gtk::GestureClick::new();
    click.set_button(1);
    click.connect_released(move |_, _, _, _| {
      open_join_window(&ssid, secured, fg, card, &refreshed);
    });
    row_box.add_controller(click);

    list_box.append(&row_box);
  }
}

/// Fill `detail` for the current daemon state. Called on first build and
/// after every radio or connection change. `refresh_flag` is set by the
/// radio toggle (whose handler must be Send + Sync and cannot touch GTK);
/// the poller in `build_page` picks it up and re-renders here.
fn render(detail: &gtk::Box, refresh_flag: &Arc<AtomicBool>) {
  while let Some(child) = detail.first_child() {
    detail.remove(&child);
  }
  let pal = palette(is_dark());
  let state = resolve_state();

  // Header row: blue Wi-Fi icon, title + subtitle, toggle on the right.
  // Without an adapter the toggle is off and insensitive; with the radio
  // off it is off; otherwise it reflects the radio state.
  let (toggle_on, toggle_live) = match &state {
    PageState::Unreachable => (true, true),
    PageState::NoAdapter => (false, false),
    PageState::Off => (false, true),
    PageState::On { .. } => (true, true),
  };

  let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  header.set_hexpand(true);

  if let Some(icon_path) = wifi_icon_path() {
    let icon = gtk::Image::from_file(&icon_path);
    icon.set_pixel_size(HEADER_ICON_PX);
    icon.set_valign(gtk::Align::Start);
    header.append(&icon);
  }

  let titles = gtk::Box::new(gtk::Orientation::Vertical, 2);
  titles.set_hexpand(true);
  titles.set_halign(gtk::Align::Fill);
  let title = markup_label(&lang::t("wifi.title"), 17, "bold", pal.fg);
  title.set_halign(gtk::Align::Start);
  title.set_xalign(0.0);
  titles.append(&title);
  let subtitle = markup_label(&lang::t("wifi.header.subtitle"), 13, "normal", pal.secondary);
  subtitle.set_halign(gtk::Align::Start);
  subtitle.set_xalign(0.0);
  subtitle.set_wrap(true);
  subtitle.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  subtitle.set_max_width_chars(48);
  titles.append(&subtitle);
  header.append(&titles);

  // The TontooUI toggle handler must be Send + Sync, so the GTK tree
  // cannot be captured. It applies the radio state and raises the refresh
  // flag; the poller in `build_page` re-renders on the main thread.
  let refresh_raised = Arc::clone(refresh_flag);
  let toggle = Toggle::new("")
    .value(toggle_on)
    .width(52.0)
    .on_change(move |on| {
      let _ = daemon::set_enabled(on);
      refresh_raised.store(true, Ordering::SeqCst);
    });
  let toggle_gtk = toggle.to_gtk();
  toggle_gtk.set_halign(gtk::Align::End);
  toggle_gtk.set_valign(gtk::Align::Start);
  toggle_gtk.set_vexpand(false);
  toggle_gtk.set_sensitive(toggle_live);
  header.append(&toggle_gtk);
  detail.append(&header);

  let gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap.set_size_request(-1, 16);
  detail.append(&gap);

  let detail_refresh = detail.clone();
  let flag_refresh = Arc::clone(refresh_flag);
  let refresh: Rc<dyn Fn()> = Rc::new(move || render(&detail_refresh, &flag_refresh));

  match state {
    // Radio off: nothing below the header.
    PageState::Off => {}
    // No adapter: both sections with the no-hardware note.
    PageState::NoAdapter => {
      detail.append(&section_header(&lang::t("wifi.known.header"), &pal));
      detail.append(&note_label(&lang::t("wifi.no_adapter"), &pal));
      detail.append(&section_header(&lang::t("wifi.networks.header"), &pal));
      detail.append(&note_label(&lang::t("wifi.no_adapter"), &pal));
    }
    // Daemon unreachable: example rows under the Networks header.
    PageState::Unreachable => {
      detail.append(&section_header(&lang::t("wifi.networks.header"), &pal));
      let list_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
      list_box.set_hexpand(true);
      append_network_rows(&list_box, &example_rows(), &pal, &refresh);
      detail.append(&list_box);
    }
    // Radio on: known networks plus the live scan.
    PageState::On { known, networks } => {
      if !known.is_empty() {
        detail.append(&section_header(&lang::t("wifi.known.header"), &pal));
        let known_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        known_box.set_hexpand(true);
        append_network_rows(&known_box, &known, &pal, &refresh);
        detail.append(&known_box);

        let gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
        gap.set_size_request(-1, 12);
        detail.append(&gap);
      }
      detail.append(&section_header(&lang::t("wifi.networks.header"), &pal));
      let list_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
      list_box.set_hexpand(true);
      append_network_rows(&list_box, &networks, &pal, &refresh);
      detail.append(&list_box);
    }
  }
}

/// The Wi-Fi detail page: header row plus known and nearby networks,
/// directly on the screen.
pub(crate) fn build_page() -> gtk::Widget {
  let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);
  let refresh_flag = Arc::new(AtomicBool::new(false));
  // Poller for radio-toggle refreshes (see `render`): stops itself once
  // the page is destroyed.
  let weak = detail.downgrade();
  let raised = Arc::clone(&refresh_flag);
  let rendered = Arc::clone(&refresh_flag);
  glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
    match weak.upgrade() {
      Some(detail) => {
        if raised.swap(false, Ordering::SeqCst) {
          render(&detail, &rendered);
        }
        glib::ControlFlow::Continue
      }
      None => glib::ControlFlow::Break,
    }
  });
  render(&detail, &refresh_flag);
  detail.upcast()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn open_networks_have_no_lock() {
    assert!(!is_secured("OPEN"));
    assert!(!is_secured("open"));
    assert!(!is_secured(""));
  }

  #[test]
  fn secured_networks_show_lock() {
    assert!(is_secured("WPA2"));
    assert!(is_secured("WPA3"));
    assert!(is_secured("WEP"));
  }
}
