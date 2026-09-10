//! Notifications settings page for SystemSettings (example content).
//!
//! Header with a red notification icon plus example rows (Allow
//! Notifications toggle, Sounds toggle). All text uses SF Pro Display
//! and both `en_us` and `de_de` strings.

use super::{is_dark, markup_label, palette, sidebar_style_icon_path};
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::apply_css;
use crate::UIKit::prelude::*;
use gtk::prelude::*;

const HEADER_ICON_PX: i32 = 32;
const NOTIFICATIONS_RED: (u8, u8, u8) = (255, 69, 58);

/// Red `bell.badge.fill` icon, same artwork as the sidebar row icon.
fn notifications_icon_path() -> Option<String> {
  sidebar_style_icon_path("bell.badge.fill", "notifications", NOTIFICATIONS_RED)
}

/// Example on/off row: label on the left, toggle on the right.
fn toggle_row(label_key: &str, on: bool, log_line: &'static str, pal_fg: &str, last: bool) -> gtk::Box {
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  row.set_hexpand(true);
  row.set_margin_top(5);
  row.set_margin_bottom(5);

  let name = markup_label(&lang::t(label_key), 13, "normal", pal_fg);
  name.set_halign(gtk::Align::Start);
  name.set_xalign(0.0);
  name.set_hexpand(true);
  name.set_ellipsize(gtk::pango::EllipsizeMode::End);
  row.append(&name);

  let toggle = Toggle::new("").value(on).width(52.0).on_change(move |on| {
    println!("{} toggled: {}", log_line, on);
  });
  let toggle_gtk = toggle.to_gtk();
  toggle_gtk.set_halign(gtk::Align::End);
  toggle_gtk.set_valign(gtk::Align::Center);
  toggle_gtk.set_vexpand(false);
  row.append(&toggle_gtk);

  if !last {
    crate::UIKit::apply_css(
      &row,
      "box { border-bottom: 1px solid rgba(128,128,128,0.25); }",
    );
  }
  row
}

/// The Notifications detail page (example content, directly on screen).
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(is_dark());

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  // Header row: red notification icon, title + subtitle.
  let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  header.set_hexpand(true);

  if let Some(icon_path) = notifications_icon_path() {
    let icon = gtk::Image::from_file(&icon_path);
    icon.set_pixel_size(HEADER_ICON_PX);
    icon.set_valign(gtk::Align::Start);
    header.append(&icon);
  }

  let titles = gtk::Box::new(gtk::Orientation::Vertical, 2);
  titles.set_hexpand(true);
  titles.set_halign(gtk::Align::Fill);
  let title = markup_label(&lang::t("notifications.title"), 17, "bold", pal.fg);
  title.set_halign(gtk::Align::Start);
  title.set_xalign(0.0);
  titles.append(&title);
  let subtitle = markup_label(
    &lang::t("notifications.header.subtitle"),
    13,
    "normal",
    pal.secondary,
  );
  subtitle.set_halign(gtk::Align::Start);
  subtitle.set_xalign(0.0);
  subtitle.set_wrap(true);
  subtitle.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  subtitle.set_max_width_chars(48);
  titles.append(&subtitle);
  header.append(&titles);
  detail.append(&header);

  let gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap.set_size_request(-1, 16);
  detail.append(&gap);

  let rows = gtk::Box::new(gtk::Orientation::Vertical, 0);
  rows.set_hexpand(true);
  rows.append(&toggle_row(
    "notifications.allow",
    true,
    "Allow Notifications",
    pal.fg,
    false,
  ));
  rows.append(&toggle_row(
    "notifications.sounds",
    true,
    "Sounds",
    pal.fg,
    true,
  ));
  detail.append(&rows);

  detail.upcast()
}
