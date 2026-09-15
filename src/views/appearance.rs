//! Appearance settings page for SystemSettings (display only).
//!
//! Three theme cards (Auto, Light, Dark) using the bundled PNG assets
//! with rounded corners and a blue selection border, a color accent row,
//! an icon & widget style row (Default, Dark, Tinted rendered via
//! CoreIcon from `app_icon.png`, Tinted uses green), and a color
//! picker. Nothing is changeable; the page is display-only. All text
//! uses SF Pro Display and both `en_us` and `de_de` strings.

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

/// Icon & widget style options: label, CoreIcon bg color, whether selected.
struct StyleOption {
  key: &'static str,
  bg: (u8, u8, u8),
  selected: bool,
}

const STYLE_OPTIONS: &[StyleOption] = &[
  StyleOption { key: "appearance.style.default", bg: (0, 122, 255), selected: true },
  StyleOption { key: "appearance.style.dark", bg: (28, 28, 30), selected: false },
  StyleOption { key: "appearance.style.tinted", bg: (52, 199, 89), selected: false },
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
  ("#8E8E93", "gray"),
  ("#BF5AF2", "purple2"),
];

/// Color picker options (display only, shown with "Tinted" selected).
const COLOR_OPTIONS: &[(&str, &str)] = &[
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

/// Blue selection border CSS for theme and style cards.
const SEL_BORDER: &str = "border: 2px solid #007AFF; border-radius: 10px;";
const NO_BORDER: &str = "border: 2px solid transparent; border-radius: 10px;";

/// Rounded card container in the page palette color.
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

/// Pre-scaled theme thumbnail at the display size with rounded corners.
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
  apply_css(&picture, "picture { border-radius: 10px; }");
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

  // Three theme thumbnails: Auto, Light, Dark with selection border.
  let theme_card = card(pal.card);
  let theme_row = gtk::Box::new(gtk::Orientation::Horizontal, 16);
  theme_row.set_hexpand(true);
  theme_row.set_valign(gtk::Align::Center);
  theme_row.set_halign(gtk::Align::Center);
  for (index, theme) in THEMES.iter().enumerate() {
    let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
    wrapper.set_halign(gtk::Align::Center);
    let border_css = if index == 0 { SEL_BORDER } else { NO_BORDER };
    apply_css(&wrapper, border_css);
    wrapper.set_margin_top(2);
    wrapper.set_margin_bottom(2);
    wrapper.set_margin_start(2);
    wrapper.set_margin_end(2);

    let inner = gtk::Box::new(gtk::Orientation::Vertical, 4);
    inner.set_halign(gtk::Align::Center);
    if let Some(picture) = theme_thumbnail(theme.png, 80) {
      inner.append(&picture);
    }
    let label = markup_label(&lang::t(theme.key), 12, "normal", pal.fg);
    label.set_halign(gtk::Align::Center);
    label.set_xalign(0.5);
    inner.append(&label);
    wrapper.append(&inner);
    theme_row.append(&wrapper);
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

  // Icon & widget style row: rendered icons via CoreIcon, one selected.
  detail.append(&section_label(
    &lang::t("appearance.style.section"),
    pal.secondary,
  ));
  let style_card = card(pal.card);
  let style_row = gtk::Box::new(gtk::Orientation::Horizontal, 16);
  style_row.set_hexpand(true);
  style_row.set_valign(gtk::Align::Center);
  style_row.set_halign(gtk::Align::End);
  for option in STYLE_OPTIONS {
    let wrapper = gtk::Box::new(gtk::Orientation::Vertical, 0);
    wrapper.set_halign(gtk::Align::Center);
    let border_css = if option.selected { SEL_BORDER } else { NO_BORDER };
    apply_css(&wrapper, border_css);
    wrapper.set_margin_top(2);
    wrapper.set_margin_bottom(2);
    wrapper.set_margin_start(2);
    wrapper.set_margin_end(2);

    let inner = gtk::Box::new(gtk::Orientation::Vertical, 4);
    inner.set_halign(gtk::Align::Center);
    if let Some(icon_path) = super::app_icon_style_path(option.key, option.bg) {
      let picture = super::wallpaper::cached_thumb_fit(
        std::path::Path::new(&icon_path),
        48,
        48,
      )
      .unwrap_or_else(|| std::path::PathBuf::from(icon_path));
      if let Some(img) = gtk::Picture::for_filename(picture).dynamic_cast::<gtk::Widget>().ok() {
        img.set_size_request(48, 48);
        apply_css(&img, "picture { border-radius: 10px; }");
        inner.append(&img);
      }
    }
    let label = markup_label(&lang::t(option.key), 12, "normal", pal.fg);
    label.set_halign(gtk::Align::Center);
    label.set_xalign(0.5);
    inner.append(&label);
    wrapper.append(&inner);
    style_row.append(&wrapper);
  }
  style_card.append(&style_row);
  detail.append(&style_card);

  // Color picker section (display only).
  detail.append(&section_label(
    &lang::t("appearance.color.section"),
    pal.secondary,
  ));
  let picker_card = card(pal.card);
  let color_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  color_row.set_hexpand(true);
  color_row.set_valign(gtk::Align::Center);
  for (hex, _name) in COLOR_OPTIONS {
    color_row.append(&color_dot(hex, 12));
  }
  picker_card.append(&color_row);
  detail.append(&picker_card);

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

  #[test]
  fn exactly_one_style_selected() {
    let count = STYLE_OPTIONS.iter().filter(|s| s.selected).count();
    assert_eq!(count, 1, "exactly one style should be selected");
  }
}
