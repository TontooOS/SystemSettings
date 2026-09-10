//! App Settings page for SystemSettings (example content).
//!
//! Header with the bundled Launchpad PNG plus example rows (Default
//! Apps, Auto Update toggle). All text uses SF Pro Display and both
//! `en_us` and `de_de` strings.

use super::{is_dark, markup_label, palette};
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::apply_css;
use crate::UIKit::prelude::*;
use gtk::prelude::*;

const HEADER_ICON_PX: i32 = 32;

/// Bundled Launchpad artwork (`Resources/launchpad.png`).
pub(crate) fn launchpad_png() -> String {
  format!(
    "{}/Resources/launchpad.png",
    env!("CARGO_MANIFEST_DIR")
  )
}

/// Example info row: label on the left, detail on the right.
fn info_row(label_key: &str, detail_key: &str, pal_fg: &str, pal_secondary: &str, last: bool) -> gtk::Box {
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

  let detail = markup_label(&lang::t(detail_key), 13, "normal", pal_secondary);
  detail.set_halign(gtk::Align::End);
  row.append(&detail);

  if !last {
    crate::UIKit::apply_css(
      &row,
      "box { border-bottom: 1px solid rgba(128,128,128,0.25); }",
    );
  }
  row
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

/// The App Settings detail page (example content, on screen).
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(is_dark());

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  // Header row: bundled PNG icon, title + subtitle.
  let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  header.set_hexpand(true);

  let icon_path = launchpad_png();
  if std::path::Path::new(&icon_path).exists() {
    let icon = gtk::Image::from_file(&icon_path);
    icon.set_pixel_size(HEADER_ICON_PX);
    icon.set_valign(gtk::Align::Start);
    header.append(&icon);
  }

  let titles = gtk::Box::new(gtk::Orientation::Vertical, 2);
  titles.set_hexpand(true);
  titles.set_halign(gtk::Align::Fill);
  let title = markup_label(&lang::t("app_settings.title"), 17, "bold", pal.fg);
  title.set_halign(gtk::Align::Start);
  title.set_xalign(0.0);
  titles.append(&title);
  let subtitle = markup_label(
    &lang::t("app_settings.header.subtitle"),
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
  rows.append(&info_row(
    "app_settings.default_apps",
    "app_settings.default_apps.detail",
    pal.fg,
    pal.secondary,
    false,
  ));
  rows.append(&toggle_row(
    "app_settings.auto_update",
    true,
    "Auto Update",
    pal.fg,
    true,
  ));
  detail.append(&rows);

  detail.upcast()
}
