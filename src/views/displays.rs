//! Displays settings page for SystemSettings.
//!
//! One card with output info plus live controls: brightness slider
//! (TontooUI, dims the whole desktop in the compositor), night light
//! toggle (warm overlay) and a refresh rate dropdown built from the
//! monitor's reported modes (capped at the monitor max). No title header
//! (like the Wallpaper page, the toolbar shows the title). All values
//! come from the settings daemon (`display_get`) with defaults when it
//! is unreachable; every change applies live via `display_set` and
//! persists there. All text uses SF Pro Display and both `en_us` and
//! `de_de` strings.

use super::{is_dark, markup_label, palette};
use crate::daemon;
use crate::lang;
use crate::TontooUI::{Slider, Toggle};
use crate::UIKit::apply_css;
use crate::UIKit::prelude::*;
use gtk::prelude::*;
use std::rc::Rc;

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

/// Rounded card container in the page palette color.
fn card(pal_card: &str) -> gtk::Box {
  let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
  card.set_hexpand(true);
  crate::UIKit::apply_css(
    &card,
    &format!(
      "box {{ background-color: {}; border-radius: 12px; padding: 16px; }}",
      pal_card
    ),
  );
  card
}

/// The Displays detail page (directly on the screen).
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(is_dark());
  let fg: &'static str = pal.fg;
  let secondary: &'static str = pal.secondary;

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 16);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  let state = daemon::display_get().unwrap_or_default();
  let primary = state.outputs.first().cloned();

  // Single card: output info, brightness, night light, refresh rate.
  let card = card(pal.card);
  let rows = gtk::Box::new(gtk::Orientation::Vertical, 0);
  rows.set_hexpand(true);

  // Output info: name plus current mode.
  let info_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  info_row.set_hexpand(true);
  info_row.set_margin_top(5);
  info_row.set_margin_bottom(5);
  let info_name = markup_label(&lang::t("displays.output"), 13, "normal", fg);
  info_name.set_halign(gtk::Align::Start);
  info_name.set_xalign(0.0);
  info_name.set_hexpand(true);
  info_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
  info_row.append(&info_name);
  let (output_name, mode_label) = match &primary {
    Some(output) => (
      output.name.clone(),
      output.current.as_ref().map(mode_text).unwrap_or_default(),
    ),
    None => (String::new(), String::new()),
  };
  let info_detail = markup_label(&output_value(&output_name, &mode_label), 13, "normal", secondary);
  info_detail.set_halign(gtk::Align::End);
  info_row.append(&info_detail);
  crate::UIKit::apply_css(
    &info_row,
    "box { border-bottom: 1px solid rgba(128,128,128,0.25); }",
  );
  rows.append(&info_row);

  // Brightness slider: dims the whole desktop live (TontooUI).
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
  let slider = Slider::new(0.0, 100.0)
    .value(state.brightness as f32)
    .step(1.0)
    .width(200.0)
    .on_change(move |value| {
      match daemon::display_set(None, None, None, None, Some(value as f64), None) {
        Ok(applied) => println!("Displays brightness: {}", applied.brightness),
        Err(e) => println!("Displays brightness failed: {}", e),
      }
    });
  let slider_gtk = slider.to_gtk();
  slider_gtk.set_halign(gtk::Align::End);
  slider_gtk.set_valign(gtk::Align::Center);
  brightness_row.append(&slider_gtk);
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

  card.append(&rows);
  detail.append(&card);

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
