//! Wi-Fi settings view for SystemSettings.
//!
//! Left: TontooUI `Sidebar` with the single Wi-Fi/WLAN category. The icon
//! is the CoreIcon SF Symbol `wifi` on a solid blue fill.
//! Right: example Wi-Fi page (title, toggle, example network list).
//! All text uses SF Pro Display and both `en_us` and `de_de` strings.

use crate::lang;
use crate::CoreIcon;
use crate::TontooUI::{List, ListRow, ListSection, ListStyle, Sidebar, SidebarIcon, Toggle};
use crate::UIKit::prelude::*;
use crate::UIKit::widget::{WidgetId, next_widget_id};
use gtk::prelude::*;

const SF_PRO: &str = "SF Pro Display";
const WIFI_BLUE: (u8, u8, u8) = (0, 122, 255);
/// Display size of the header icon (same artwork as the sidebar row icon).
const HEADER_ICON_PX: i32 = 44;

struct Palette {
  fg: &'static str,
  secondary: &'static str,
}

fn palette(dark: bool) -> Palette {
  if dark {
    Palette {
      fg: "#F5F5F7",
      secondary: "#A1A1A6",
    }
  } else {
    Palette {
      fg: "#1E1E1E",
      secondary: "#6E6E73",
    }
  }
}

fn markup_label(text: &str, size: u32, weight: &str, color: &str) -> gtk::Label {
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

/// Render the blue `wifi` SF Symbol header icon with CoreIcon, using the
/// exact same artwork parameters as the sidebar row icon (solid blue fill,
/// white glyph). Returns the cached PNG path.
fn wifi_icon_path() -> Option<String> {
  let symbol = CoreIcon::SFSymbol::from_name("wifi")?;

  let assets = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("../../TontooLibs/CoreIcon/assets/icons");
  if assets.exists() {
    unsafe {
      CoreIcon::generator::ASSETS_DIR =
        Box::leak(assets.to_str()?.to_string().into_boxed_str());
    }
  }

  let path = std::env::temp_dir().join(format!(
    "settings_wifi_{:02x}{:02x}{:02x}.png",
    WIFI_BLUE.0, WIFI_BLUE.1, WIFI_BLUE.2
  ));
  if path.exists() {
    return Some(path.to_str()?.to_string());
  }

  let blue = CoreIcon::Color::new(
    WIFI_BLUE.0 as f32 / 255.0,
    WIFI_BLUE.1 as f32 / 255.0,
    WIFI_BLUE.2 as f32 / 255.0,
    1.0,
  );
  let canvas = CoreIcon::generator::IconCanvas::new()
    .background(CoreIcon::generator::Background::color(blue))
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

/// Root widget: sidebar on the left, Wi-Fi example page on the right.
pub struct SettingsRoot {
  id: WidgetId,
}

impl SettingsRoot {
  pub fn new() -> Self {
    Self {
      id: next_widget_id(),
    }
  }
}

impl Default for SettingsRoot {
  fn default() -> Self {
    Self::new()
  }
}

impl Widget for SettingsRoot {
  fn id(&self) -> WidgetId {
    self.id
  }

  fn to_gtk(&self) -> gtk::Widget {
    let dark = crate::UIKit::app::current_color_scheme()
      .unwrap_or_else(ColorScheme::detect_system)
      == ColorScheme::Dark;
    let pal = palette(dark);

    let outer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    outer.set_hexpand(true);
    outer.set_vexpand(true);

    let sidebar = Sidebar::new()
      .item(
        lang::t("sidebar.wifi"),
        SidebarIcon::sf(
          "wifi",
          Color::from_rgb(WIFI_BLUE.0, WIFI_BLUE.1, WIFI_BLUE.2),
        ),
      )
      .selected(0)
      .search_placeholder(lang::t("sidebar.search"))
      .width(220.0)
      .on_select(|i| println!("Settings selected: {}", i));
    let sidebar_gtk = sidebar.to_gtk();
    outer.append(&sidebar_gtk);

    let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
    detail.set_hexpand(true);
    detail.set_vexpand(true);
    detail.set_margin_top(28);
    detail.set_margin_bottom(24);
    detail.set_margin_start(28);
    detail.set_margin_end(28);

    // Header row: blue Wi-Fi icon, title + subtitle, toggle on the right.
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    header.set_hexpand(true);

    if let Some(icon_path) = wifi_icon_path() {
      let icon = gtk::Image::from_file(&icon_path);
      icon.set_pixel_size(HEADER_ICON_PX);
      icon.set_valign(gtk::Align::Start);
      header.append(&icon);
    }

    let titles = gtk::Box::new(gtk::Orientation::Vertical, 2);
    titles.set_hexpand(true);
    titles.set_halign(gtk::Align::Fill);
    let title = markup_label(&lang::t("wifi.title"), 17, "bold", pal.fg);
    title.set_halign(gtk::Align::Start);
    title.set_xalign(0.0);
    titles.append(&title);
    let subtitle = markup_label(&lang::t("wifi.header.subtitle"), 13, "normal", pal.secondary);
    subtitle.set_halign(gtk::Align::Start);
    subtitle.set_xalign(0.0);
    subtitle.set_wrap(true);
    subtitle.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    subtitle.set_max_width_chars(48);
    titles.append(&subtitle);
    header.append(&titles);

    let toggle = Toggle::new("")
      .value(true)
      .width(52.0)
      .on_change(|on| println!("Wi-Fi toggled: {}", on));
    let toggle_gtk = toggle.to_gtk();
    toggle_gtk.set_halign(gtk::Align::End);
    toggle_gtk.set_valign(gtk::Align::Center);
    header.append(&toggle_gtk);
    detail.append(&header);

    let gap2 = gtk::Box::new(gtk::Orientation::Vertical, 0);
    gap2.set_size_request(-1, 12);
    detail.append(&gap2);

    let list = List::new()
      .section(
        ListSection::new()
          .header(lang::t("wifi.networks.header"))
          .row(
            ListRow::new(lang::t("wifi.row.home")).detail(lang::t("wifi.row.home.detail")),
          )
          .row(ListRow::new(lang::t("wifi.row.lab")).detail(lang::t("wifi.row.lab.detail"))),
      )
      .list_style(ListStyle::InsetGrouped);
    let list_gtk = list.to_gtk();
    list_gtk.set_hexpand(true);
    detail.append(&list_gtk);

    outer.append(&detail);
    outer.upcast()
  }
}
