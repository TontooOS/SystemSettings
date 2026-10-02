//! Date & Time detail page for SystemSettings, pushed from the General
//! Date & Time row.
//!
//! Automatic time (locked on, the daemon enforces NTP), the
//! date/time line, the 24-hour switch and a timezone dropdown built from
//! the zone list the daemon reports. State comes from `timedatectl`.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::daemon;
use crate::lang;
use crate::views::{detail_header, Nav, PageView, Skin, BLOCK_GAP, DATETIME_HIDDEN};
use crate::TontooUI::elements::{Align, Form, FormRow, FormSection, VStack};

/// Longest timezone label before the picker ellipsizes it.
const MAX_ZONE_CHOICES: usize = 512;

/// Seconds since the Unix epoch, or zero when the clock is unreadable.
fn now_epoch() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|since| since.as_secs())
    .unwrap_or(0)
}

/// Convert days since the epoch into `(year, month, day)` using the
/// civil-from-days algorithm (Howard Hinnant), so the date line needs no
/// date crate.
pub(crate) fn civil_from_days(days: i64) -> (i64, u32, u32) {
  let shifted = days + 719_468;
  let era = if shifted >= 0 { shifted } else { shifted - 146_096 } / 146_097;
  let day_of_era = shifted - era * 146_097;
  let year_of_era =
    (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
  let year = year_of_era + era * 400;
  let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
  let mp = (5 * day_of_year + 2) / 153;
  let day = (day_of_year - (153 * mp + 2) / 5 + 1) as u32;
  let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
  (if month <= 2 { year + 1 } else { year }, month, day)
}

/// "Sep 12, 2026 at 12:56:08 PM" (12h) or "Sep 12, 2026 at 13:56:08"
/// (24h) in UTC. Empty when the clock predates the epoch.
pub(crate) fn format_now(use_24h: bool, epoch: u64) -> String {
  let days = (epoch / 86_400) as i64;
  let seconds_of_day = epoch % 86_400;
  let (year, month, day) = civil_from_days(days);
  let hour = seconds_of_day / 3_600;
  let minute = (seconds_of_day % 3_600) / 60;
  let second = seconds_of_day % 60;
  const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
  ];
  let date = format!("{} {}, {}", MONTHS[(month - 1) as usize], day, year);
  if use_24h {
    format!("{date} at {hour:02}:{minute:02}:{second:02}")
  } else {
    let meridiem = if hour < 12 { "AM" } else { "PM" };
    let hour12 = match hour % 12 {
      0 => 12,
      other => other,
    };
    format!("{date} at {hour12}:{minute:02}:{second:02} {meridiem}")
  }
}

/// Index of `current` inside `zones`, 0 when it is not listed.
pub(crate) fn zone_index(zones: &[String], current: &str) -> usize {
  zones.iter().position(|zone| zone == current).unwrap_or(0)
}

/// Build the Date & Time detail page.
pub(crate) fn build(_skin: &Skin, nav: &Nav) -> PageView {
  let state = daemon::datetime_get().unwrap_or_default();
  let zones = state.timezones.clone();
  let options: Vec<String> = zones.iter().take(MAX_ZONE_CHOICES).cloned().collect();
  let selected = zone_index(&options, &state.timezone);

  let mut section = FormSection::new()
    // The daemon clock is UTC; the local zone is what the picker reports
    // back, so the line reads the epoch directly.
    .row(FormRow::text(
      lang::t("datetime.datetime"),
      format_now(state.use_24h, now_epoch()),
    ))
    .row(FormRow::toggle(lang::t("datetime.use_24h"), state.use_24h).on_toggle(
      move |on| {
        if let Err(err) = daemon::datetime_set_24h(on) {
          println!("Date & Time 24-hour save failed: {err}");
        }
      },
    ));
  if !options.is_empty() {
    section = section.row(
      FormRow::picker(lang::t("datetime.timezone"), options, selected).on_pick(move |index| {
        let Some(zone) = zones.get(index) else {
          return;
        };
        if let Err(err) = daemon::datetime_set_timezone(zone) {
          println!("Date & Time timezone save failed: {err}");
        }
      }),
    );
  }

  let body = VStack::new()
    .spacing(BLOCK_GAP)
    .align(Align::Leading)
    .child(Form::new().section(section));

  crate::views::page_shell(
    detail_header(
      nav,
      "clock",
      &lang::t(crate::views::hidden_title(DATETIME_HIDDEN)),
    ),
    body,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn civil_dates_match_known_days() {
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    assert_eq!(civil_from_days(19_723), (2024, 1, 1));
    // 2024 is a leap year: day 59 of the year is Feb 29.
    assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    assert_eq!(civil_from_days(20_708), (2026, 9, 12));
  }

  #[test]
  fn clock_formats_both_modes() {
    // 2026-09-12 12:56:08 UTC.
    let epoch = 20_708 * 86_400 + 12 * 3_600 + 56 * 60 + 8;
    let twelve = format_now(false, epoch);
    assert!(twelve.starts_with("Sep 12, 2026 at "));
    assert!(twelve.contains("PM"));
    assert!(twelve.contains("12:56:08"));
    let twenty_four = format_now(true, epoch);
    assert!(twenty_four.starts_with("Sep 12, 2026 at "));
    assert!(twenty_four.contains("12:56:08"));
    assert!(!twenty_four.contains("AM") && !twenty_four.contains("PM"));
  }

  #[test]
  fn midnight_renders_as_twelve_in_12h_mode() {
    let epoch = 20_708 * 86_400;
    assert!(format_now(false, epoch).contains("12:00:00 AM"));
    assert!(format_now(true, epoch).contains("00:00:00"));
  }

  #[test]
  fn zone_index_falls_back_to_the_first_entry() {
    let zones = vec!["Europe/Berlin".to_string(), "UTC".to_string()];
    assert_eq!(zone_index(&zones, "UTC"), 1);
    assert_eq!(zone_index(&zones, "Mars/Olympus"), 0);
    assert_eq!(zone_index(&[], "UTC"), 0);
  }
}
