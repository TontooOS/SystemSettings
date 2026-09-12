//! Network settings page for SystemSettings (example content).
//!
//! Header plus example rows for DNS Server, VPN and wired networks
//! (Ethernet, phone over USB-C). Wired rows carry on/off toggles.
//! All text uses SF Pro Display and both `en_us` and `de_de` strings.

use super::{WIFI_BLUE, is_dark, markup_label, palette, sidebar_style_icon_path};
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::prelude::*;
use gtk::prelude::*;

const HEADER_ICON_PX: i32 = 32;

/// Blue `network` icon, same artwork as the sidebar row icon.
fn network_icon_path() -> Option<String> {
  sidebar_style_icon_path("network", "network", WIFI_BLUE)
}

/// Rounded card container in the page palette color (same style as the
/// General/About/Wi-Fi pages).
fn card(pal_card: &str) -> gtk::Box {
  let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
  card.set_hexpand(true);
  crate::UIKit::apply_css(
    &card,
    &format!(
      "box {{ background-color: {}; border-radius: 12px; padding: 12px 16px; }}",
      pal_card
    ),
  );
  card
}

/// Example on/off row: label on the left, toggle on the right.
fn toggle_row(
  label_key: &str,
  on: bool,
  log_line: &'static str,
  pal_fg: &str,
  last: bool,
) -> gtk::Box {
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

/// The Network detail page (example content, directly on the screen).
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(is_dark());

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  // Header card: blue network icon, title + subtitle, master toggle.
  let header_card = card(pal.card);
  let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  header.set_hexpand(true);
  header.set_valign(gtk::Align::Center);

  if let Some(icon_path) = network_icon_path() {
    let icon = gtk::Image::from_file(&icon_path);
    icon.set_pixel_size(HEADER_ICON_PX);
    icon.set_valign(gtk::Align::Start);
    header.append(&icon);
  }

  let titles = gtk::Box::new(gtk::Orientation::Vertical, 2);
  titles.set_hexpand(true);
  titles.set_halign(gtk::Align::Fill);
  let title = markup_label(&lang::t("network.title"), 17, "bold", pal.fg);
  title.set_halign(gtk::Align::Start);
  title.set_xalign(0.0);
  titles.append(&title);
  let subtitle = markup_label(
    &lang::t("network.header.subtitle"),
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
    .on_change(|on| println!("Network toggled: {}", on));
  let master_gtk = master.to_gtk();
  master_gtk.set_halign(gtk::Align::End);
  master_gtk.set_valign(gtk::Align::Start);
  master_gtk.set_vexpand(false);
  header.append(&master_gtk);
  header_card.append(&header);
  detail.append(&header_card);

  let gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap.set_size_request(-1, 16);
  detail.append(&gap);

  // DNS server card with an example address on the right.
  let section = markup_label(&lang::t("network.dns"), 12, "normal", pal.secondary);
  section.set_halign(gtk::Align::Start);
  section.set_margin_bottom(2);
  detail.append(&section);

  let dns_card = card(pal.card);
  let dns_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  dns_row.set_hexpand(true);
  dns_row.set_valign(gtk::Align::Center);
  dns_row.set_margin_top(5);
  dns_row.set_margin_bottom(5);
  let dns_name = markup_label(&lang::t("network.dns"), 13, "normal", pal.fg);
  dns_name.set_halign(gtk::Align::Start);
  dns_name.set_hexpand(true);
  dns_row.append(&dns_name);
  let dns_value = markup_label(&lang::t("network.dns.detail"), 13, "normal", pal.secondary);
  dns_value.set_halign(gtk::Align::End);
  dns_row.append(&dns_value);
  dns_card.append(&dns_row);
  detail.append(&dns_card);

  let gap2 = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap2.set_size_request(-1, 12);
  detail.append(&gap2);

  // VPN card with an on/off toggle.
  let vpn_card = card(pal.card);
  vpn_card.append(&toggle_row("network.vpn", false, "VPN", pal.fg, true));
  detail.append(&vpn_card);

  let gap3 = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap3.set_size_request(-1, 12);
  detail.append(&gap3);

  // Wired networks: physical links with on/off toggles.
  let wired = markup_label(
    &lang::t("network.wired.header"),
    12,
    "normal",
    pal.secondary,
  );
  wired.set_halign(gtk::Align::Start);
  wired.set_margin_bottom(2);
  detail.append(&wired);

  let wired_card = card(pal.card);
  let wired_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
  wired_box.set_hexpand(true);
  wired_box.append(&toggle_row(
    "network.wired.ethernet",
    true,
    "Ethernet",
    pal.fg,
    false,
  ));
  wired_box.append(&toggle_row(
    "network.wired.iphone",
    false,
    "iPhone USB",
    pal.fg,
    true,
  ));
  wired_card.append(&wired_box);
  detail.append(&wired_card);

  detail.upcast()
}
