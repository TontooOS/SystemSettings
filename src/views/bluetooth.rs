//! Bluetooth settings page for SystemSettings (example content).
//!
//! Header with master toggle plus example device rows with on/off
//! toggles. All text uses SF Pro Display and both `en_us` and `de_de`
//! strings.

use super::{WIFI_BLUE, is_dark, markup_label, palette, sidebar_style_icon_path};
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::prelude::*;
use gtk::prelude::*;

const HEADER_ICON_PX: i32 = 32;

/// Blue Bluetooth icon (same artwork as the sidebar row icon).
fn bluetooth_icon_path() -> Option<String> {
  sidebar_style_icon_path(
    "antenna.radiowaves.left.and.right",
    "bluetooth",
    WIFI_BLUE,
  )
}

/// Example on/off row: label on the left, toggle on the right.
fn toggle_row(
  label: String,
  on: bool,
  log_line: &'static str,
  pal_fg: &str,
  last: bool,
) -> gtk::Box {
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  row.set_hexpand(true);
  row.set_margin_top(5);
  row.set_margin_bottom(5);

  let name = markup_label(&label, 13, "normal", pal_fg);
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

/// The Bluetooth detail page (example content, directly on the screen).
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(is_dark());

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  // Header row: blue Bluetooth icon, title + subtitle, master toggle.
  let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  header.set_hexpand(true);

  if let Some(icon_path) = bluetooth_icon_path() {
    let icon = gtk::Image::from_file(&icon_path);
    icon.set_pixel_size(HEADER_ICON_PX);
    icon.set_valign(gtk::Align::Start);
    header.append(&icon);
  }

  let titles = gtk::Box::new(gtk::Orientation::Vertical, 2);
  titles.set_hexpand(true);
  titles.set_halign(gtk::Align::Fill);
  let title = markup_label(&lang::t("bluetooth.title"), 17, "bold", pal.fg);
  title.set_halign(gtk::Align::Start);
  title.set_xalign(0.0);
  titles.append(&title);
  let subtitle = markup_label(
    &lang::t("bluetooth.header.subtitle"),
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

  let master = Toggle::new("")
    .value(true)
    .width(52.0)
    .on_change(|on| println!("Bluetooth toggled: {}", on));
  let master_gtk = master.to_gtk();
  master_gtk.set_halign(gtk::Align::End);
  master_gtk.set_valign(gtk::Align::Start);
  master_gtk.set_vexpand(false);
  header.append(&master_gtk);
  detail.append(&header);

  let gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap.set_size_request(-1, 16);
  detail.append(&gap);

  // Example devices with on/off toggles.
  let devices = markup_label(
    &lang::t("bluetooth.devices.header"),
    12,
    "normal",
    pal.secondary,
  );
  devices.set_halign(gtk::Align::Start);
  devices.set_margin_bottom(2);
  detail.append(&devices);

  let device_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
  device_box.set_hexpand(true);
  device_box.append(&toggle_row(
    lang::t("bluetooth.device.buds"),
    true,
    "Tontoo Buds",
    pal.fg,
    false,
  ));
  device_box.append(&toggle_row(
    lang::t("bluetooth.device.mouse"),
    false,
    "Tontoo Mouse",
    pal.fg,
    true,
  ));
  detail.append(&device_box);

  detail.upcast()
}
