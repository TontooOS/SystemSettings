//! Appearance settings page for SystemSettings (display only).
//!
//! Three theme cards (Auto, Light, Dark) using the bundled PNG assets,
//! a color accent row and an icon & widget style row. Nothing is
//! changeable; the page is display-only. All text uses SF Pro Display
//! and both `en_us` and `de_de` strings.

use super::{is_dark, markup_label, palette};
use crate::lang;
use crate::UIKit::apply_css;
use gtk::prelude::*;

/// Bundled Appearance artwork for the sidebar icon
/// (`Resources/mf4of5ol1b5a1inx0512nn8mq6wd.png`).
pub(crate) fn appearance_png() -> String {
  format!(
    "{}/Resources/mf4of5ol1b5a1inx0512nn8mq6wd.png",
    env!("CARGO_MANIFEST_DIR")
  )
}

/// One of the three theme thumbnails (Auto / Light / Dark).
struct ThemeCard {
  key: &'static str,
  png: &'static str,
}

const THEMES: &[ThemeCard] = &[
  ThemeCard {
    key: "appearance.theme.auto",
    png: "auto.png",
  },
  ThemeCard {
    key: "appearance.theme.light",
    png: "light.png",
  },
  ThemeCard {
    key: "appearance.theme.dark",
    png: "dark.png",
  },
];

/// Accent color dots displayed in the color row (display only).
const ACCENT_COLORS: &[(&str, &str)] = &[
  ("#AF52DE", "purple"),
  ("#FF2D55", "pink"),
  ("#FF9500", "orange"),
  ("#FFCC00", "yellow"),
  ("#34C759", "green"),
  ("#00C7BE", "teal"),
  ("#30B0C7", "cyan"),
  ("#007AFF", "blue"),
  ("#5856D6", "indigo"),
  ("#BF5AF2", "purple2"),
  ("#8E8E93", "gray"),
];

/// Icon & widget style options (display only).
const STYLE_OPTIONS: &[(&str, &str)] = &[
  ("appearance.style.default", "Default"),
  ("appearance.style.dark", "Dark"),
  ("appearance.style.clear", "Clear"),
  ("appearance.style.tinted", "Tinted"),
];

/// Rounded card container in the page palette color (same style as the
/// other pages).
fn card(pal_card: &str) -> gtk::Box {
  let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
  card.set_hexpand(true);
  apply_css(
    &card,
    &format!(
      "box {{ background-color: {}; border-radius: 12px; padding: 12px 16px; }}",
      pal_card
    ),
  );
  card
}

/// Small section header.
fn section_label(title: &str, secondary: &str) -> gtk::Widget {
  let section = markup_label(title, 12, "normal", secondary);
  section.set_halign(gtk::Align::Start);
  section.set_margin_bottom(2);
  section.upcast()
}

/// One colored circle (accent color dot).
fn color_dot(hex: &str, radius: i32) -> gtk::Box {
  let dot = gtk::Box::new(gtk::Orientation::Vertical, 0);
  dot.set_size_request(radius * 2, radius * 2);
  apply_css(
    &dot,
    &format!(
      "box {{ background-color: {}; border-radius: {}px; }}",
      hex, radius
    ),
  );
  dot
}

/// Pre-scaled theme thumbnail at the display size (matches the other
/// fixed-size artwork pattern).
fn theme_thumbnail(name: &str, px: i32) -> Option<gtk::Picture> {
  let path = format!("{}/Resources/{}", env!("CARGO_MANIFEST_DIR"), name);
  if !std::path::Path::new(&path).is_file() {
    return None;
  }
  let file = super::wallpaper::cached_thumb_fit(std::path::Path::new(&path), px, px)
    .unwrap_or_else(|| std::path::PathBuf::from(path));
  let picture = gtk::Picture::for_filename(file);
  picture.set_content_fit(gtk::ContentFit::Cover);
  picture.set_hexpand(false);
  picture.set_vexpand(false);
  picture.set_can_shrink(true);
  picture.set_size_request(px, px);
  Some(picture)
}

/// The Appearance detail page (display only, directly on the screen).
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(is_dark());

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 8);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  // Header: just the "Appearance" title.
  let title = markup_label(&lang::t("appearance.title"), 17, "bold", pal.fg);
  title.set_halign(gtk::Align::Start);
  title.set_margin_bottom(4);
  detail.append(&title);

  // Three theme thumbnails: Auto, Light, Dark.
  let theme_card = card(pal.card);
  let theme_row = gtk::Box::new(gtk::Orientation::Horizontal, 16);
  theme_row.set_hexpand(true);
  theme_row.set_valign(gtk::Align::Center);
  theme_row.set_halign(gtk::Align::Center);
  for theme in THEMES {
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
    vbox.set_halign(gtk::Align::Center);
    if let Some(picture) = theme_thumbnail(theme.png, 80) {
      vbox.append(&picture);
    }
    let label = markup_label(&lang::t(theme.key), 12, "normal", pal.fg);
    label.set_halign(gtk::Align::Center);
    label.set_xalign(0.5);
    vbox.append(&label);
    theme_row.append(&vbox);
  }
  theme_card.append(&theme_row);
  detail.append(&theme_card);

  // Theme section: accent color dots.
  detail.append(&section_label(&lang::t("appearance.theme.section"), pal.secondary));
  let color_card = card(pal.card);
  let color_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  color_row.set_hexpand(true);
  color_row.set_valign(gtk::Align::Center);
  for (hex, _name) in ACCENT_COLORS {
    color_row.append(&color_dot(hex, 12));
  }
  color_card.append(&color_row);
  detail.append(&color_card);

  // Icon & widget style row: display only.
  detail.append(&section_label(
    &lang::t("appearance.style.section"),
    pal.secondary,
  ));
  let style_card = card(pal.card);
  let style_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  style_row.set_hexpand(true);
  style_row.set_valign(gtk::Align::Center);
  style_row.set_margin_top(2);
  style_row.set_margin_bottom(2);
  for (key, _fallback) in STYLE_OPTIONS {
    let pill = markup_label(&lang::t(key), 12, "normal", pal.fg);
    pill.set_halign(gtk::Align::Center);
    pill.set_xalign(0.5);
    apply_css(
      &pill,
      &format!(
        "label {{ background-color: rgba(128,128,128,0.20); border-radius: 8px; padding: 6px 12px; }}",
      ),
    );
    style_row.append(&pill);
  }
  style_card.append(&style_row);
  detail.append(&style_card);

  detail.upcast()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn theme_icons_exist() {
    for theme in THEMES {
      let path = format!("{}/Resources/{}", env!("CARGO_MANIFEST_DIR"), theme.png);
      assert!(
        std::path::Path::new(&path).is_file(),
        "missing theme icon: {}",
        theme.png
      );
    }
  }

  #[test]
  fn accent_colors_are_valid_hex() {
    for (hex, _name) in ACCENT_COLORS {
      assert!(hex.starts_with('#'));
      assert_eq!(hex.len(), 7);
    }
  }
}
