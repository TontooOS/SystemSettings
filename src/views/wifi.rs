//! Wi-Fi settings page for SystemSettings.
//!
//! Live network list from the settings daemon (blue icon, signal bars,
//! name, lock for secured networks) with example fallback rows.
//! Clicking a row opens the join dialog (password entry for secured
//! networks). All text uses SF Pro Display and both `en_us` and `de_de`
//! strings.

use super::{BADGE_GRAY, WIFI_BLUE, is_dark, markup_label, palette, sidebar_style_icon_path};
use crate::daemon;
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::apply_css;
use crate::UIKit::prelude::*;
use gtk::prelude::*;

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

/// One network row for the list.
struct NetworkRow {
  ssid: String,
  signal_pct: i32,
  secured: bool,
}

fn is_secured(security: &str) -> bool {
  !security.trim().is_empty() && security.trim().to_uppercase() != "OPEN"
}

/// Live networks from the daemon, example rows when unreachable.
fn network_rows() -> Vec<NetworkRow> {
  if let Ok(networks) = daemon::list() {
    return networks
      .into_iter()
      .map(|n| NetworkRow {
        ssid: n.ssid,
        signal_pct: n.signal_pct,
        secured: is_secured(&n.security),
      })
      .collect();
  }
  vec![
    NetworkRow {
      ssid: lang::t("wifi.row.home"),
      signal_pct: 82,
      secured: true,
    },
    NetworkRow {
      ssid: lang::t("wifi.row.lab"),
      signal_pct: 64,
      secured: true,
    },
  ]
}

/// Join dialog for one network: password entry for secured networks,
/// direct connect for open ones.
fn open_join_window(ssid: &str, secured: bool, fg: &str, card: &str) {
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
      Ok(_) => win_connect.close(),
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

/// The Wi-Fi detail page: header row plus the network list, directly on
/// the screen.
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(is_dark());

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  // Header row: blue Wi-Fi icon, title + subtitle, toggle on the right.
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

  let toggle = Toggle::new("")
    .value(true)
    .width(52.0)
    .on_change(|on| println!("Wi-Fi toggled: {}", on));
  let toggle_gtk = toggle.to_gtk();
  toggle_gtk.set_halign(gtk::Align::End);
  toggle_gtk.set_valign(gtk::Align::Start);
  toggle_gtk.set_vexpand(false);
  header.append(&toggle_gtk);
  detail.append(&header);

  let gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap.set_size_request(-1, 16);
  detail.append(&gap);

  // Network rows: compact rows with blue icon, signal bars, name and
  // lock for secured networks.
  let section = markup_label(&lang::t("wifi.networks.header"), 12, "normal", pal.secondary);
  section.set_halign(gtk::Align::Start);
  section.set_margin_bottom(2);
  detail.append(&section);

  let list_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
  list_box.set_hexpand(true);

  let rows = network_rows();
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

    row_box.append(&signal_bars(row.signal_pct, pal.fg, pal.secondary));

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
    let click = gtk::GestureClick::new();
    click.set_button(1);
    click.connect_released(move |_, _, _, _| {
      open_join_window(&ssid, secured, fg, card);
    });
    row_box.add_controller(click);

    list_box.append(&row_box);
  }
  detail.append(&list_box);

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
