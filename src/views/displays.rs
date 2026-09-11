//! Displays settings page for SystemSettings.
//!
//! Output info plus live controls: brightness slider (dims the whole
//! desktop in the compositor), night light toggle (warm overlay) and a
//! refresh rate dropdown built from the monitor's reported modes (capped
//! at the monitor max). All values come from the settings daemon
//! (`display_get`) with defaults when it is unreachable; every change
//! applies live via `display_set` and persists there. All text uses
//! SF Pro Display and both `en_us` and `de_de` strings.

use super::{WIFI_BLUE, is_dark, markup_label, palette, sidebar_style_icon_path};
use crate::daemon;
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::apply_css;
use crate::UIKit::prelude::*;
use gtk::prelude::*;
use std::rc::Rc;

const HEADER_ICON_PX: i32 = 32;

/// Blue `sun.max.fill` icon, same artwork as the sidebar row icon.
fn displays_icon_path() -> Option<String> {
  sidebar_style_icon_path("sun.max.fill", "displays", WIFI_BLUE)
}

/// Standard refresh rates offered up to the monitor max.
const STANDARD_RATES: &[u32] = &[10, 30, 60, 120, 240];
/// Hard cap for offered refresh rates in Hz.
const MAX_RATE: u32 = 1000;

/// Refresh rate options for an output: standard rates up to the monitor
/// max plus every reported rate, capped, sorted and deduplicated.
/// A 120 Hz monitor offers 10/30/60/120, a 240 Hz one adds 240 plus
/// whatever else it reports.
pub(crate) fn refresh_rates(modes: &[daemon::DisplayMode]) -> Vec<u32> {
  let mut reported: Vec<u32> = modes
    .iter()
    .map(|mode| mode.refresh)
    .filter(|rate| (1..=MAX_RATE).contains(rate))
    .collect();
  reported.sort_unstable();
  reported.dedup();
  let max = reported.iter().copied().max().unwrap_or(60);
  let mut options: Vec<u32> = STANDARD_RATES
    .iter()
    .copied()
    .filter(|rate| *rate <= max)
    .chain(reported)
    .collect();
  options.sort_unstable();
  options.dedup();
  options
}

/// "1920 × 1080 @ 60 Hz" mode text for an info row.
pub(crate) fn mode_text(mode: &daemon::DisplayMode) -> String {
  format!("{} × {} @ {} Hz", mode.width, mode.height, mode.refresh)
}

/// Output info value: placeholder when missing, name plus mode otherwise.
pub(crate) fn output_value(output_name: &str, mode_label: &str) -> String {
  if output_name.is_empty() {
    lang::t("displays.no_output")
  } else if mode_label.is_empty() {
    output_name.to_string()
  } else {
    format!("{} — {}", output_name, mode_label)
  }
}

/// Info row: label on the left, dynamic value on the right.
fn info_value(label_key: &str, value: &str, pal_fg: &str, pal_secondary: &str, last: bool) -> gtk::Box {
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

  let detail = markup_label(value, 13, "normal", pal_secondary);
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

/// The Displays detail page (directly on the screen).
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(is_dark());
  let fg: &'static str = pal.fg;
  let secondary: &'static str = pal.secondary;

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  // Header row: blue display icon, title + subtitle.
  let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  header.set_hexpand(true);

  if let Some(icon_path) = displays_icon_path() {
    let icon = gtk::Image::from_file(&icon_path);
    icon.set_pixel_size(HEADER_ICON_PX);
    icon.set_valign(gtk::Align::Start);
    header.append(&icon);
  }

  let titles = gtk::Box::new(gtk::Orientation::Vertical, 2);
  titles.set_hexpand(true);
  titles.set_halign(gtk::Align::Fill);
  let title = markup_label(&lang::t("displays.title"), 17, "bold", fg);
  title.set_halign(gtk::Align::Start);
  title.set_xalign(0.0);
  titles.append(&title);
  let subtitle = markup_label(
    &lang::t("displays.header.subtitle"),
    13,
    "normal",
    secondary,
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

  let state = daemon::display_get().unwrap_or_default();
  let primary = state.outputs.first().cloned();

  let rows = gtk::Box::new(gtk::Orientation::Vertical, 0);
  rows.set_hexpand(true);

  // Output info: name plus current mode.
  let (output_name, mode_label) = match &primary {
    Some(output) => (
      output.name.clone(),
      output.current.as_ref().map(mode_text).unwrap_or_default(),
    ),
    None => (String::new(), String::new()),
  };
  rows.append(&info_value(
    "displays.output",
    &output_value(&output_name, &mode_label),
    fg,
    secondary,
    false,
  ));

  // Brightness slider: dims the whole desktop live.
  let brightness_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  brightness_row.set_hexpand(true);
  brightness_row.set_margin_top(5);
  brightness_row.set_margin_bottom(5);
  let brightness_name = markup_label(&lang::t("displays.brightness"), 13, "normal", fg);
  brightness_name.set_halign(gtk::Align::Start);
  brightness_name.set_xalign(0.0);
  brightness_name.set_hexpand(true);
  brightness_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
  brightness_row.append(&brightness_name);
  let slider = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
  slider.set_value(state.brightness as f64);
  slider.set_digits(0);
  slider.set_size_request(200, -1);
  slider.set_halign(gtk::Align::End);
  slider.set_valign(gtk::Align::Center);
  let brightness_known = Rc::new(std::cell::Cell::new(state.brightness as f64));
  let brightness_known_cb = brightness_known.clone();
  slider.connect_value_changed(move |slider| {
    let value = slider.value();
    match daemon::display_set(None, None, None, None, Some(value), None) {
      Ok(applied) => {
        println!("Displays brightness: {}", applied.brightness);
        brightness_known_cb.set(applied.brightness as f64);
      }
      Err(e) => {
        println!("Displays brightness failed: {}", e);
        slider.set_value(brightness_known_cb.get());
      }
    }
  });
  brightness_row.append(&slider);
  crate::UIKit::apply_css(
    &brightness_row,
    "box { border-bottom: 1px solid rgba(128,128,128,0.25); }",
  );
  rows.append(&brightness_row);

  // Night light toggle: warm overlay.
  let night_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  night_row.set_hexpand(true);
  night_row.set_margin_top(5);
  night_row.set_margin_bottom(5);
  let night_name = markup_label(&lang::t("displays.night_light"), 13, "normal", fg);
  night_name.set_halign(gtk::Align::Start);
  night_name.set_xalign(0.0);
  night_name.set_hexpand(true);
  night_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
  night_row.append(&night_name);
  let toggle = Toggle::new("").value(state.night_light).width(52.0).on_change(
    move |on| match daemon::display_set(None, None, None, None, None, Some(on)) {
      Ok(applied) => println!("Displays night light: {}", applied.night_light),
      Err(e) => println!("Displays night light failed: {}", e),
    },
  );
  let toggle_gtk = toggle.to_gtk();
  toggle_gtk.set_halign(gtk::Align::End);
  toggle_gtk.set_valign(gtk::Align::Center);
  toggle_gtk.set_vexpand(false);
  night_row.append(&toggle_gtk);
  crate::UIKit::apply_css(
    &night_row,
    "box { border-bottom: 1px solid rgba(128,128,128,0.25); }",
  );
  rows.append(&night_row);

  // Refresh rate dropdown from the monitor's reported modes.
  let refresh_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  refresh_row.set_hexpand(true);
  refresh_row.set_margin_top(5);
  refresh_row.set_margin_bottom(5);
  let refresh_name = markup_label(&lang::t("displays.refresh_rate"), 13, "normal", fg);
  refresh_name.set_halign(gtk::Align::Start);
  refresh_name.set_xalign(0.0);
  refresh_name.set_hexpand(true);
  refresh_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
  refresh_row.append(&refresh_name);
  let modes = primary.as_ref().map(|o| o.modes.clone()).unwrap_or_default();
  let options = refresh_rates(&modes);
  let current_refresh = primary.as_ref().and_then(|o| o.current.as_ref()).map(|m| m.refresh);
  let labels: Vec<String> = options.iter().map(|rate| format!("{} Hz", rate)).collect();
  let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
  let refresh_drop = gtk::DropDown::from_strings(&refs);
  refresh_drop.set_halign(gtk::Align::End);
  let selected = current_refresh
    .and_then(|rate| options.iter().position(|r| *r == rate))
    .unwrap_or(0) as u32;
  refresh_drop.set_selected(selected);
  let selected_known = Rc::new(std::cell::Cell::new(selected));
  let selected_known_cb = selected_known.clone();
  let output_name_cb = output_name.clone();
  let current_cb = primary.and_then(|o| o.current.clone());
  refresh_drop.connect_selected_notify(move |drop| {
    let index = drop.selected() as usize;
    let rate = match options.get(index).copied() {
      Some(rate) => rate,
      None => return,
    };
    let (width, height) = match &current_cb {
      Some(mode) => (Some(mode.width), Some(mode.height)),
      None => (None, None),
    };
    let output = if output_name_cb.is_empty() {
      None
    } else {
      Some(output_name_cb.as_str())
    };
    match daemon::display_set(output, width, height, Some(rate), None, None) {
      Ok(_) => {
        println!("Displays refresh rate: {} Hz", rate);
        selected_known_cb.set(index as u32);
      }
      Err(e) => {
        println!("Displays refresh rate failed: {}", e);
        drop.set_selected(selected_known_cb.get());
      }
    }
  });
  refresh_row.append(&refresh_drop);
  rows.append(&refresh_row);

  detail.append(&rows);

  detail.upcast()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::daemon::DisplayMode;

  fn mode(refresh: u32) -> DisplayMode {
    DisplayMode { width: 1920, height: 1080, refresh }
  }

  #[test]
  fn rates_cover_monitor_max_plus_reported() {
    // 120 Hz monitor reports 60/120: standard set up to the max.
    assert_eq!(refresh_rates(&[mode(60), mode(120)]), vec![10, 30, 60, 120]);
    // 240 Hz monitor: standard set plus whatever it reports.
    assert_eq!(
      refresh_rates(&[mode(60), mode(144), mode(240)]),
      vec![10, 30, 60, 120, 144, 240]
    );
    // Reports cap at 1000 Hz, duplicates collapse.
    assert_eq!(refresh_rates(&[mode(1200), mode(60), mode(60)]), vec![10, 30, 60]);
    // No modes: sensible 60 Hz default offer.
    assert_eq!(refresh_rates(&[]), vec![10, 30, 60]);
  }

  #[test]
  fn mode_text_formats() {
    assert_eq!(mode_text(&mode(120)), "1920 × 1080 @ 120 Hz");
  }

  #[test]
  fn output_value_covers_states() {
    assert_eq!(output_value("", ""), lang::t("displays.no_output"));
    assert_eq!(output_value("HDMI-1", ""), "HDMI-1");
    assert_eq!(output_value("HDMI-1", "1920 × 1080 @ 60 Hz"), "HDMI-1 — 1920 × 1080 @ 60 Hz");
  }
}
