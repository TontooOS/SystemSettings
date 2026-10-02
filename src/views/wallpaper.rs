//! Wallpaper settings page for SystemSettings.
//!
//! Current wallpaper block (preview, name, fill mode dropdown) plus the
//! available wallpapers: one tappable list per group. Picking a row opens
//! the apply sheet with the Light/Auto/Dark variant preview; applying goes
//! through the settings daemon (`wallpaper_apply`) and the page refreshes.
//! Nothing here renders the desktop itself.

use std::path::{Path, PathBuf};

use crate::daemon;
use crate::lang;
use crate::views::{
  caption, header_subtitle, note, Nav, PageView, Skin, BLOCK_GAP, WALLPAPER,
};
use crate::CoreImage::{FilterType, ImageFormat, TiImage};
use crate::TontooUI::elements::{
  Align, BasicOutlineGroup, FileImage, Form, FormRow, FormSection, HStack, ImageFit, OutlineNode,
  VStack,
};

/// Fill mode ids in dropdown order (daemon `wallpaper` values).
pub(crate) const FILL_ORDER: &[&str] = &["fill", "fit", "stretch", "center", "tile"];

/// Popup preview variants in button order (Auto in the middle).
pub(crate) const PREVIEW_ORDER: &[&str] = &["light", "auto", "dark"];

/// Longest cached thumbnail edge in px (4K originals stay on disk).
const THUMB_MAX_PX: u32 = 640;
/// Longest preview edge used for the Auto split composite.
const PREVIEW_PX: u32 = 640;

/// Lang key for a fill mode label (`wallpaper.fill.<id>`).
pub(crate) fn fill_key(fill: &str) -> String {
  format!("wallpaper.fill.{}", fill)
}

/// Lang key for a popup preview variant (`wallpaper.mode.<id>`).
pub(crate) fn mode_key(variant: &str) -> String {
  format!("wallpaper.mode.{}", variant)
}

/// Index of a fill mode in the dropdown order (0 when unknown).
pub(crate) fn fill_index(fill: &str) -> usize {
  FILL_ORDER
    .iter()
    .position(|mode| *mode == fill)
    .unwrap_or(0)
}

/// Directory caching scaled-down thumbnails (never the full images).
pub(crate) fn thumb_cache_dir() -> PathBuf {
  std::env::temp_dir().join("tontoo-wallpaper-thumbs")
}

/// Cache file for a source image, keyed by path, size and mtime, so an
/// updated file regenerates. Returns `None` for missing sources.
pub(crate) fn thumb_cache_path(
  cache_dir: &Path,
  source: &Path,
) -> Option<PathBuf> {
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

/// Scaled-down cached PNG for a source image. Falls back to the source
/// path when caching fails, and never fails for an existing file.
pub(crate) fn cached_thumb(source: &Path) -> Option<PathBuf> {
  let cache_dir = thumb_cache_dir();
  let cached = thumb_cache_path(&cache_dir, source)?;
  if cached.is_file() {
    return Some(cached);
  }
  if std::fs::create_dir_all(&cache_dir).is_err() {
    return Some(source.to_path_buf());
  }
  let path = source.to_str()?;
  let image = TiImage::thumbnail_fast(path, THUMB_MAX_PX).ok()?;
  if image.save(&cached.to_string_lossy(), ImageFormat::Png, 90).is_err() {
    return Some(source.to_path_buf());
  }
  Some(cached)
}

/// Diagonal light/dark split preview: left of the drifting divider shows
/// the light image, right shows the dark image, joined by a white line.
/// Cached next to the other thumbnails.
pub(crate) fn split_preview(
  light: &Path,
  dark: &Path,
  width: u32,
  height: u32,
) -> Option<PathBuf> {
  let cache_dir = thumb_cache_dir();
  let _ = std::fs::create_dir_all(&cache_dir);
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};
  let mut hash = DefaultHasher::new();
  light.to_string_lossy().hash(&mut hash);
  dark.to_string_lossy().hash(&mut hash);
  width.hash(&mut hash);
  height.hash(&mut hash);
  std::fs::metadata(light).ok()?.len().hash(&mut hash);
  std::fs::metadata(dark).ok()?.len().hash(&mut hash);
  let dest = cache_dir.join(format!("split-{:016x}.png", hash.finish()));
  if dest.is_file() {
    return Some(dest);
  }

  let light_image = TiImage::load(light.to_str()?).ok()?
    .resize(width, height, FilterType::Triangle);
  let dark_image = TiImage::load(dark.to_str()?).ok()?
    .resize(width, height, FilterType::Triangle);
  let light_pixels = light_image.as_rgba().as_raw().clone();
  let dark_pixels = dark_image.as_rgba().as_raw().clone();

  let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
  for y in 0..height {
    // Divider drifts right going down: left stays light, right is dark.
    let line = width as f32 * (0.38 + 0.24 * (y as f32 / height as f32));
    for x in 0..width {
      let offset = ((y * width + x) * 4) as usize;
      let dx = x as f32 - line;
      let source = if dx.abs() <= 2.0 {
        None
      } else if dx < 0.0 {
        Some(&light_pixels)
      } else {
        Some(&dark_pixels)
      };
      match source {
        None => pixels[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]),
        Some(src) => pixels[offset..offset + 4].copy_from_slice(&src[offset..offset + 4]),
      }
    }
  }
  let out = TiImage::new(width, height, pixels).ok()?;
  out.save(&dest.to_string_lossy(), ImageFormat::Png, 90).ok()?;
  Some(dest)
}

/// Small cached variant of one wallpaper entry, or `None` when missing.
fn small(path: &str) -> Option<PathBuf> {
  if path.is_empty() {
    return None;
  }
  let source = PathBuf::from(path);
  if !source.is_file() {
    return None;
  }
  Some(cached_thumb(&source).unwrap_or(source))
}

/// Preview file for one popup variant: light and dark resolve to cached
/// small files, auto composites the diagonal split from the small
/// thumbnails so a 6K original is never decoded.
pub(crate) fn preview_file(entry: &daemon::WallpaperEntry, variant: &str) -> Option<PathBuf> {
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
      let light_source = PathBuf::from(&entry.path);
      let dark_source = if entry.path_dark.is_empty() {
        light_source.clone()
      } else {
        PathBuf::from(&entry.path_dark)
      };
      if !light_source.is_file() || !dark_source.is_file() {
        return small(&entry.path);
      }
      let light_small = cached_thumb(&light_source).unwrap_or(light_source.clone());
      let dark_small = cached_thumb(&dark_source).unwrap_or(dark_source);
      // Single-variant pack: plain image, no fake divider.
      if light_small == dark_small {
        return small(&entry.path);
      }
      split_preview(&light_small, &dark_small, PREVIEW_PX, PREVIEW_PX / 2)
        .or_else(|| small(&entry.path))
    }
    _ => small(&entry.path),
  }
}

/// Tappable wallpaper list. Picking a row asks the app to open the apply
/// sheet for that entry.
fn wallpaper_list(entries: &[daemon::WallpaperEntry], nav: Nav) -> BasicOutlineGroup {
  let picks: Vec<daemon::WallpaperEntry> = entries.to_vec();
  let nodes: Vec<OutlineNode> = entries
    .iter()
    .map(|entry| OutlineNode::file(entry.name.clone()).icon("photo"))
    .collect();
  BasicOutlineGroup::new(nodes)
    .selectable(true)
    .trailing_chevron(false)
    .on_select(move |path| {
      let Some(&index) = path.first() else {
        return;
      };
      if let Some(entry) = picks.get(index) {
        nav.request_wallpaper(entry.clone());
      }
    })
}

/// Current wallpaper block: preview plus the fill mode dropdown.
fn current_block(state: &daemon::WallpaperState, nav: Nav) -> VStack {
  let labels: Vec<String> = FILL_ORDER
    .iter()
    .map(|mode| lang::t(&fill_key(mode)))
    .collect();
  let selected = fill_index(&state.fill);
  let row = FormRow::picker(lang::t("wallpaper.fill_mode"), labels, selected).on_pick(move |index| {
    let Some(mode) = FILL_ORDER.get(index) else {
      return;
    };
    match daemon::wallpaper_set_fill(mode) {
      Ok(applied) => println!("Wallpaper fill mode: {applied}"),
      Err(err) => println!("Wallpaper fill mode failed: {err}"),
    }
    nav.touch();
  });

  VStack::new()
    .spacing(10.0)
    .align(Align::Leading)
    .child(caption(&lang::t("wallpaper.current")))
    .child(current_preview(state))
    .child(Form::new().section(FormSection::new().row(row)))
}

/// Current wallpaper preview: the cached thumbnail, or the localized
/// placeholder when nothing is set.
fn current_preview(state: &daemon::WallpaperState) -> HStack {
  let path = state
    .current
    .as_ref()
    .and_then(|entry| small(&entry.path));
  match path {
    Some(path) => HStack::new().align(Align::Leading).child(
      FileImage::new(path, 300.0, 169.0).radius(12.0).fit(ImageFit::Cover),
    ),
    None => HStack::new()
      .align(Align::Leading)
      .child(note(&lang::t("wallpaper.no_wallpaper"))),
  }
}

/// Build the Wallpaper detail page.
pub(crate) fn build(_skin: &Skin, nav: &Nav) -> PageView {
  let state = daemon::wallpaper_get().unwrap_or_default();
  let mut body = VStack::new()
    .spacing(BLOCK_GAP)
    .align(Align::Leading)
    .child(current_block(&state, nav.clone()));

  if !state.customs.is_empty() {
    body = body
      .child(caption(&lang::t("wallpaper.custom")))
      .child(wallpaper_list(&state.customs, nav.clone()));
  }
  if !state.premade.is_empty() {
    body = body
      .child(caption(&lang::t("wallpaper.premade")))
      .child(wallpaper_list(&state.premade, nav.clone()));
  } else {
    body = body.child(note(&lang::t("wallpaper.no_wallpapers")));
  }

  crate::views::page_shell(
    crate::views::page_header(crate::views::header_symbol(WALLPAPER), &header_subtitle(WALLPAPER)),
    body,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  fn entry(id: &str, light: &str, dark: &str) -> daemon::WallpaperEntry {
    daemon::WallpaperEntry {
      kind: "premade".to_string(),
      id: id.to_string(),
      name: id.to_string(),
      path: light.to_string(),
      path_dark: dark.to_string(),
    }
  }

  #[test]
  fn fill_modes_resolve_in_order() {
    assert_eq!(FILL_ORDER, &["fill", "fit", "stretch", "center", "tile"]);
    assert_eq!(fill_index("fill"), 0);
    assert_eq!(fill_index("tile"), 4);
    assert_eq!(fill_index("melt"), 0);
    assert_eq!(fill_key("center"), "wallpaper.fill.center");
  }

  #[test]
  fn preview_modes_resolve() {
    assert_eq!(mode_key("auto"), "wallpaper.mode.auto");
    assert_eq!(PREVIEW_ORDER, &["light", "auto", "dark"]);
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

  fn solid_png(dir: &Path, name: &str, pixel: [u8; 3]) -> PathBuf {
    let image = TiImage::solid(16, 12, crate::CoreImage::Rgba8::new(pixel[0], pixel[1], pixel[2], 255))
      .unwrap();
    let path = dir.join(name);
    image.save(&path.to_string_lossy(), ImageFormat::Png, 90).unwrap();
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
    let image = TiImage::load(&split.to_string_lossy()).unwrap();
    let pixels = image.as_rgba().as_raw();
    assert_eq!(image.dimensions(), (16, 12));
    let at = |x: u32, y: u32| {
      let offset = ((y * 16 + x) * 4) as usize;
      [pixels[offset], pixels[offset + 1], pixels[offset + 2]]
    };
    // Top-left corner is light, bottom-right corner is dark.
    assert_eq!(at(0, 0), [200, 50, 50]);
    assert_eq!(at(15, 11), [50, 50, 200]);
    let white = pixels.chunks_exact(4).filter(|px| *px == [255, 255, 255, 255]).count();
    assert!(white > 0);
    assert!(split_preview(&dir.join("missing.png"), &dark, 16, 12).is_none());
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn preview_file_prefers_small_files_and_skips_divider_for_singles() {
    let dir = std::env::temp_dir().join("systemsettings-preview-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let light = solid_png(&dir, "day.png", [200, 50, 50]).to_str().unwrap().to_string();
    let dark = solid_png(&dir, "night.png", [50, 50, 200]).to_str().unwrap().to_string();

    // Two variants: auto composites from the small thumbs.
    let duo = entry("DUO", &light, &dark);
    assert!(preview_file(&duo, "auto").unwrap().is_file());
    let light_file = preview_file(&duo, "light").unwrap();
    let dark_file = preview_file(&duo, "dark").unwrap();
    assert!(light_file.is_file());
    assert!(dark_file.is_file());
    assert_ne!(light_file, dark_file);

    // Single variant: plain image, no divider stripe.
    let solo = entry("SOLO", &light, "");
    let plain = preview_file(&solo, "auto").unwrap();
    let image = TiImage::load(&plain.to_string_lossy()).unwrap();
    let white = image
      .as_rgba()
      .as_raw()
      .chunks_exact(4)
      .filter(|px| *px == [255, 255, 255, 255])
      .count();
    assert_eq!(white, 0);

    // Missing files yield no preview.
    assert!(preview_file(&entry("MISS", "/none", ""), "auto").is_none());
    let _ = std::fs::remove_dir_all(&dir);
  }
}
