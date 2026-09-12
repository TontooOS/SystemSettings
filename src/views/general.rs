//! General settings page for SystemSettings.
//!
//! Centered header (gear tile, title, subtitle) plus one card per row
//! (About, Software Update, Storage, AirDrop & Handoff, AutoFill &
//! Passwords, Date & Time, Language & Region, Login Items & Extensions,
//! Sharing, Startup Disk, Time Machine, Device Management, Transfer or
//! Reset). Display only: rows have no click actions yet. All text uses
//! SF Pro Display and both `en_us` and `de_de` strings.

use super::{markup_label, palette, sidebar_style_icon_path};
use crate::lang;
use gtk::prelude::*;

const HEADER_ICON_PX: i32 = 48;
const ROW_ICON_PX: i32 = 28;
const GEAR_GRAY: (u8, u8, u8) = (142, 142, 147);
const WIFI_BLUE: (u8, u8, u8) = (0, 122, 255);
const BADGE_GRAY: (u8, u8, u8) = (142, 142, 147);
const INK_BLACK: (u8, u8, u8) = (0, 0, 0);
const TONTOO_ORANGE: (u8, u8, u8) = (255, 107, 43);

/// One General row: lang key, SF Symbol name and tile color.
pub(crate) struct GeneralRow {
  pub key: &'static str,
  pub symbol: &'static str,
  pub color: (u8, u8, u8),
}

/// Rows in mockup order, each rendered as its own card.
pub(crate) const ROWS: &[GeneralRow] = &[
  GeneralRow { key: "general.about", symbol: "questionmark", color: WIFI_BLUE },
  GeneralRow { key: "general.software_update", symbol: "arrow.triangle.2.circlepath", color: WIFI_BLUE },
  GeneralRow { key: "general.storage", symbol: "internaldrive.fill", color: BADGE_GRAY },
  GeneralRow { key: "general.airdrop", symbol: "square.and.arrow.up", color: BADGE_GRAY },
  GeneralRow { key: "general.autofill", symbol: "key.fill", color: BADGE_GRAY },
  GeneralRow { key: "general.datetime", symbol: "clock.fill", color: INK_BLACK },
  GeneralRow { key: "general.language", symbol: "globe", color: WIFI_BLUE },
  GeneralRow { key: "general.login_items", symbol: "square.stack.3d.up.fill", color: TONTOO_ORANGE },
  GeneralRow { key: "general.sharing", symbol: "person.2.circle.fill", color: BADGE_GRAY },
  GeneralRow { key: "general.startup_disk", symbol: "internaldrive.fill", color: BADGE_GRAY },
  GeneralRow { key: "general.time_machine", symbol: "clock.arrow.circlepath", color: BADGE_GRAY },
  GeneralRow { key: "general.device_management", symbol: "checkmark.seal.fill", color: BADGE_GRAY },
  GeneralRow { key: "general.transfer_reset", symbol: "arrow.triangle.swap", color: BADGE_GRAY },
];

/// Row groups (card boundaries) in mockup order: first 3 together,
/// then AirDrop alone, then the next 7 together, then the last 2 alone.
pub(crate) const GROUPS: &[usize] = &[3, 1, 7, 1, 1];

/// Rounded card container in the page palette color.
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

/// One row: tile icon, label and chevron. No click action yet.
/// All but the last row in a card get a divider.
fn nav_row(row: &GeneralRow, pal_fg: &str, pal_secondary: &str, last: bool) -> gtk::Box {
  let inner = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  inner.set_hexpand(true);
  inner.set_valign(gtk::Align::Center);
  inner.set_margin_top(5);
  inner.set_margin_bottom(5);

  let tag = row.key.replace("general.", "general-row-");
  if let Some(icon_path) = sidebar_style_icon_path(row.symbol, &tag, row.color) {
    let icon = gtk::Image::from_file(&icon_path);
    icon.set_pixel_size(ROW_ICON_PX);
    icon.set_valign(gtk::Align::Center);
    inner.append(&icon);
  }

  let name = markup_label(&lang::t(row.key), 13, "normal", pal_fg);
  name.set_halign(gtk::Align::Start);
  name.set_xalign(0.0);
  name.set_hexpand(true);
  name.set_ellipsize(gtk::pango::EllipsizeMode::End);
  inner.append(&name);

  let chevron = markup_label("›", 15, "normal", pal_secondary);
  chevron.set_halign(gtk::Align::End);
  inner.append(&chevron);

  if !last {
    crate::UIKit::apply_css(
      &inner,
      "box { border-bottom: 1px solid rgba(128,128,128,0.25); }",
    );
  }
  inner
}

 /// The General detail page (directly on the screen).
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(super::is_dark());
  let fg: &'static str = pal.fg;
  let secondary: &'static str = pal.secondary;

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 8);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  // Header card: centered gear tile, title and subtitle.
  let header = card(pal.card);
  let header_inner = gtk::Box::new(gtk::Orientation::Vertical, 8);
  header_inner.set_hexpand(true);
  if let Some(icon_path) = sidebar_style_icon_path("gear", "general-header", GEAR_GRAY) {
    let icon = gtk::Image::from_file(&icon_path);
    icon.set_pixel_size(HEADER_ICON_PX);
    icon.set_halign(gtk::Align::Center);
    header_inner.append(&icon);
  }
  let title = markup_label(&lang::t("general.title"), 20, "bold", fg);
  title.set_halign(gtk::Align::Center);
  title.set_xalign(0.5);
  header_inner.append(&title);
  let subtitle = markup_label(&lang::t("general.header.subtitle"), 13, "normal", secondary);
  subtitle.set_halign(gtk::Align::Center);
  subtitle.set_xalign(0.5);
  subtitle.set_wrap(true);
  subtitle.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  subtitle.set_max_width_chars(44);
  subtitle.set_justify(gtk::Justification::Center);
  header_inner.append(&subtitle);
  header.append(&header_inner);
  detail.append(&header);

  // Grouped cards: first 3 together, AirDrop alone, next 7 together,
  // last 2 alone (any future rows land in a final card).
  let mut bounds: Vec<(usize, usize)> = Vec::new();
  let mut start = 0;
  for size in GROUPS.iter().copied().chain(std::iter::once(usize::MAX)) {
    if start >= ROWS.len() {
      break;
    }
    let end = start.saturating_add(size).min(ROWS.len());
    bounds.push((start, end));
    start = end;
  }
  for (start, end) in bounds {
    let card = card(pal.card);
    for (i, row) in ROWS[start..end].iter().enumerate() {
      card.append(&nav_row(row, fg, secondary, i + 1 == end - start));
    }
    detail.append(&card);
  }

  detail.upcast()
}
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rows_match_mockup() {
    assert_eq!(ROWS.len(), 13);
    let keys: Vec<&str> = ROWS.iter().map(|row| row.key).collect();
    assert_eq!(keys[0], "general.about");
    assert_eq!(keys[3], "general.airdrop");
    assert_eq!(keys[12], "general.transfer_reset");
    for row in ROWS {
      assert!(row.key.starts_with("general."));
      assert!(!row.symbol.is_empty());
    }
  }

  #[test]
  fn groups_cover_all_rows() {
    assert_eq!(GROUPS, &[3, 1, 7, 1, 1]);
    assert_eq!(GROUPS.iter().sum::<usize>(), ROWS.len());
  }
}
