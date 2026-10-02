//! Appearance settings page for SystemSettings (display only).
//!
//! Three theme thumbnails (Auto, Light, Dark) from the bundled PNG
//! assets, an accent color row, an icon and widget style row rendered
//! through the CoreIcon `AppIcon` pipeline (Default / Dark / Tinted Light
//! / Tinted Dark) and a color row. Nothing is changeable: the page shows
//! what TontooUI is currently themed with.

use std::path::PathBuf;

use crate::lang;
use crate::views::{
  caption, header_subtitle, Nav, PageView, Skin, APPEARANCE, BLOCK_GAP,
};
use crate::CoreIcon;
use crate::TontooUI::elements::{
  Align, BasicText, Circle, FileImage, HStack, ImageFit, SFSymbolImage, Spacer, TextAlignment,
  VStack,
};

/// One of the three theme thumbnails.
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

/// Icon and widget style options: label plus CoreIcon `AppIcon` variant.
struct StyleOption {
  key: &'static str,
  style: AppIconStyle,
}

const STYLE_OPTIONS: &[StyleOption] = &[
  StyleOption { key: "appearance.style.default", style: AppIconStyle::Default },
  StyleOption { key: "appearance.style.dark", style: AppIconStyle::Dark },
  StyleOption { key: "appearance.style.tinted_light", style: AppIconStyle::TintedLight },
  StyleOption { key: "appearance.style.tinted_dark", style: AppIconStyle::TintedDark },
];

/// Accent color dots in dropdown order, matching the TontooUI `Accent`
/// variants the theme daemon can report.
const ACCENT_COLORS: &[&str] = &[
  "#AF52DE", "#FF2D55", "#FF9500", "#FFCC00", "#34C759", "#00C7BE", "#30B0C7", "#007AFF", "#BF5AF2",
  "#5856D6", "#8E8E93",
];

/// Icon and widget style variant for the previews. Maps 1:1 onto the
/// CoreIcon `AppIcon` pipeline (full recolor plus Liquid Glass finish,
/// not just a colored border).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppIconStyle {
  /// Original colors, light background (`AppIcon::from_file` default).
  Default,
  /// Dark background, artwork colors kept (`.dark()`).
  Dark,
  /// Green-tinted artwork on the original background (`.tint(green)`).
  TintedLight,
  /// Green-tinted artwork on the dark background (`.dark().tint(green)`).
  TintedDark,
}

impl AppIconStyle {
  fn tag(self) -> &'static str {
    match self {
      AppIconStyle::Default => "default",
      AppIconStyle::Dark => "dark",
      AppIconStyle::TintedLight => "tinted_light",
      AppIconStyle::TintedDark => "tinted_dark",
    }
  }
}

/// Point CoreIcon at the development asset folder when it is there, so the
/// SF Symbols resolve from a checkout instead of the installed library.
fn point_to_coreicon() {
  let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../TontooLibs/CoreIcon/assets/icons");
  if !assets.exists() {
    return;
  }
  if let Some(dir) = assets.to_str() {
    unsafe {
      CoreIcon::generator::ASSETS_DIR = Box::leak(dir.to_string().into_boxed_str());
    }
  }
}

/// Render the app icon (`Resources/app_icon.png`) through the CoreIcon
/// `AppIcon` pipeline with the full Liquid Glass finish. Used for the
/// icon style previews. Returns the cached PNG path.
pub(crate) fn app_icon_style_path(style: AppIconStyle) -> Option<String> {
  let icon_path = format!("{}/Resources/app_icon.png", env!("CARGO_MANIFEST_DIR"));
  if !std::path::Path::new(&icon_path).is_file() {
    return None;
  }
  point_to_coreicon();
  let path = std::env::temp_dir().join(format!("settings_appicon_{}.png", style.tag()));
  if path.exists() {
    return path.to_str().map(str::to_string);
  }
  let green = CoreIcon::Color::from_hex("#34C759").unwrap_or(CoreIcon::Color::new(
    52.0 / 255.0,
    199.0 / 255.0,
    89.0 / 255.0,
    1.0,
  ));
  let app_icon = CoreIcon::generator::AppIcon::from_file(&icon_path);
  let app_icon = match style {
    AppIconStyle::Default => app_icon.light(),
    AppIconStyle::Dark => app_icon.dark(),
    AppIconStyle::TintedLight => app_icon.tint(green),
    AppIconStyle::TintedDark => app_icon.dark().tint(green),
  };
  app_icon.save(&path).ok()?;
  path.to_str().map(str::to_string)
}

/// Bundled theme thumbnail path, when the asset exists.
fn theme_thumbnail(name: &str) -> Option<String> {
  let path = format!("{}/Resources/{}", env!("CARGO_MANIFEST_DIR"), name);
  std::path::Path::new(&path).is_file().then_some(path)
}

/// One thumbnail cell: the image plus its label below. Stacks take sized
/// views only, so each arm builds its own concrete element.
fn thumbnail_cell(image: Option<String>, fallback: &str, size: f32, label: &str) -> VStack {
  let mut cell = VStack::new().spacing(8.0).align(Align::Leading);
  match image {
    Some(path) => {
      cell = cell.child(FileImage::new(path, size, size).radius(10.0).fit(ImageFit::Cover));
    }
    None => {
      cell = cell.child(SFSymbolImage::new(fallback).size(size / 2.0));
    }
  }
  cell.child(
    BasicText::new(label)
      .size(12.0)
      .width(90.0)
      .alignment(TextAlignment::Center),
  )
}

/// Row of the three theme thumbnails.
fn theme_row() -> HStack {
  let mut row = HStack::new().spacing(18.0).align(Align::Leading);
  for theme in THEMES {
    row = row.child(thumbnail_cell(
      theme_thumbnail(theme.png),
      "circle.lefthalf.filled",
      96.0,
      &lang::t(theme.key),
    ));
  }
  row.child(Spacer::new().factor(1.0))
}

/// Row of the four icon and widget style previews.
fn style_row() -> HStack {
  let mut row = HStack::new().spacing(18.0).align(Align::Leading);
  for option in STYLE_OPTIONS {
    row = row.child(thumbnail_cell(
      app_icon_style_path(option.style),
      "app.dashed",
      48.0,
      &lang::t(option.key),
    ));
  }
  row.child(Spacer::new().factor(1.0))
}

/// Row of accent color dots, using the TontooUI accent tokens.
fn accent_row(skin: &Skin) -> HStack {
  let active = skin.accent;
  let mut row = HStack::new().spacing(12.0).align(Align::Leading);
  for hex in ACCENT_COLORS {
    let mut dot = Circle::new(24.0).fill(crate::views::parse_color(hex));
    if *hex == crate::views::accent_hex(active) {
      dot = dot.stroke(skin.text).stroke_width(2.0);
    }
    row = row.child(dot);
  }
  row.child(Spacer::new().factor(1.0))
}

/// Build the Appearance detail page.
pub(crate) fn build(skin: &Skin, _nav: &Nav) -> PageView {
  let body = VStack::new()
    .spacing(BLOCK_GAP)
    .align(Align::Leading)
    .child(caption(&lang::t("appearance.theme.section")))
    .child(theme_row())
    .child(caption(&lang::t("appearance.color.section")))
    .child(accent_row(skin))
    .child(caption(&lang::t("appearance.style.section")))
    .child(style_row());

  crate::views::page_shell(
    crate::views::page_header(crate::views::header_symbol(APPEARANCE), &header_subtitle(APPEARANCE)),
    body,
  )
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
    for hex in ACCENT_COLORS {
      assert!(hex.starts_with('#'));
      assert_eq!(hex.len(), 7);
      assert!(crate::views::parse_color(hex).to_rgba8().a > 0);
    }
  }

  #[test]
  fn style_tags_are_unique_cache_keys() {
    let mut tags: Vec<&str> = STYLE_OPTIONS.iter().map(|o| o.style.tag()).collect();
    tags.sort_unstable();
    tags.dedup();
    assert_eq!(tags.len(), STYLE_OPTIONS.len());
  }
}
