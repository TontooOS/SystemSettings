//! Customize settings page for SystemSettings.
//!
//! Wallpaper pack picker (packs from `/System/User/Wallpapers`, display
//! names from each pack's `wallpaper.fish` manifest), accent color choices
//! and a dark mode toggle. Values are read once via `customize_get` and
//! persisted via `customize_set`; when the daemon is unreachable the page
//! falls back to the built-in defaults and keeps working (writes then only
//! log). All text uses SF Pro Display and both `en_us` and `de_de`
//! strings.

use super::{is_dark, markup_label, palette, sidebar_style_icon_path, SF_PRO};
use crate::daemon;
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::prelude::*;
use gtk::prelude::*;
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

const HEADER_ICON_PX: i32 = 32;
const CUSTOMIZE_ORANGE: (u8, u8, u8) = (255, 107, 43);

/// Accent colors offered on the page (daemon `customize` values).
pub(crate) const ACCENTS: &[&str] = &["orange", "blue", "green", "purple"];

/// Directory holding one wallpaper pack per subdirectory.
/// `TONTOO_WALLPAPERS_DIR` overrides the ISO path (dev/test).
pub(crate) fn wallpapers_dir() -> PathBuf {
  if let Ok(dir) = std::env::var("TONTOO_WALLPAPERS_DIR") {
    if !dir.is_empty() {
      return PathBuf::from(dir);
    }
  }
  PathBuf::from("/System/User/Wallpapers")
}

/// Display name from a pack manifest (`name: "..."` line), if any.
fn fish_name(manifest: &Path) -> Option<String> {
  let content = std::fs::read_to_string(manifest).ok()?;
  for line in content.lines() {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("name:") {
      let name = rest.trim().trim_matches('"').trim();
      if !name.is_empty() {
        return Some(name.to_string());
      }
    }
  }
  None
}

/// `(pack id, display name)` pairs sorted by display name.
/// The display name comes from the pack's `wallpaper.fish` manifest and
/// falls back to the directory name.
pub(crate) fn wallpaper_packs_in(dir: &Path) -> Vec<(String, String)> {
  let mut packs = Vec::new();
  let entries = match std::fs::read_dir(dir) {
    Ok(entries) => entries,
    Err(_) => return packs,
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if !path.is_dir() {
      continue;
    }
    let id = match path.file_name().and_then(|n| n.to_str()) {
      Some(id) => id.to_string(),
      None => continue,
    };
    let display = fish_name(&path.join("wallpaper.fish")).unwrap_or_else(|| id.clone());
    packs.push((id, display));
  }
  packs.sort_by(|a, b| a.1.cmp(&b.1));
  packs
}

/// Available packs from the system wallpapers directory.
pub(crate) fn wallpaper_packs() -> Vec<(String, String)> {
  wallpaper_packs_in(&wallpapers_dir())
}

/// Display name for a pack id, falling back to the id itself.
fn display_for(packs: &[(String, String)], id: &str) -> String {
  packs
    .iter()
    .find(|(pack_id, _)| pack_id == id)
    .map(|(_, display)| display.clone())
    .unwrap_or_else(|| id.to_string())
}

/// Effective settings: daemon values when reachable, else the defaults.
pub(crate) fn current_settings() -> daemon::CustomizeSettings {
  daemon::customize_get().unwrap_or_default()
}

/// Lang key for an accent color name (`customize.accent.<name>`).
pub(crate) fn accent_key(accent: &str) -> String {
  format!("customize.accent.{}", accent)
}

/// Info row: localized label on the left, dynamic value on the right.
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

/// Refresh a value label created by `info_value` (same font/size/color).
fn set_value(label: &gtk::Label, value: &str, color: &str) {
  label.set_markup(&format!(
    "<span font_desc=\"{} normal 13\" foreground=\"{}\">{}</span>",
    SF_PRO,
    color,
    glib::markup_escape_text(value),
  ));
}

/// The Customize detail page (directly on the screen).
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

  // Header row: paintbrush icon, title + subtitle.
  let header = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  header.set_hexpand(true);

  if let Some(icon_path) = sidebar_style_icon_path("paintbrush.fill", "customize", CUSTOMIZE_ORANGE) {
    let icon = gtk::Image::from_file(&icon_path);
    icon.set_pixel_size(HEADER_ICON_PX);
    icon.set_valign(gtk::Align::Start);
    header.append(&icon);
  }

  let titles = gtk::Box::new(gtk::Orientation::Vertical, 2);
  titles.set_hexpand(true);
  titles.set_halign(gtk::Align::Fill);
  let title = markup_label(&lang::t("customize.title"), 17, "bold", fg);
  title.set_halign(gtk::Align::Start);
  title.set_xalign(0.0);
  titles.append(&title);
  let subtitle = markup_label(
    &lang::t("customize.header.subtitle"),
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

  let current = current_settings();
  let packs: Rc<Vec<(String, String)>> = Rc::new(wallpaper_packs());

  // Wallpaper section: current pack plus one button per pack.
  let rows = gtk::Box::new(gtk::Orientation::Vertical, 0);
  rows.set_hexpand(true);
  rows.append(&info_value(
    "customize.wallpaper",
    &display_for(&packs, &current.wallpaper),
    fg,
    secondary,
    false,
  ));
  // Reach the value label (second child of the row) for live updates.
  let current_label = rows
    .last_child()
    .and_then(|row| row.first_child()?.next_sibling())
    .and_then(|w| w.downcast::<gtk::Label>().ok());

  let flow = gtk::FlowBox::new();
  flow.set_selection_mode(gtk::SelectionMode::None);
  flow.set_max_children_per_line(4);
  flow.set_row_spacing(8);
  flow.set_column_spacing(8);
  flow.set_margin_top(8);
  flow.set_margin_bottom(8);

  let pack_buttons: Rc<RefCell<Vec<(String, gtk::Button)>>> =
    Rc::new(RefCell::new(Vec::new()));
  for (id, display) in packs.iter() {
    let button = gtk::Button::with_label(display);
    button.set_sensitive(id != &current.wallpaper);
    let id_clicked = id.clone();
    let packs_clicked = packs.clone();
    let buttons_clicked = pack_buttons.clone();
    let label_clicked = current_label.clone();
    button.connect_clicked(move |_| match daemon::customize_set(Some(&id_clicked), None, None) {
      Ok(applied) => {
        println!("Customize wallpaper: {}", applied.wallpaper);
        if let Some(ref label) = label_clicked {
          set_value(label, &display_for(&packs_clicked, &applied.wallpaper), secondary);
        }
        for (pack_id, btn) in buttons_clicked.borrow().iter() {
          btn.set_sensitive(pack_id != &applied.wallpaper);
        }
      }
      Err(e) => println!("Customize wallpaper failed: {}", e),
    });
    pack_buttons.borrow_mut().push((id.clone(), button.clone()));
    flow.insert(&button, -1);
  }
  rows.append(&flow);

  // Accent section: current accent plus one button per color.
  rows.append(&info_value(
    "customize.accent",
    &lang::t(&accent_key(&current.accent)),
    fg,
    secondary,
    false,
  ));
  let accent_label = rows
    .last_child()
    .and_then(|row| row.first_child()?.next_sibling())
    .and_then(|w| w.downcast::<gtk::Label>().ok());

  let accent_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  accent_row.set_hexpand(true);
  accent_row.set_margin_top(8);
  accent_row.set_margin_bottom(8);
  let accent_buttons: Rc<RefCell<Vec<(String, gtk::Button)>>> =
    Rc::new(RefCell::new(Vec::new()));
  for accent in ACCENTS {
    let button = gtk::Button::with_label(&lang::t(&accent_key(accent)));
    button.set_sensitive(*accent != current.accent);
    let accent_clicked = accent.to_string();
    let buttons_clicked = accent_buttons.clone();
    let label_clicked = accent_label.clone();
    button.connect_clicked(move |_| match daemon::customize_set(None, Some(&accent_clicked), None) {
      Ok(applied) => {
        println!("Customize accent: {}", applied.accent);
        if let Some(ref label) = label_clicked {
          set_value(label, &lang::t(&accent_key(&applied.accent)), secondary);
        }
        for (name, btn) in buttons_clicked.borrow().iter() {
          btn.set_sensitive(name != &applied.accent);
        }
      }
      Err(e) => println!("Customize accent failed: {}", e),
    });
    accent_buttons
      .borrow_mut()
      .push((accent.to_string(), button.clone()));
    accent_row.append(&button);
  }
  rows.append(&accent_row);

  // Theme section: dark mode toggle.
  let theme_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  theme_row.set_hexpand(true);
  theme_row.set_margin_top(5);
  theme_row.set_margin_bottom(5);
  let name = markup_label(&lang::t("customize.dark_mode"), 13, "normal", fg);
  name.set_halign(gtk::Align::Start);
  name.set_xalign(0.0);
  name.set_hexpand(true);
  name.set_ellipsize(gtk::pango::EllipsizeMode::End);
  theme_row.append(&name);
  let toggle = Toggle::new("").value(current.theme == "dark").width(52.0).on_change(
    move |on| {
      let theme = if on { "dark" } else { "light" };
      match daemon::customize_set(None, None, Some(theme)) {
        Ok(applied) => println!("Customize theme: {}", applied.theme),
        Err(e) => println!("Customize theme failed: {}", e),
      }
    },
  );
  let toggle_gtk = toggle.to_gtk();
  toggle_gtk.set_halign(gtk::Align::End);
  toggle_gtk.set_valign(gtk::Align::Center);
  toggle_gtk.set_vexpand(false);
  theme_row.append(&toggle_gtk);
  rows.append(&theme_row);

  detail.append(&rows);

  detail.upcast()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn write_pack(dir: &Path, id: &str, manifest: Option<&str>) {
    let pack = dir.join(id);
    std::fs::create_dir_all(&pack).unwrap();
    if let Some(body) = manifest {
      std::fs::write(pack.join("wallpaper.fish"), body).unwrap();
    }
  }

  #[test]
  fn packs_use_manifest_names_and_fall_back_to_dir_names() {
    let dir = std::env::temp_dir().join("systemsettings-customize-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    write_pack(&dir, "THAOELAKE", Some("name: \"Tahoe Lake\"\n"));
    write_pack(&dir, "VENTURA", None);
    std::fs::write(dir.join("stray.txt"), "not a pack").unwrap();

    let packs = wallpaper_packs_in(&dir);
    assert_eq!(
      packs,
      vec![
        ("THAOELAKE".to_string(), "Tahoe Lake".to_string()),
        ("VENTURA".to_string(), "VENTURA".to_string()),
      ]
    );
    assert_eq!(display_for(&packs, "VENTURA"), "VENTURA");
    assert_eq!(display_for(&packs, "MISSING"), "MISSING");
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn missing_dir_yields_no_packs() {
    let packs = wallpaper_packs_in(Path::new("/nonexistent-customize-test"));
    assert!(packs.is_empty());
  }

  #[test]
  fn accent_keys_resolve() {
    assert_eq!(accent_key("orange"), "customize.accent.orange");
  }
}
