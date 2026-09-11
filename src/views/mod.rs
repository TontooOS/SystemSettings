//! Settings views for SystemSettings.
//!
//! One module per settings page (`wifi`, `bluetooth`, `network`,
//! `battery`, `general`, `accessibility`, `appearance`, `desktop_dock`,
//! `displays`, `menu_bar`, `tinti_ai`, `spotlight`, `wallpaper`,
//! `notifications`, `sound`, `focus`, `screen_time`, `lock_screen`,
//! `privacy`, `touch_id`, `users`, `internet_accounts`, `octo_cloud`,
//! `keyboard`, `mouse`, `printers`, `app_settings`, `developer`,
//! `customize`);
//! `root` assembles the sidebar and swaps the detail page on selection.
//! Shared helpers (palette, labels, CoreIcon rendering) live here.

pub mod accessibility;
pub mod appearance;
pub mod app_settings;
pub mod battery;
pub mod bluetooth;
pub mod customize;
pub mod desktop_dock;
pub mod developer;
pub mod displays;
pub mod focus;
pub mod general;
pub mod internet_accounts;
pub mod keyboard;
pub mod lock_screen;
pub mod menu_bar;
pub mod mouse;
pub mod network;
pub mod notifications;
pub mod octo_cloud;
pub mod printers;
pub mod privacy;
pub mod root;
pub mod screen_time;
pub mod tinti_ai;
pub mod sound;
pub mod spotlight;
pub mod touch_id;
pub mod users;
pub mod wallpaper;
pub mod wifi;

use crate::CoreIcon;
use gtk::prelude::*;

pub(crate) const SF_PRO: &str = "SF Pro Display";
pub(crate) const WIFI_BLUE: (u8, u8, u8) = (0, 122, 255);
pub(crate) const BATTERY_GREEN: (u8, u8, u8) = (52, 199, 89);
pub(crate) const BADGE_GRAY: (u8, u8, u8) = (142, 142, 147);

pub(crate) struct Palette {
  pub fg: &'static str,
  pub secondary: &'static str,
  pub card: &'static str,
}

pub(crate) fn palette(dark: bool) -> Palette {
  if dark {
    Palette {
      fg: "#F5F5F7",
      secondary: "#A1A1A6",
      card: "#2C2C2E",
    }
  } else {
    Palette {
      fg: "#1E1E1E",
      secondary: "#6E6E73",
      card: "#F5F5F7",
    }
  }
}

pub(crate) fn is_dark() -> bool {
  crate::UIKit::app::current_color_scheme()
    .unwrap_or_else(crate::UIKit::app::ColorScheme::detect_system)
    == crate::UIKit::app::ColorScheme::Dark
}

pub(crate) fn markup_label(text: &str, size: u32, weight: &str, color: &str) -> gtk::Label {
  let label = gtk::Label::new(None);
  label.set_use_markup(true);
  label.set_markup(&format!(
    "<span font_desc=\"{} {} {}\" foreground=\"{}\">{}</span>",
    SF_PRO,
    weight,
    size,
    color,
    glib::markup_escape_text(text),
  ));
  label
}

fn point_to_coreicon() {
  let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../TontooLibs/CoreIcon/assets/icons");
  if assets.exists() {
    if let Some(dir) = assets.to_str() {
      unsafe {
        CoreIcon::generator::ASSETS_DIR = Box::leak(dir.to_string().into_boxed_str());
      }
    }
  }
}

/// Render an SF Symbol with CoreIcon in sidebar style (solid fill, white
/// glyph, the exact parameters the sidebar row icons use). Returns the
/// cached PNG path. `color` is the tile fill, `tag` scopes the cache file.
pub(crate) fn sidebar_style_icon_path(
  symbol_name: &str,
  tag: &str,
  color: (u8, u8, u8),
) -> Option<String> {
  let symbol = CoreIcon::SFSymbol::from_name(symbol_name)?;
  point_to_coreicon();

  let path = std::env::temp_dir().join(format!(
    "settings_{}_{:02x}{:02x}{:02x}.png",
    tag, color.0, color.1, color.2
  ));
  if path.exists() {
    return Some(path.to_str()?.to_string());
  }

  let fill = CoreIcon::Color::new(
    color.0 as f32 / 255.0,
    color.1 as f32 / 255.0,
    color.2 as f32 / 255.0,
    1.0,
  );
  let canvas = CoreIcon::generator::IconCanvas::new()
    .background(CoreIcon::generator::Background::color(fill))
    .corner_radius(220.0)
    .layer(
      CoreIcon::generator::Layer::new(CoreIcon::generator::LayerContent::icon(symbol))
        .position(120.0, 120.0)
        .size(784.0, 784.0)
        .tint(CoreIcon::Color::new(1.0, 1.0, 1.0, 1.0)),
    );
  canvas.save(&path).ok()?;
  Some(path.to_str()?.to_string())
}
