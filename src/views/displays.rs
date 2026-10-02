//! Displays settings page for SystemSettings.
//!
//! Output info, a live brightness slider (dims the whole desktop through
//! the compositor), the night light switch and a refresh rate dropdown
//! built from the monitor's reported modes. All values come from the
//! settings daemon (`display_get`) with defaults when it is unreachable;
//! every change applies live via `display_set`.

use crate::daemon;
use crate::lang;
use crate::views::{
  header_subtitle, Nav, PageView, Skin, BLOCK_GAP, DISPLAYS,
};
use crate::TontooUI::elements::{
  Align, Form, FormRow, FormSection, HStack, Slider, TextAlignment, VStack, View,
};

/// Standard refresh rates offered up to the monitor max.
const STANDARD_RATES: &[u32] = &[10, 30, 60, 120, 240];
/// Hard cap for offered refresh rates in Hz.
const MAX_RATE: u32 = 1000;

/// Refresh rate options for an output: standard rates up to the monitor
/// max plus every reported rate, capped, sorted and deduplicated.
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

/// "1920 x 1080 @ 60 Hz" mode text for an info row.
pub(crate) fn mode_text(mode: &daemon::DisplayMode) -> String {
  format!("{} x {} @ {} Hz", mode.width, mode.height, mode.refresh)
}

/// Output info value: placeholder when missing, name plus mode otherwise.
pub(crate) fn output_value(output_name: &str, mode_label: &str) -> String {
  if output_name.is_empty() {
    lang::t("displays.no_output")
  } else if mode_label.is_empty() {
    output_name.to_string()
  } else {
    format!("{} - {}", output_name, mode_label)
  }
}

/// Build the brightness slider row as a labelled control block. The
/// slider is a native TontooUI element that dims the desktop live.
fn brightness_row(value: u32) -> impl View + 'static {
  HStack::new()
    .spacing(14.0)
    .align(Align::Leading)
    .child(
      crate::TontooUI::elements::BasicText::new(lang::t("displays.brightness"))
        .size(13.0)
        .width(120.0)
        .alignment(TextAlignment::Leading),
    )
    .child(Slider::new(value.min(100) as f64, 0.0, 100.0).step(1.0).on_change(
      |next| {
        match daemon::display_set(None, None, None, None, Some(next), None) {
          Ok(applied) => println!("Displays brightness: {}", applied.brightness),
          Err(err) => println!("Displays brightness failed: {err}"),
        }
      },
    ))
}

/// Build the Displays detail page.
pub(crate) fn build(_skin: &Skin, _nav: &Nav) -> PageView {
  let state = daemon::display_get().unwrap_or_default();
  let primary = state.outputs.first().cloned();
  let modes = primary.as_ref().map(|out| out.modes.clone()).unwrap_or_default();
  let options = refresh_rates(&modes);
  let labels: Vec<String> = options.iter().map(|rate| format!("{} Hz", rate)).collect();
  let selected = primary
    .as_ref()
    .and_then(|out| out.current.as_ref())
    .and_then(|mode| options.iter().position(|rate| *rate == mode.refresh))
    .unwrap_or(0);
  let (output_name, mode_label) = match &primary {
    Some(out) => (
      out.name.clone(),
      out.current.as_ref().map(mode_text).unwrap_or_default(),
    ),
    None => (String::new(), String::new()),
  };

  let mut section = FormSection::new()
    .row(FormRow::text(
      lang::t("displays.output"),
      output_value(&output_name, &mode_label),
    ))
    .row(FormRow::toggle(lang::t("displays.night_light"), state.night_light).on_toggle(
      move |on| match daemon::display_set(None, None, None, None, None, Some(on)) {
        Ok(applied) => println!("Displays night light: {}", applied.night_light),
        Err(err) => println!("Displays night light failed: {err}"),
      },
    ));
  if !labels.is_empty() {
    let output_for_pick = output_name.clone();
    let current_for_pick = primary.as_ref().and_then(|out| out.current.clone());
    section = section.row(
      FormRow::picker(lang::t("displays.refresh_rate"), labels, selected).on_pick(move |index| {
        let Some(rate) = options.get(index).copied() else {
          return;
        };
        let (width, height) = match &current_for_pick {
          Some(mode) => (Some(mode.width), Some(mode.height)),
          None => (None, None),
        };
        let output = if output_for_pick.is_empty() {
          None
        } else {
          Some(output_for_pick.as_str())
        };
        if let Err(err) = daemon::display_set(output, width, height, Some(rate), None, None) {
          println!("Displays refresh rate failed: {err}");
        }
      }),
    );
  }

  let body = VStack::new()
    .spacing(BLOCK_GAP)
    .align(Align::Leading)
    .child(Form::new().section(section))
    .child(brightness_row(state.brightness));

  crate::views::page_shell(
    crate::views::page_header(crate::views::header_symbol(DISPLAYS), &header_subtitle(DISPLAYS)),
    body,
  )
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
    assert_eq!(mode_text(&mode(120)), "1920 x 1080 @ 120 Hz");
  }

  #[test]
  fn output_value_covers_states() {
    assert_eq!(output_value("", ""), lang::t("displays.no_output"));
    assert_eq!(output_value("HDMI-1", ""), "HDMI-1");
    assert_eq!(output_value("HDMI-1", "1920 x 1080 @ 60 Hz"), "HDMI-1 - 1920 x 1080 @ 60 Hz");
  }
}
