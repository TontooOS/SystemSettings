//! Wallpaper settings page for SystemSettings.
//!
//! Current wallpaper card (preview thumbnail, name, fill mode dropdown)
//! plus an available wallpapers card (premade grid; custom uploads come
//! later). Everything is display only except the fill mode dropdown
//! (persisted via `wallpaper_set_fill`); nothing here applies the
//! wallpaper to the desktop. All data comes from the settings daemon
//! (`wallpaper_get`) with empty fallbacks when it is unreachable. All
//! text uses SF Pro Display and both `en_us` and `de_de` strings.

use super::{markup_label, palette};
use crate::daemon;
use crate::lang;
use gtk::prelude::*;

const CURRENT_THUMB_W: i32 = 96;
const CURRENT_THUMB_H: i32 = 64;
const PREMADE_THUMB_W: i32 = 160;
const PREMADE_THUMB_H: i32 = 100;
/// Longest cached thumbnail edge (crisp on HiDPI, tiny on disk).
const THUMB_MAX_PX: i32 = 320;
const THUMB_CORNER_PX: i32 = 12;

/// Fill mode ids in dropdown order (daemon `wallpaper` values).
pub(crate) const FILL_ORDER: &[&str] = &["fill", "fit", "stretch", "center", "tile"];

/// Lang key for a fill mode label (`wallpaper.fill.<id>`).
pub(crate) fn fill_key(fill: &str) -> String {
  format!("wallpaper.fill.{}", fill)
}

/// Index of a fill mode in the dropdown order (0 when unknown).
pub(crate) fn fill_index(fill: &str) -> u32 {
  FILL_ORDER
    .iter()
    .position(|mode| *mode == fill)
    .unwrap_or(0) as u32
}

/// Rounded card container in the page palette color.
fn card(pal_card: &str) -> gtk::Box {  let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
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

/// Small secondary section label.
fn section_label(text_key: &str, pal_secondary: &str) -> gtk::Label {
  let label = markup_label(&lang::t(text_key), 13, "normal", pal_secondary);
  label.set_halign(gtk::Align::Start);
  label.set_xalign(0.0);
  label
}

/// Directory caching scaled-down thumbnails (never the full images).
pub(crate) fn thumb_cache_dir() -> std::path::PathBuf {
  std::env::temp_dir().join("tontoo-wallpaper-thumbs")
}

/// Cache file for a source image, keyed by path, size and mtime, so an
/// updated file regenerates. Returns `None` for missing sources.
pub(crate) fn thumb_cache_path(
  cache_dir: &std::path::Path,
  source: &std::path::Path,
) -> Option<std::path::PathBuf> {
  let meta = std::fs::metadata(source).ok()?;
  if !meta.is_file() {
    return None;
  }
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};
  let mut hash = DefaultHasher::new();
  source.to_string_lossy().hash(&mut hash);
  meta.len().hash(&mut hash);
  meta.modified().ok().hash(&mut hash);
  Some(cache_dir.join(format!("{:016x}.png", hash.finish())))
}

/// Scaled-down cached PNG for a source image (4K/6K files stay on disk).
/// Falls back to the source path when caching fails.
pub(crate) fn cached_thumb(source: &std::path::Path) -> Option<std::path::PathBuf> {
  let cache_dir = thumb_cache_dir();
  let cached = thumb_cache_path(&cache_dir, source)?;
  if cached.is_file() {
    return Some(cached);
  }
  if std::fs::create_dir_all(&cache_dir).is_err() {
    return Some(source.to_path_buf());
  }
  let pixbuf = gdk_pixbuf::Pixbuf::from_file_at_scale(
    source.to_str()?,
    THUMB_MAX_PX,
    THUMB_MAX_PX,
    true,
  )
  .ok()?;
  if pixbuf.savev(&cached, "png", &[]).is_err() {
    return Some(source.to_path_buf());
  }
  Some(cached)
}

/// Rounded thumbnail: cached small file in a cropped `Picture` with a
/// border radius (full-res images are never loaded into the UI).
/// Returns `None` when the source is missing.
fn thumb_picture(path: &str, width: i32, height: i32) -> Option<gtk::Picture> {
  if path.is_empty() {
    return None;
  }
  let source = std::path::Path::new(path);
  if !source.is_file() {
    return None;
  }
  let file = cached_thumb(source).unwrap_or_else(|| source.to_path_buf());
  let picture = gtk::Picture::for_filename(file.to_str()?);
  picture.set_content_fit(gtk::ContentFit::Cover);
  picture.set_size_request(width, height);
  picture.add_css_class("wallpaper-thumb");
  crate::UIKit::apply_css(
    &picture,
    &format!(
      "picture.wallpaper-thumb {{ border-radius: {}px; }}",
      THUMB_CORNER_PX
    ),
  );
  Some(picture)
}

/// Thumbnail cell: rounded preview (when the file exists) plus name below.
fn thumb_cell(name: &str, path: &str, width: i32, height: i32, pal_fg: &str) -> gtk::Box {
  let cell = gtk::Box::new(gtk::Orientation::Vertical, 6);
  if let Some(picture) = thumb_picture(path, width, height) {
    picture.set_halign(gtk::Align::Center);
    cell.append(&picture);
  }
  let label = markup_label(name, 12, "normal", pal_fg);
  label.set_halign(gtk::Align::Center);
  label.set_xalign(0.5);
  label.set_wrap(true);
  label.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  label.set_max_width_chars(14);
  cell.append(&label);
  cell
}

/// Fill the premade grid from a wallpaper state.
fn fill_lists(premade_grid: &gtk::FlowBox, state: &daemon::WallpaperState, pal_fg: &str) {
  while let Some(child) = premade_grid.first_child() {
    premade_grid.remove(&child);
  }
  for entry in &state.premade {
    let cell = thumb_cell(&entry.name, &entry.path, PREMADE_THUMB_W, PREMADE_THUMB_H, pal_fg);
    premade_grid.insert(&cell, -1);
  }
}

/// The Wallpaper detail page (directly on the screen).
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(super::is_dark());
  let fg: &'static str = pal.fg;
  let secondary: &'static str = pal.secondary;
  let card_color: &'static str = pal.card;

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 16);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  let state = daemon::wallpaper_get().unwrap_or_default();

  // Current wallpaper card: preview plus name and fill mode.
  let current_card = card(card_color);
  let current_row = gtk::Box::new(gtk::Orientation::Horizontal, 16);
  current_row.set_hexpand(true);
  if let Some(entry) = state.current.as_ref() {
    if let Some(thumb) = thumb_picture(&entry.path, CURRENT_THUMB_W, CURRENT_THUMB_H) {
      thumb.set_valign(gtk::Align::Start);
      current_row.append(&thumb);
    }
  }
  let current_text = gtk::Box::new(gtk::Orientation::Vertical, 4);
  current_text.set_hexpand(true);
  current_text.append(&section_label("wallpaper.current", secondary));
  let current_name = state
    .current
    .as_ref()
    .map(|e| e.name.clone())
    .unwrap_or_else(|| lang::t("wallpaper.no_wallpaper"));
  let name = markup_label(&current_name, 15, "bold", fg);
  name.set_halign(gtk::Align::Start);
  name.set_xalign(0.0);
  current_text.append(&name);
  current_text.append(&section_label("wallpaper.fill_mode", secondary));
  let fill_labels: Vec<String> = FILL_ORDER
    .iter()
    .map(|mode| lang::t(&fill_key(mode)))
    .collect();
  let fill_refs: Vec<&str> = fill_labels.iter().map(String::as_str).collect();
  let fill_drop = gtk::DropDown::from_strings(&fill_refs);
  fill_drop.set_selected(fill_index(&state.fill));
  fill_drop.set_halign(gtk::Align::Start);
  let fill_selected = std::rc::Rc::new(std::cell::Cell::new(fill_index(&state.fill)));
  let fill_selected_cb = fill_selected.clone();
  fill_drop.connect_selected_notify(move |drop| {
    let index = drop.selected();
    let id = FILL_ORDER.get(index as usize).copied().unwrap_or("fill");
    match daemon::wallpaper_set_fill(id) {
      Ok(applied) => {
        println!("Wallpaper fill mode: {}", applied);
        fill_selected_cb.set(fill_index(&applied));
      }
      Err(e) => {
        println!("Wallpaper fill mode failed: {}", e);
        drop.set_selected(fill_selected_cb.get());
      }
    }
  });
  current_text.append(&fill_drop);
  current_row.append(&current_text);
  current_card.append(&current_row);
  detail.append(&current_card);

  // Available wallpapers card: premade grid (custom uploads come later).
  let available_card = card(card_color);
  let available_title = markup_label(&lang::t("wallpaper.available"), 15, "bold", fg);
  available_title.set_halign(gtk::Align::Start);
  available_title.set_xalign(0.0);
  available_title.set_hexpand(true);
  available_card.append(&available_title);

  let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
  separator.set_margin_top(12);
  separator.set_margin_bottom(12);
  available_card.append(&separator);

  let lists = gtk::Box::new(gtk::Orientation::Vertical, 8);
  lists.set_hexpand(true);
  lists.append(&section_label("wallpaper.premade", secondary));
  let premade_grid = gtk::FlowBox::new();
  premade_grid.set_selection_mode(gtk::SelectionMode::None);
  // No per-line cap: the boxes flow responsively, as many per row as the
  // window width fits.
  premade_grid.set_row_spacing(12);
  premade_grid.set_column_spacing(12);
  premade_grid.set_hexpand(true);
  lists.append(&premade_grid);
  fill_lists(&premade_grid, &state, fg);
  available_card.append(&lists);
  detail.append(&available_card);

  detail.upcast()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fill_modes_resolve_in_order() {
    assert_eq!(FILL_ORDER, &["fill", "fit", "stretch", "center", "tile"]);
    assert_eq!(fill_index("fill"), 0);
    assert_eq!(fill_index("tile"), 4);
    assert_eq!(fill_index("melt"), 0);
    assert_eq!(fill_key("center"), "wallpaper.fill.center");
  }

  #[test]
  fn thumb_cache_key_tracks_file_and_misses_missing() {
    let dir = std::env::temp_dir().join("systemsettings-thumb-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let source = dir.join("a.png");
    assert!(thumb_cache_path(&dir, &source).is_none());
    std::fs::write(&source, b"fake-png").unwrap();
    let first = thumb_cache_path(&dir, &source).unwrap();
    assert_eq!(first.extension().and_then(|e| e.to_str()), Some("png"));
    assert_eq!(thumb_cache_path(&dir, &source).unwrap(), first);
    let _ = std::fs::remove_dir_all(&dir);
  }
}
