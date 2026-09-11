//! Wallpaper settings page for SystemSettings.
//!
//! Current wallpaper card (preview thumbnail, name, fill mode dropdown)
//! plus an available wallpapers card (Browse button, horizontal custom
//! row, premade grid). Everything is display only except the fill mode
//! dropdown (persisted via `wallpaper_set_fill`) and Browse (uploads via
//! `wallpaper_add`); nothing here applies the wallpaper to the desktop.
//! All data comes from the settings daemon (`wallpaper_get`) with empty
//! fallbacks when it is unreachable. All text uses SF Pro Display and
//! both `en_us` and `de_de` strings.

use super::{markup_label, palette};
use crate::daemon;
use crate::lang;
use gtk::prelude::*;

const CURRENT_THUMB_PX: i32 = 64;
const CUSTOM_THUMB_PX: i32 = 72;
const PREMADE_THUMB_PX: i32 = 96;

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
fn card(pal_card: &str) -> gtk::Box {
  let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
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

/// Thumbnail cell: preview image (when the file exists) plus name below.
fn thumb_cell(name: &str, path: &str, px: i32, pal_fg: &str) -> gtk::Box {
  let cell = gtk::Box::new(gtk::Orientation::Vertical, 6);
  if !path.is_empty() && std::path::Path::new(path).is_file() {
    let image = gtk::Image::from_file(path);
    image.set_pixel_size(px);
    image.set_halign(gtk::Align::Center);
    cell.append(&image);
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
/// Shows the "No wallpapers found." placeholder for an empty customs row.
fn fill_lists(
  customs_area: &gtk::Box,
  premade_grid: &gtk::FlowBox,
  state: &daemon::WallpaperState,
  pal_fg: &str,
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
      row.append(&thumb_cell(&entry.name, &entry.path, CUSTOM_THUMB_PX, pal_fg));
    }
    scroll.set_child(Some(&row));
    customs_area.append(&scroll);
  }

  while let Some(child) = premade_grid.first_child() {
    premade_grid.remove(&child);
  }
  for entry in &state.premade {
    let cell = thumb_cell(&entry.name, &entry.path, PREMADE_THUMB_PX, pal_fg);
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
    if !entry.path.is_empty() && std::path::Path::new(&entry.path).is_file() {
      let thumb = gtk::Image::from_file(&entry.path);
      thumb.set_pixel_size(CURRENT_THUMB_PX);
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
  premade_grid.set_max_children_per_line(4);
  premade_grid.set_row_spacing(12);
  premade_grid.set_column_spacing(12);
  premade_grid.set_hexpand(true);
  lists.append(&premade_grid);
  fill_lists(&customs_area, &premade_grid, &state, fg);
  available_card.append(&lists);
  detail.append(&available_card);

  // Browse uploads into the customs and refreshes the lists.
  let customs_refresh = customs_area.clone();
  let premade_refresh = premade_grid.clone();
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
                fill_lists(&customs_done, &premade_done, &fresh, fg);
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
}
