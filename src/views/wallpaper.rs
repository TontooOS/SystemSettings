//! Wallpaper settings page for SystemSettings.
//!
//! Current wallpaper card (preview thumbnail, name, fill mode dropdown)
//! plus an available wallpapers card (Browse button, horizontal custom
//! row, clickable premade grid). Custom cells switch straight to the
//! wallpaper on click (no popup); premade cells open the apply popup
//! with Light/Auto/Dark variants. Browse uploads an image file (daemon
//! converts to PNG). Nothing here renders the desktop itself; all data
//! and writes go through the settings daemon (`wallpaper_get` and
//! friends) with empty fallbacks when it is unreachable. All text uses
//! SF Pro Display and both `en_us` and `de_de` strings.

use super::{markup_label, palette, SF_PRO};
use crate::daemon;
use crate::lang;
use gtk::prelude::*;
use std::rc::Rc;

const CURRENT_THUMB_W: i32 = 160;
const CURRENT_THUMB_H: i32 = 100;
const CUSTOM_THUMB_W: i32 = 112;
const CUSTOM_THUMB_H: i32 = 72;
const PREMADE_THUMB_W: i32 = 160;
const PREMADE_THUMB_H: i32 = 100;
/// Longest cached thumbnail edge (crisp on HiDPI, tiny on disk).
const THUMB_MAX_PX: i32 = 320;
const THUMB_CORNER_PX: i32 = 12;

/// Fill mode ids in dropdown order (daemon `wallpaper` values).
pub(crate) const FILL_ORDER: &[&str] = &["fill", "fit", "stretch", "center", "tile"];

/// Popup preview variants in button order (Auto in the middle).
pub(crate) const PREVIEW_ORDER: &[&str] = &["light", "auto", "dark"];

/// Popup preview size.
const PREVIEW_W: u32 = 480;
const PREVIEW_H: u32 = 270;

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

/// Refresh a label created by `markup_label` (same font).
fn set_markup_label(label: &gtk::Label, text: &str, size: u32, weight: &str, color: &str) {
  label.set_markup(&format!(
    "<span font_desc=\"{} {} {}\" foreground=\"{}\">{}</span>",
    SF_PRO,
    weight,
    size,
    color,
    glib::markup_escape_text(text),
  ));
}

/// Lang key for a popup preview variant (`wallpaper.mode.<id>`).
pub(crate) fn mode_key(variant: &str) -> String {
  format!("wallpaper.mode.{}", variant)
}

/// Diagonal light/dark split preview: left of the diagonal line shows the
/// light image, right shows the dark image, joined by a white divider
/// (not a straight middle cut). Cached next to the thumbnails.
pub(crate) fn split_preview(
  light: &std::path::Path,
  dark: &std::path::Path,
  width: u32,
  height: u32,
) -> Option<std::path::PathBuf> {
  let light_img = image::open(light).ok()?.resize_exact(
    width,
    height,
    image::imageops::FilterType::Triangle,
  );
  let dark_img = image::open(dark).ok()?.resize_exact(
    width,
    height,
    image::imageops::FilterType::Triangle,
  );
  let light_rgb = light_img.to_rgb8();
  let dark_rgb = dark_img.to_rgb8();
  let mut out = image::RgbImage::new(width, height);
  for y in 0..height {
    // Divider drifts right going down: left stays light, right is dark.
    let line = width as f32 * (0.38 + 0.24 * (y as f32 / height as f32));
    for x in 0..width {
      let dx = x as f32 - line;
      let pixel = if dx.abs() <= 2.0 {
        image::Rgb([255, 255, 255])
      } else if dx < 0.0 {
        *light_rgb.get_pixel(x, y)
      } else {
        *dark_rgb.get_pixel(x, y)
      };
      out.put_pixel(x, y, pixel);
    }
  }
  let cache_dir = thumb_cache_dir();
  std::fs::create_dir_all(&cache_dir).ok()?;
  let dest = cache_dir.join(format!(
    "split-{:016x}.png",
    {
      use std::collections::hash_map::DefaultHasher;
      use std::hash::{Hash, Hasher};
      let mut hash = DefaultHasher::new();
      light.to_string_lossy().hash(&mut hash);
      dark.to_string_lossy().hash(&mut hash);
      width.hash(&mut hash);
      height.hash(&mut hash);
      std::fs::metadata(light).ok()?.len().hash(&mut hash);
      std::fs::metadata(dark).ok()?.len().hash(&mut hash);
      hash.finish()
    }
  ));
  if !dest.is_file() {
    out.save_with_format(&dest, image::ImageFormat::Png).ok()?;
  }
  Some(dest)
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

/// Exact-size cached thumbnail (cover-cropped to `width` x `height`):
/// dimensions come from the file header (no decode), then a fast
/// scale-on-load plus a center crop. The texture matches the display
/// size, so grid cells keep their width and rows flow responsively.
/// Falls back to the aspect-kept thumbnail on any error.
pub(crate) fn cached_thumb_exact(
  source: &std::path::Path,
  width: i32,
  height: i32,
) -> Option<std::path::PathBuf> {
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};

  let meta = std::fs::metadata(source).ok()?;
  if !meta.is_file() || width <= 0 || height <= 0 {
    return None;
  }
  let cache_dir = thumb_cache_dir();
  let mut hash = DefaultHasher::new();
  source.to_string_lossy().hash(&mut hash);
  meta.len().hash(&mut hash);
  meta.modified().ok().hash(&mut hash);
  (width as u32).hash(&mut hash);
  (height as u32).hash(&mut hash);
  let cached = cache_dir.join(format!("exact-{:016x}.png", hash.finish()));
  if cached.is_file() {
    return Some(cached);
  }
  if std::fs::create_dir_all(&cache_dir).is_err() {
    return None;
  }
  let (_format, src_w, src_h) = gdk_pixbuf::Pixbuf::file_info(source.to_str()?)?;
  if src_w <= 0 || src_h <= 0 {
    return None;
  }
  // Scale so the image covers the target, then center-crop.
  let scale = (width as f64 / src_w as f64).max(height as f64 / src_h as f64);
  let load_w = (src_w as f64 * scale).ceil() as i32;
  let load_h = (src_h as f64 * scale).ceil() as i32;
  let pixbuf = gdk_pixbuf::Pixbuf::from_file_at_scale(source.to_str()?, load_w, load_h, true).ok()?;
  let (buf_w, buf_h) = (pixbuf.width(), pixbuf.height());
  if buf_w < width || buf_h < height {
    return None;
  }
  let crop = gdk_pixbuf::Pixbuf::new_subpixbuf(&pixbuf, (buf_w - width) / 2, (buf_h - height) / 2, width, height);
  if crop.savev(&cached, "png", &[]).is_err() {
    return None;
  }
  Some(cached)
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

/// Rounded thumbnail: exact-size cached file in a cropped `Picture` with
/// a border radius, so the texture matches the display size and grid
/// cells keep their width. Falls back to the aspect-kept thumbnail, then
/// the source. Returns `None` when the source is missing.
fn thumb_picture(path: &str, width: i32, height: i32) -> Option<gtk::Picture> {
  if path.is_empty() {
    return None;
  }
  let source = std::path::Path::new(path);
  if !source.is_file() {
    return None;
  }
  let file = cached_thumb_exact(source, width, height)
    .or_else(|| cached_thumb(source))
    .unwrap_or_else(|| source.to_path_buf());
  picture_from_file(&file, width, height)
}

/// Small `Picture` with cover fit and rounded corners. The file must
/// already be thumbnail-sized (never a 4K original).
fn picture_from_file(file: &std::path::Path, width: i32, height: i32) -> Option<gtk::Picture> {
  let picture = gtk::Picture::for_filename(file.to_str()?);
  picture.set_content_fit(gtk::ContentFit::Cover);
  // Fixed cell size: no expand (Picture expands by default and would
  // stretch every grid row full width), shrinkable below the intrinsic
  // texture size so boxes flow responsively.
  picture.set_hexpand(false);
  picture.set_vexpand(false);
  picture.set_can_shrink(true);
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

/// Fill the customs row and the premade grid from a wallpaper state.
/// Custom cells switch straight to the wallpaper on click (no popup);
/// premade cells open the apply popup. Shows the "No wallpapers found."
/// placeholder for an empty customs row.
fn fill_lists(
  customs_area: &gtk::Box,
  premade_grid: &gtk::FlowBox,
  state: &daemon::WallpaperState,
  pal_fg: &str,
  on_applied: &Rc<dyn Fn(daemon::WallpaperEntry)>,
) {
  while let Some(child) = customs_area.first_child() {
    customs_area.remove(&child);
  }
  if state.customs.is_empty() {
    let empty = markup_label(&lang::t("wallpaper.no_wallpapers"), 13, "normal", pal_fg);
    empty.set_halign(gtk::Align::Center);
    empty.set_xalign(0.5);
    empty.set_hexpand(true);
    empty.set_margin_top(12);
    empty.set_margin_bottom(12);
    customs_area.append(&empty);
  } else {
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Never);
    scroll.set_hexpand(true);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    for entry in &state.customs {
      let cell = thumb_cell(&entry.name, &entry.path, CUSTOM_THUMB_W, CUSTOM_THUMB_H, pal_fg);
      if let Some(cursor) = gtk::gdk::Cursor::from_name("pointer", None) {
        cell.set_cursor(Some(&cursor));
      }
      let switch_entry = entry.clone();
      let switch_applied = on_applied.clone();
      let gesture = gtk::GestureClick::new();
      gesture.connect_released(move |_, _, _, _| {
        match daemon::wallpaper_apply(&switch_entry.kind, &switch_entry.id, "light") {
          Ok(applied) => {
            println!("Wallpaper applied: {}", applied.path);
            switch_applied(applied);
          }
          Err(e) => println!("Wallpaper apply failed: {}", e),
        }
      });
      cell.add_controller(gesture);
      row.append(&cell);
    }
    scroll.set_child(Some(&row));
    customs_area.append(&scroll);
  }

  while let Some(child) = premade_grid.first_child() {
    premade_grid.remove(&child);
  }
  for entry in &state.premade {
    let cell = thumb_cell(&entry.name, &entry.path, PREMADE_THUMB_W, PREMADE_THUMB_H, pal_fg);
    if let Some(cursor) = gtk::gdk::Cursor::from_name("pointer", None) {
      cell.set_cursor(Some(&cursor));
    }
    let popup_entry = entry.clone();
    let popup_applied = on_applied.clone();
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(move |gesture, _, _, _| {
      let parent = gesture
        .widget()
        .and_then(|w| w.root())
        .and_then(|root| root.downcast::<gtk::Window>().ok());
      open_popup(&popup_entry, parent, popup_applied.clone());
    });
    cell.add_controller(gesture);
    premade_grid.insert(&cell, -1);
  }
}

/// Image file filters for the Browse dialog: png, jpeg, webp plus every
/// image format.
fn browse_filters() -> Vec<gtk::FileFilter> {
  let mut filters = Vec::new();
  let png = gtk::FileFilter::new();
  png.set_name(Some("PNG"));
  png.add_mime_type("image/png");
  png.add_pattern("*.png");
  filters.push(png);
  let jpeg = gtk::FileFilter::new();
  jpeg.set_name(Some("JPEG"));
  jpeg.add_mime_type("image/jpeg");
  jpeg.add_pattern("*.jpg");
  jpeg.add_pattern("*.jpeg");
  filters.push(jpeg);
  let webp = gtk::FileFilter::new();
  webp.set_name(Some("WebP"));
  webp.add_mime_type("image/webp");
  webp.add_pattern("*.webp");
  filters.push(webp);
  let all = gtk::FileFilter::new();
  all.set_name(Some(&lang::t("wallpaper.all_images")));
  all.add_mime_type("image/*");
  filters.push(all);
  filters
}

/// Preview file for a popup variant, always thumbnail-sized (never 4K):
/// light and dark resolve to cached small files (fallback to the default
/// path), auto composites the diagonal split.
fn preview_file(entry: &daemon::WallpaperEntry, variant: &str) -> Option<std::path::PathBuf> {
  let small = |path: &str| {
    if path.is_empty() {
      None
    } else {
      let source = std::path::PathBuf::from(path);
      if !source.is_file() {
        return None;
      }
      Some(cached_thumb(&source).unwrap_or(source))
    }
  };
  match variant {
    "dark" => {
      let dark = if entry.path_dark.is_empty() {
        entry.path.clone()
      } else {
        entry.path_dark.clone()
      };
      small(&dark)
    }
    "auto" => {
      let light_src = std::path::PathBuf::from(&entry.path);
      let dark_src = if entry.path_dark.is_empty() {
        light_src.clone()
      } else {
        std::path::PathBuf::from(&entry.path_dark)
      };
      if !light_src.is_file() || !dark_src.is_file() {
        return small(&entry.path);
      }
      // Fast path: composite from the cached small thumbnails instead of
      // decoding the 4K/6K originals (that blocked the popup for seconds).
      let light_small = cached_thumb(&light_src).unwrap_or(light_src.clone());
      let dark_small = cached_thumb(&dark_src).unwrap_or(dark_src);
      if light_small == dark_small {
        // Single-variant pack: plain image, no fake divider.
        return small(&entry.path);
      }
      split_preview(&light_small, &dark_small, PREVIEW_W, PREVIEW_H)
        .or_else(|| small(&entry.path))
    }
    _ => small(&entry.path),
  }
}

/// Preview cell width/height inside the popup (three fit side by side).
const POPUP_PREVIEW_W: i32 = 150;
const POPUP_PREVIEW_H: i32 = 95;

/// Apply popup for one wallpaper: Light/Auto/Dark previews side by side
/// (all thumbnail-sized, never 4K), click selects with an accent border,
/// Cancel and Set at the bottom. Borderless modal centered on the
/// Settings window. Set applies to the desktop via the daemon and reports
/// back through `on_applied`; errors only log and keep the popup open.
fn open_popup(
  entry: &daemon::WallpaperEntry,
  parent: Option<gtk::Window>,
  on_applied: Rc<dyn Fn(daemon::WallpaperEntry)>,
) {
  let dialog = gtk::Window::new();
  dialog.set_title(Some(&entry.name));
  dialog.set_modal(true);
  dialog.set_decorated(false);
  if let Some(parent) = parent {
    dialog.set_transient_for(Some(&parent));
  }
  dialog.set_default_size(520, -1);

  let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
  content.set_margin_top(16);
  content.set_margin_bottom(16);
  content.set_margin_start(16);
  content.set_margin_end(16);

  // Three selectable previews side by side, Auto selected by default.
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  row.set_halign(gtk::Align::Center);
  let variant = Rc::new(std::cell::Cell::new(1usize));
  let mut frames: Vec<gtk::Box> = Vec::new();
  for (index, id) in PREVIEW_ORDER.iter().enumerate() {
    let frame = gtk::Box::new(gtk::Orientation::Vertical, 6);
    frame.add_css_class("wp-pick");
    if index == 1 {
      frame.add_css_class("selected");
    }
    crate::UIKit::apply_css(
      &frame,
      ".wp-pick { padding: 3px; border-radius: 14px; border: 3px solid transparent; } \
       .wp-pick.selected { border-color: #FF6B2B; }",
    );
    if let Some(file) = preview_file(entry, id) {
      if let Some(picture) = picture_from_file(&file, POPUP_PREVIEW_W, POPUP_PREVIEW_H) {
        picture.set_halign(gtk::Align::Center);
        frame.append(&picture);
      }
    }
    let label = markup_label(&lang::t(&mode_key(id)), 12, "normal", "#F5F5F7");
    label.set_halign(gtk::Align::Center);
    label.set_xalign(0.5);
    frame.append(&label);
    if let Some(cursor) = gtk::gdk::Cursor::from_name("pointer", None) {
      frame.set_cursor(Some(&cursor));
    }
    frames.push(frame);
  }
  for (index, frame) in frames.iter().enumerate() {
    let siblings = frames.clone();
    let variant_cb = variant.clone();
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(move |_, _, _, _| {
      variant_cb.set(index);
      for (other_index, other) in siblings.iter().enumerate() {
        if other_index == index {
          other.add_css_class("selected");
        } else {
          other.remove_css_class("selected");
        }
      }
    });
    frame.add_controller(gesture);
    row.append(frame);
  }
  content.append(&row);

  dialog.set_child(Some(&content));
  let popup_entry = entry.clone();
  let button_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  button_row.set_halign(gtk::Align::End);
  let cancel = gtk::Button::with_label(&lang::t("wallpaper.cancel"));
  let set = gtk::Button::with_label(&lang::t("wallpaper.set"));
  set.add_css_class("suggested-action");
  button_row.append(&cancel);
  button_row.append(&set);
  content.append(&button_row);

  let dialog_close = dialog.clone();
  cancel.connect_clicked(move |_| {
    dialog_close.destroy();
  });
  let dialog_set = dialog.clone();
  set.connect_clicked(move |_| {
    let id = PREVIEW_ORDER.get(variant.get()).copied().unwrap_or("auto");
    match daemon::wallpaper_apply(&popup_entry.kind, &popup_entry.id, id) {
      Ok(applied) => {
        println!("Wallpaper applied: {} ({})", applied.path, id);
        on_applied(applied);
        dialog_set.destroy();
      }
      Err(e) => println!("Wallpaper apply failed: {}", e),
    }
  });
  dialog.set_child(Some(&content));
  dialog.present();
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
  let current_thumb_slot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
  current_thumb_slot.set_valign(gtk::Align::Start);
  if let Some(entry) = state.current.as_ref() {
    if let Some(thumb) = thumb_picture(&entry.path, CURRENT_THUMB_W, CURRENT_THUMB_H) {
      current_thumb_slot.append(&thumb);
    }
  }
  current_row.append(&current_thumb_slot);
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

  // Available wallpapers card: Browse plus customs row and premade grid.
  let available_card = card(card_color);
  let available_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  available_header.set_hexpand(true);
  let available_title = markup_label(&lang::t("wallpaper.available"), 15, "bold", fg);
  available_title.set_halign(gtk::Align::Start);
  available_title.set_xalign(0.0);
  available_title.set_hexpand(true);
  available_header.append(&available_title);
  let browse = gtk::Button::with_label(&lang::t("wallpaper.browse"));
  browse.set_halign(gtk::Align::End);
  browse.set_valign(gtk::Align::Center);
  available_header.append(&browse);
  available_card.append(&available_header);

  let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
  separator.set_margin_top(12);
  separator.set_margin_bottom(12);
  available_card.append(&separator);

  let lists = gtk::Box::new(gtk::Orientation::Vertical, 8);
  lists.set_hexpand(true);
  lists.append(&section_label("wallpaper.custom", secondary));
  let customs_area = gtk::Box::new(gtk::Orientation::Vertical, 0);
  customs_area.set_hexpand(true);
  lists.append(&customs_area);
  lists.append(&section_label("wallpaper.premade", secondary));
  let premade_grid = gtk::FlowBox::new();
  premade_grid.set_selection_mode(gtk::SelectionMode::None);
  // No per-line cap: the boxes flow responsively, as many per row as the
  // window width fits.
  premade_grid.set_row_spacing(12);
  premade_grid.set_column_spacing(12);
  premade_grid.set_hexpand(true);
  lists.append(&premade_grid);
  // Refresh the current card after an apply; rebuild the lists after Browse.
  let applied_thumb = current_thumb_slot.clone();
  let applied_name = name.clone();
  let on_applied: Rc<dyn Fn(daemon::WallpaperEntry)> = Rc::new(move |applied| {
    while let Some(child) = applied_thumb.first_child() {
      applied_thumb.remove(&child);
    }
    if let Some(thumb) = thumb_picture(&applied.path, CURRENT_THUMB_W, CURRENT_THUMB_H) {
      applied_thumb.append(&thumb);
    }
    set_markup_label(&applied_name, &applied.name, 15, "bold", fg);
  });
  fill_lists(&customs_area, &premade_grid, &state, fg, &on_applied);
  available_card.append(&lists);
  detail.append(&available_card);

  // Browse uploads an image file into the customs and refreshes the lists.
  // The daemon decodes it and stores a PNG (unique names).
  let customs_refresh = customs_area.clone();
  let premade_refresh = premade_grid.clone();
  let browse_applied = on_applied.clone();
  browse.connect_clicked(move |_| {
    let dialog = gtk::FileDialog::new();
    dialog.set_title(&lang::t("wallpaper.browse"));
    dialog.set_accept_label(Some(&lang::t("wallpaper.open")));
    let filters = gtk::gio::ListStore::new::<gtk::FileFilter>();
    for filter in browse_filters() {
      filters.append(&filter);
    }
    dialog.set_filters(Some(&filters));
    let customs_done = customs_refresh.clone();
    let premade_done = premade_refresh.clone();
    let applied_done = browse_applied.clone();
    dialog.open(
      None::<&gtk::Window>,
      None::<&gtk::gio::Cancellable>,
      move |result| match result {
        Ok(file) => {
          if let Some(path) = file.path() {
            let display = path.to_str().unwrap_or_default().to_string();
            match daemon::wallpaper_add(&display, None) {
              Ok(entry) => {
                println!("Wallpaper added: {}", entry.path);
                let fresh = daemon::wallpaper_get().unwrap_or_default();
                fill_lists(&customs_done, &premade_done, &fresh, fg, &applied_done);
              }
              Err(e) => println!("Wallpaper add failed: {}", e),
            }
          }
        }
        Err(e) => println!("Wallpaper browse dismissed: {}", e),
      },
    );
  });

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

  #[test]
  fn exact_thumbs_match_display_size() {
    let dir = std::env::temp_dir().join("systemsettings-exact-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // Wide source (32x8) cover-cropped to a square.
    let mut img = image::RgbImage::new(32, 8);
    for p in img.pixels_mut() {
      *p = image::Rgb([10, 200, 30]);
    }
    let source = dir.join("wide.png");
    img
      .save_with_format(&source, image::ImageFormat::Png)
      .unwrap();
    let thumb = cached_thumb_exact(&source, 8, 8).unwrap();
    assert!(thumb.is_file());
    let decoded = image::open(&thumb).unwrap().to_rgb8();
    assert_eq!(decoded.dimensions(), (8, 8));
    // Cached: same path, no regeneration.
    assert_eq!(cached_thumb_exact(&source, 8, 8).unwrap(), thumb);
    // Missing source yields nothing.
    assert!(cached_thumb_exact(&dir.join("missing.png"), 8, 8).is_none());
    let _ = std::fs::remove_dir_all(&dir);
  }
  fn solid_png(dir: &std::path::Path, name: &str, pixel: [u8; 3]) -> std::path::PathBuf {
    let mut img = image::RgbImage::new(16, 12);
    for p in img.pixels_mut() {
      *p = image::Rgb(pixel);
    }
    let path = dir.join(name);
    img
      .save_with_format(&path, image::ImageFormat::Png)
      .unwrap();
    path
  }

  #[test]
  fn split_preview_joins_light_left_dark_right_with_divider() {
    let dir = std::env::temp_dir().join("systemsettings-split-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let light = solid_png(&dir, "day.png", [200, 50, 50]);
    let dark = solid_png(&dir, "night.png", [50, 50, 200]);
    let split = split_preview(&light, &dark, 16, 12).unwrap();
    assert!(split.is_file());
    let img = image::open(&split).unwrap().to_rgb8();
    assert_eq!(img.dimensions(), (16, 12));
    // Top-left corner is light, bottom-right corner is dark.
    assert_eq!(*img.get_pixel(0, 0), image::Rgb([200, 50, 50]));
    assert_eq!(*img.get_pixel(15, 11), image::Rgb([50, 50, 200]));
    // The diagonal divider leaves a white band somewhere mid-image.
    let white = img.pixels().filter(|p| **p == image::Rgb([255, 255, 255])).count();
    assert!(white > 0);
    // Missing inputs yield no preview.
    assert!(split_preview(&dir.join("missing.png"), &dark, 16, 12).is_none());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn preview_modes_resolve() {
    assert_eq!(mode_key("auto"), "wallpaper.mode.auto");
    assert_eq!(PREVIEW_ORDER, &["light", "auto", "dark"]);
  }

  fn preview_entry(id: &str, light: &str, dark: &str) -> crate::daemon::WallpaperEntry {
    crate::daemon::WallpaperEntry {
      kind: "premade".to_string(),
      id: id.to_string(),
      name: id.to_string(),
      path: light.to_string(),
      path_dark: dark.to_string(),
    }
  }

  #[test]
  fn preview_file_prefers_small_files_and_skips_divider_for_singles() {
    let dir = std::env::temp_dir().join("systemsettings-preview-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let light = solid_png(&dir, "day.png", [200, 50, 50]).to_str().unwrap().to_string();
    let dark = solid_png(&dir, "night.png", [50, 50, 200]).to_str().unwrap().to_string();

    // Two variants: auto composites from the small thumbs (fast open).
    let duo = preview_entry("DUO", &light, &dark);
    let split = preview_file(&duo, "auto").unwrap();
    assert!(split.is_file());
    let img = image::open(&split).unwrap().to_rgb8();
    assert_eq!(img.dimensions(), (PREVIEW_W, PREVIEW_H));
    // Light and dark arms resolve cached small files.
    let light_file = preview_file(&duo, "light").unwrap();
    let dark_file = preview_file(&duo, "dark").unwrap();
    assert!(light_file.is_file());
    assert!(dark_file.is_file());
    assert_ne!(light_file, dark_file);

    // Single variant: plain image, no divider stripe.
    let solo = preview_entry("SOLO", &light, "");
    let plain = preview_file(&solo, "auto").unwrap();
    let img = image::open(&plain).unwrap().to_rgb8();
    assert_eq!(
      img.pixels().filter(|p| **p == image::Rgb([255, 255, 255])).count(),
      0
    );
    // Missing files yield no preview.
    let missing = preview_entry("MISS", "/none", "");
    assert!(preview_file(&missing, "auto").is_none());
    let _ = std::fs::remove_dir_all(&dir);
  }
}
