//! Network settings page for SystemSettings.
//!
//! Header plus DNS card (click-to-edit, applied system-wide through the
//! daemon) and example wired rows (Ethernet, phone over USB-C) with
//! on/off toggles. All text uses SF Pro Display and both `en_us` and
//! `de_de` strings.

use super::{WIFI_BLUE, is_dark, markup_label, palette, sidebar_style_icon_path};
use crate::daemon;
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::prelude::*;
use gtk::prelude::*;
use std::rc::Rc;

const HEADER_ICON_PX: i32 = 32;

/// Suggested manual servers when switching from DHCP.
const DEFAULT_DNS_INPUT: &str = "1.1.1.1, 8.8.8.8";

/// Blue `network` icon, same artwork as the sidebar row icon.
fn network_icon_path() -> Option<String> {
  sidebar_style_icon_path("network", "network", WIFI_BLUE)
}

/// Inline markup matching `markup_label`, for updating labels in place.
fn span(text: &str, size: u32, weight: &str, color: &str) -> String {
  format!(
    "<span font_desc=\"{} {} {}\" foreground=\"{}\">{}</span>",
    super::SF_PRO,
    weight,
    size,
    color,
    glib::markup_escape_text(text),
  )
}

/// User-facing text for a `dns_set` failure: validation errors get the
/// format hint, missing NetworkManager/hardware gets the unavailable
/// note, anything else passes through raw.
fn dns_error_text(e: &str) -> String {
  if e.contains("invalid IPv4") {
    format!("{} ({})", lang::t("network.dns.invalid"), e)
  } else if e.contains("not available") || e.contains("no active connection") {
    format!("{} ({})", lang::t("network.dns.unavailable"), e)
  } else {
    e.to_string()
  }
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

/// DNS card: single big title, clickable value, inline editor.
/// Clicking the value turns it into a text field prefilled with the
/// current servers (or `1.1.1.1, 8.8.8.8` on DHCP); Enter or leaving the
/// field saves through the daemon (empty means DHCP) and the card shows
/// the effective state.
fn build_dns_card(fg: &'static str, secondary: &'static str, card_color: &str) -> gtk::Box {
  let card_box = card(card_color);

  let title_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  title_row.set_hexpand(true);
  title_row.set_valign(gtk::Align::Center);
  title_row.set_margin_top(5);
  title_row.set_margin_bottom(5);
  let title = markup_label(&lang::t("network.dns"), 15, "bold", fg);
  title.set_halign(gtk::Align::Start);
  title.set_xalign(0.0);
  title.set_hexpand(true);
  title_row.append(&title);
  let mode = markup_label("", 13, "normal", secondary);
  mode.set_halign(gtk::Align::End);
  mode.set_valign(gtk::Align::Center);
  title_row.append(&mode);
  card_box.append(&title_row);

  let value = markup_label("", 13, "normal", secondary);
  value.set_halign(gtk::Align::Start);
  value.set_xalign(0.0);
  value.set_hexpand(true);
  value.set_margin_bottom(5);
  value.set_focusable(true);
  if let Some(cursor) = gtk::gdk::Cursor::from_name("pointer", None) {
    value.set_cursor(Some(&cursor));
  }
  card_box.append(&value);

  let entry = gtk::Entry::new();
  entry.set_hexpand(true);
  entry.set_margin_bottom(5);
  entry.set_visible(false);
  card_box.append(&entry);

  let hint = markup_label(&lang::t("network.dns.hint"), 12, "normal", secondary);
  hint.set_halign(gtk::Align::Start);
  hint.set_xalign(0.0);
  hint.set_margin_bottom(5);
  hint.set_visible(false);
  card_box.append(&hint);

  let error = markup_label("", 12, "normal", "#FF453A");
  error.set_halign(gtk::Align::Start);
  error.set_xalign(0.0);
  error.set_wrap(true);
  error.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  error.set_margin_bottom(5);
  error.set_visible(false);
  card_box.append(&error);

  // Reload the daemon state into the labels.
  let value_r = value.clone();
  let mode_r = mode.clone();
  let refresh: Rc<dyn Fn()> = Rc::new(move || {
    let state = daemon::dns_get().unwrap_or(daemon::DnsState {
      servers: Vec::new(),
      manual: false,
    });
    if state.manual && !state.servers.is_empty() {
      value_r.set_markup(&span(&state.servers.join(", "), 13, "normal", secondary));
      mode_r.set_markup(&span("", 13, "normal", secondary));
    } else {
      value_r.set_markup(&span(&lang::t("network.dns.automatic"), 13, "normal", secondary));
      mode_r.set_markup(&span(&lang::t("network.dns.automatic"), 13, "normal", secondary));
    }
  });
  refresh();

  // Save the editor content through the daemon; empty means DHCP.
  let value_s = value.clone();
  let entry_s = entry.clone();
  let hint_s = hint.clone();
  let error_s = error.clone();
  let refresh_s = Rc::clone(&refresh);
  let save: Rc<dyn Fn()> = Rc::new(move || {
    if !gtk::prelude::WidgetExt::is_visible(&entry_s) {
      return;
    }
    match daemon::dns_set(&entry_s.text().to_string()) {
      Ok(_) => {
        refresh_s();
        entry_s.set_visible(false);
        hint_s.set_visible(false);
        error_s.set_visible(false);
        value_s.set_visible(true);
      }
      Err(e) => {
        error_s.set_markup(&span(&dns_error_text(&e), 12, "normal", "#FF453A"));
        error_s.set_visible(true);
      }
    }
  });

  // Click the value to edit: prefill current servers, or the suggested
  // defaults when on DHCP.
  let value_c = value.clone();
  let entry_c = entry.clone();
  let hint_c = hint.clone();
  let error_c = error.clone();
  let click = gtk::GestureClick::new();
  click.set_button(1);
  click.connect_released(move |_, _, _, _| {
    let state = daemon::dns_get().unwrap_or(daemon::DnsState {
      servers: Vec::new(),
      manual: false,
    });
    if state.manual && !state.servers.is_empty() {
      entry_c.set_text(&state.servers.join(", "));
    } else {
      entry_c.set_text(DEFAULT_DNS_INPUT);
    }
    value_c.set_visible(false);
    error_c.set_visible(false);
    entry_c.set_visible(true);
    hint_c.set_visible(true);
    entry_c.grab_focus();
  });
  value.add_controller(click);

  // Enter saves; leaving the field saves too.
  let save_enter = Rc::clone(&save);
  entry.connect_activate(move |_| save_enter());
  let save_leave = Rc::clone(&save);
  let focus = gtk::EventControllerFocus::new();
  focus.connect_leave(move |_| save_leave());
  entry.add_controller(focus);

  card_box
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
  header_card.append(&header);
  detail.append(&header_card);

  let gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap.set_size_request(-1, 16);
  detail.append(&gap);

  // DNS card: single big title with click-to-edit value below.
  detail.append(&build_dns_card(pal.fg, pal.secondary, pal.card));

  let gap2 = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap2.set_size_request(-1, 12);
  detail.append(&gap2);

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dns_validation_error_gets_format_hint() {
    let text = dns_error_text("dns set failed: invalid IPv4 address: nope");
    assert!(text.contains(&lang::t("network.dns.invalid")));
    assert!(text.contains("invalid IPv4 address: nope"));
  }

  #[test]
  fn dns_missing_tool_error_gets_unavailable_note() {
    let text = dns_error_text("dns set failed: Network hardware or tool not available");
    assert!(text.contains(&lang::t("network.dns.unavailable")));
    assert!(!text.contains(&lang::t("network.dns.invalid")));
  }

  #[test]
  fn dns_no_connection_error_gets_unavailable_note() {
    let text = dns_error_text("dns set failed: no active connection");
    assert!(text.contains(&lang::t("network.dns.unavailable")));
  }

  #[test]
  fn dns_other_errors_pass_through() {
    assert_eq!(dns_error_text("boom"), "boom");
  }
}
