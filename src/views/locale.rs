//! Language & Region settings page for SystemSettings.
//!
//! System language (English/German plus a "More soon" note), region
//! picker with the full country list and keyboard layout picker with
//! variants plus Auto Detect. Everything applies system-wide through
//! the daemon (`localectl`). All text uses SF Pro Display and both
//! `en_us` and `de_de` strings.

use super::{Palette, is_dark, markup_label, palette};
use crate::daemon;
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::apply_css;
use crate::UIKit::prelude::*;
use gtk::prelude::*;
use std::rc::Rc;
use std::sync::{
  Arc, Mutex,
  atomic::{AtomicBool, Ordering},
};

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

/// Inline markup matching `markup_label`, for updating labels in place.
fn span(text: &str, size: u32, weight: &str, color: &str) -> String {
  format!(
    "<span font_desc=\"{} {} {}\" foreground=\"{}\">{}</span>",
    super::SF_PRO,
    weight,
    size,
    color,
    glib::markup_escape_text(text),
  )
}

/// Filterable option list: search field plus rows. `on_pick` receives
/// the picked item text.
fn option_menu(
  items: &Rc<Vec<String>>,
  pal: &Palette,
  on_pick: Rc<dyn Fn(String)>,
) -> gtk::Box {
  let body = gtk::Box::new(gtk::Orientation::Vertical, 6);
  body.set_hexpand(true);
  body.set_vexpand(true);
  body.set_margin_top(8);
  body.set_margin_bottom(8);
  body.set_margin_start(8);
  body.set_margin_end(8);

  let search = gtk::SearchEntry::new();
  search.set_hexpand(true);
  search.set_placeholder_text(Some(&lang::t("sidebar.search")));
  body.append(&search);

  let scroll = gtk::ScrolledWindow::new();
  scroll.set_hscrollbar_policy(gtk::PolicyType::Never);
  scroll.set_vscrollbar_policy(gtk::PolicyType::Automatic);
  scroll.set_size_request(280, 300);
  scroll.set_vexpand(true);
  let list = gtk::ListBox::new();
  list.set_hexpand(true);
  list.set_selection_mode(gtk::SelectionMode::None);
  let mut rows: Vec<(gtk::ListBoxRow, String)> = Vec::new();
  for item in items.iter() {
    let label = markup_label(item, 13, "normal", pal.fg);
    label.set_halign(gtk::Align::Start);
    label.set_xalign(0.0);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let row = gtk::ListBoxRow::new();
    row.set_child(Some(&label));
    list.append(&row);
    rows.push((row, item.to_lowercase()));
  }
  let rows = Rc::new(rows);
  scroll.set_child(Some(&list));
  body.append(&scroll);

  let rows_filter = Rc::clone(&rows);
  search.connect_search_changed(move |entry| {
    let query = entry.text().to_string().to_lowercase();
    for (row, text) in rows_filter.iter() {
      row.set_visible(query.is_empty() || text.contains(&query));
    }
  });

  let items_pick = Rc::clone(items);
  list.connect_row_activated(move |_, row| {
    if let Some(item) = items_pick.get(row.index() as usize).cloned() {
      on_pick(item);
    }
  });

  body
}

/// Menu button showing `current` text, opening `menu_body` in a popover.
fn menu_button(current: &str, pal_fg: &str, menu_body: &gtk::Box) -> gtk::MenuButton {
  let menu = gtk::MenuButton::new();
  menu.set_halign(gtk::Align::End);
  menu.set_valign(gtk::Align::Center);
  let label = markup_label(current, 13, "normal", pal_fg);
  label.set_ellipsize(gtk::pango::EllipsizeMode::End);
  label.set_max_width_chars(26);
  menu.set_child(Some(&label));
  let popover = gtk::Popover::new();
  popover.set_child(Some(menu_body));
  menu.set_popover(Some(&popover));
  menu
}

/// Keyboard menu context: shared across the layout and variant views.
struct KbCtx {
  stack: gtk::Box,
  keymaps: Rc<Vec<String>>,
  current_layout: String,
  current_variant: Option<String>,
  fg: &'static str,
  secondary: &'static str,
  kb_error: gtk::Label,
  refresh: Arc<AtomicBool>,
}

fn clear_box(target: &gtk::Box) {
  while let Some(child) = target.first_child() {
    target.remove(&child);
  }
}

/// Searchable row list inside `target`; `on_pick` receives the row index.
fn searchable_list(
  target: &gtk::Box,
  entries: &[(String, bool)],
  fg: &'static str,
  secondary: &'static str,
  on_pick: Rc<dyn Fn(usize)>,
) {
  let search = gtk::SearchEntry::new();
  search.set_hexpand(true);
  search.set_placeholder_text(Some(&lang::t("sidebar.search")));
  target.append(&search);

  let scroll = gtk::ScrolledWindow::new();
  scroll.set_hscrollbar_policy(gtk::PolicyType::Never);
  scroll.set_vscrollbar_policy(gtk::PolicyType::Automatic);
  scroll.set_vexpand(true);
  let list = gtk::ListBox::new();
  list.set_hexpand(true);
  list.set_selection_mode(gtk::SelectionMode::None);
  let mut rows: Vec<(gtk::ListBoxRow, String)> = Vec::new();
  for (text, current) in entries {
    let label = markup_label(text, 13, "normal", fg);
    label.set_halign(gtk::Align::Start);
    label.set_xalign(0.0);
    label.set_hexpand(true);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let row = gtk::ListBoxRow::new();
    let inner = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    inner.set_hexpand(true);
    inner.append(&label);
    if *current {
      let check = markup_label("✓", 13, "bold", secondary);
      check.set_halign(gtk::Align::End);
      check.set_valign(gtk::Align::Center);
      inner.append(&check);
    }
    row.set_child(Some(&inner));
    list.append(&row);
    rows.push((row, text.to_lowercase()));
  }
  let rows = Rc::new(rows);
  scroll.set_child(Some(&list));
  target.append(&scroll);

  let rows_filter = Rc::clone(&rows);
  search.connect_search_changed(move |entry| {
    let query = entry.text().to_string().to_lowercase();
    for (row, text) in rows_filter.iter() {
      row.set_visible(query.is_empty() || text.contains(&query));
    }
  });

  list.connect_row_activated(move |_, row| {
    on_pick(row.index() as usize);
  });
}

/// Apply a layout/variant through the daemon; failures show in the card.
fn apply_variant(ctx: &Rc<KbCtx>, layout: &str, variant: Option<&str>) {
  match daemon::locale_set_keymap(layout, variant) {
    Ok(_) => {
      ctx.kb_error.set_visible(false);
      ctx.refresh.store(true, Ordering::SeqCst);
    }
    Err(e) => {
      ctx.kb_error.set_markup(&span(
        &format!("{} ({})", lang::t("locale.failed"), e),
        12,
        "normal",
        "#FF453A",
      ));
      ctx.kb_error.set_visible(true);
    }
  }
}

/// Layout list view.
fn show_layout_list(ctx: &Rc<KbCtx>) {
  clear_box(&ctx.stack);
  let keymaps = ctx.keymaps.clone();
  let current = ctx.current_layout.clone();
  let entries: Vec<(String, bool)> = keymaps
    .iter()
    .map(|layout| (layout.clone(), layout == &current))
    .collect();
  let ctx_pick = Rc::clone(ctx);
  searchable_list(
    &ctx.stack,
    &entries,
    ctx.fg,
    ctx.secondary,
    Rc::new(move |index: usize| {
      let layout = keymaps.get(index).cloned().unwrap_or_default();
      if layout.is_empty() {
        return;
      }
      let variants = daemon::locale_keymap_variants(&layout).unwrap_or_default();
      if variants.is_empty() {
        apply_variant(&ctx_pick, &layout, None);
      } else {
        show_variant_list(&ctx_pick, &layout, &variants);
      }
    }),
  );
}

/// Variant list view for one layout: Back plus Default plus variants.
fn show_variant_list(ctx: &Rc<KbCtx>, layout: &str, variants: &[String]) {
  clear_box(&ctx.stack);
  let back = gtk::ListBoxRow::new();
  let back_label = markup_label("‹ ", 15, "bold", ctx.secondary);
  back_label.set_halign(gtk::Align::Start);
  back.set_child(Some(&back_label));
  let back_list = gtk::ListBox::new();
  back_list.set_selection_mode(gtk::SelectionMode::None);
  back_list.append(&back);
  ctx.stack.append(&back_list);
  let ctx_back = Rc::clone(ctx);
  back_list.connect_row_activated(move |_, _| {
    show_layout_list(&ctx_back);
  });

  let current_variant = ctx.current_variant.clone();
  let mut entries: Vec<(String, bool)> = vec![(
    lang::t("locale.default_variant"),
    current_variant.is_none(),
  )];
  for variant in variants {
    entries.push((
      variant.clone(),
      current_variant.as_deref() == Some(variant),
    ));
  }
  let ctx_pick = Rc::clone(ctx);
  let layout_owned = layout.to_string();
  let variants_owned: Vec<String> = variants.to_vec();
  searchable_list(
    &ctx.stack,
    &entries,
    ctx.fg,
    ctx.secondary,
    Rc::new(move |index: usize| {
      if index == 0 {
        apply_variant(&ctx_pick, &layout_owned, None);
      } else if let Some(variant) = variants_owned.get(index - 1) {
        apply_variant(&ctx_pick, &layout_owned, Some(variant));
      }
    }),
  );
}

/// Keyboard menu body: layout list, then the variant list for the picked
/// layout (Default plus variants, with Back). Applies through the daemon.
fn keyboard_menu(
  keymaps: &Rc<Vec<String>>,
  current_layout: &str,
  current_variant: &Option<String>,
  pal: &Palette,
  kb_error: &gtk::Label,
  refresh_flag: &Arc<AtomicBool>,
) -> gtk::Box {
  let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
  body.set_hexpand(true);
  body.set_vexpand(true);
  body.set_size_request(296, 360);
  let stack = gtk::Box::new(gtk::Orientation::Vertical, 0);
  stack.set_hexpand(true);
  stack.set_vexpand(true);
  stack.set_margin_top(8);
  stack.set_margin_bottom(8);
  stack.set_margin_start(8);
  stack.set_margin_end(8);
  body.append(&stack);
  let ctx = Rc::new(KbCtx {
    stack,
    keymaps: Rc::clone(keymaps),
    current_layout: current_layout.to_string(),
    current_variant: current_variant.clone(),
    fg: pal.fg,
    secondary: pal.secondary,
    kb_error: kb_error.clone(),
    refresh: Arc::clone(refresh_flag),
  });
  show_layout_list(&ctx);
  body
}

/// Fill `detail` for the current daemon state. `refresh_flag` is set by
/// the Auto Detect toggle (whose handler must be Send + Sync);
/// `last_error` carries a failed save into the re-render.
fn render(detail: &gtk::Box, refresh_flag: &Arc<AtomicBool>, last_error: &Arc<Mutex<Option<String>>>) {
  while let Some(child) = detail.first_child() {
    detail.remove(&child);
  }
  let pal = palette(is_dark());
  let state = daemon::locale_get().unwrap_or_default();

  // System language: English/German rows with a checkmark, "More soon".
  detail.append(&section_label(&lang::t("locale.system"), pal.secondary));
  let lang_card = card(pal.card);
  for (index, language) in state.languages.iter().enumerate() {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.set_hexpand(true);
    row.set_valign(gtk::Align::Center);
    row.set_margin_top(5);
    row.set_margin_bottom(5);
    row.set_focusable(true);
    if let Some(cursor) = gtk::gdk::Cursor::from_name("pointer", None) {
      row.set_cursor(Some(&cursor));
    }
    if index + 1 != state.languages.len() {
      apply_css(
        &row,
        "box { border-bottom: 1px solid rgba(128,128,128,0.25); }",
      );
    }
    let name = markup_label(&language.name, 13, "normal", pal.fg);
    name.set_halign(gtk::Align::Start);
    name.set_xalign(0.0);
    name.set_hexpand(true);
    name.set_ellipsize(gtk::pango::EllipsizeMode::End);
    row.append(&name);
    if language.code == state.language {
      let check = markup_label("✓", 14, "bold", pal.fg);
      check.set_halign(gtk::Align::End);
      check.set_valign(gtk::Align::Center);
      row.append(&check);
    }
    let code = language.code.clone();
    let refresh_picked = Arc::clone(refresh_flag);
    let click = gtk::GestureClick::new();
    click.set_button(1);
    click.connect_released(move |_, _, _, _| {
      let _ = daemon::locale_set_language(&code);
      refresh_picked.store(true, Ordering::SeqCst);
    });
    row.add_controller(click);
    lang_card.append(&row);
  }
  detail.append(&lang_card);
  let soon = markup_label(&lang::t("locale.more_soon"), 12, "normal", pal.secondary);
  soon.set_halign(gtk::Align::Start);
  soon.set_xalign(0.0);
  detail.append(&soon);

  let gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap.set_size_request(-1, 12);
  detail.append(&gap);

  // Region picker with the full country list.
  detail.append(&section_label(&lang::t("locale.region"), pal.secondary));
  let region_card = card(pal.card);
  let region_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  region_row.set_hexpand(true);
  region_row.set_valign(gtk::Align::Center);
  region_row.set_margin_top(5);
  region_row.set_margin_bottom(5);
  let region_label = markup_label(&lang::t("locale.region"), 13, "normal", pal.fg);
  region_label.set_halign(gtk::Align::Start);
  region_label.set_xalign(0.0);
  region_label.set_hexpand(true);
  region_row.append(&region_label);
  let current_country = state
    .regions
    .iter()
    .find(|r| r.code == state.region)
    .map(|r| r.name.clone())
    .unwrap_or_else(|| state.region.clone());
  let region_names: Rc<Vec<String>> = Rc::new(state.regions.iter().map(|r| r.name.clone()).collect());
  let region_error = markup_label("", 12, "normal", "#FF453A");
  region_error.set_halign(gtk::Align::End);
  region_error.set_xalign(1.0);
  region_error.set_wrap(true);
  region_error.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  region_error.set_visible(false);
  let region_names_pick = Rc::clone(&region_names);
  let codes: Rc<Vec<daemon::RegionEntry>> = Rc::new(state.regions.clone());
  let region_error_pick = region_error.clone();
  let refresh_region = Arc::clone(refresh_flag);
  let region_menu = option_menu(
    &region_names,
    &pal,
    Rc::new(move |name: String| {
      let code = codes
        .iter()
        .find(|r| r.name == name)
        .map(|r| r.code.clone())
        .unwrap_or_default();
      if code.is_empty() {
        return;
      }
      match daemon::locale_set_region(&code) {
        Ok(_) => {
          region_error_pick.set_visible(false);
          refresh_region.store(true, Ordering::SeqCst);
        }
        Err(e) => {
          region_error_pick.set_markup(&span(
            &format!("{} ({})", lang::t("locale.failed"), e),
            12,
            "normal",
            "#FF453A",
          ));
          region_error_pick.set_visible(true);
        }
      }
    }),
  );
  let _ = region_names_pick;
  region_row.append(&menu_button(&current_country, pal.fg, &region_menu));
  region_card.append(&region_row);
  region_card.append(&region_error);
  detail.append(&region_card);

  let gap2 = gtk::Box::new(gtk::Orientation::Vertical, 0);
  gap2.set_size_request(-1, 12);
  detail.append(&gap2);

  // Keyboard: Auto Detect toggle plus the layout menu button.
  detail.append(&section_label(&lang::t("locale.keyboard"), pal.secondary));
  let kb_card = card(pal.card);
  let auto_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  auto_row.set_hexpand(true);
  auto_row.set_valign(gtk::Align::Center);
  auto_row.set_margin_top(5);
  auto_row.set_margin_bottom(5);
  let auto_label = markup_label(&lang::t("locale.auto_detect"), 13, "normal", pal.fg);
  auto_label.set_halign(gtk::Align::Start);
  auto_label.set_xalign(0.0);
  auto_label.set_hexpand(true);
  auto_row.append(&auto_label);
  let refresh_raised = Arc::clone(refresh_flag);
  let error_slot = Arc::clone(last_error);
  let auto_toggle = Toggle::new("")
    .value(state.auto_keymap)
    .width(52.0)
    .on_change(move |on| {
      let result = daemon::locale_set_auto_keymap(on).err();
      if let Ok(mut slot) = error_slot.lock() {
        *slot = result;
      }
      refresh_raised.store(true, Ordering::SeqCst);
    });
  let auto_gtk = auto_toggle.to_gtk();
  auto_gtk.set_halign(gtk::Align::End);
  auto_gtk.set_valign(gtk::Align::Center);
  auto_gtk.set_vexpand(false);
  auto_row.append(&auto_gtk);
  kb_card.append(&auto_row);

  let current_layout = if state.keymap.is_empty() {
    lang::t("locale.keyboard")
  } else {
    match &state.keymap_variant {
      Some(variant) => format!("{} ({})", state.keymap, variant),
      None => state.keymap.clone(),
    }
  };
  let layout_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  layout_row.set_hexpand(true);
  layout_row.set_valign(gtk::Align::Center);
  layout_row.set_margin_top(5);
  layout_row.set_margin_bottom(5);
  let layout_label = markup_label(&lang::t("locale.keyboard"), 13, "normal", pal.fg);
  layout_label.set_halign(gtk::Align::Start);
  layout_label.set_xalign(0.0);
  layout_label.set_hexpand(true);
  layout_row.append(&layout_label);
  let keymaps: Rc<Vec<String>> = Rc::new(state.keymaps.clone());
  let kb_error = markup_label("", 12, "normal", "#FF453A");
  kb_error.set_halign(gtk::Align::End);
  kb_error.set_xalign(1.0);
  kb_error.set_wrap(true);
  kb_error.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  kb_error.set_visible(false);
  layout_row.append(&menu_button(
    &current_layout,
    pal.fg,
    &keyboard_menu(&keymaps, &state.keymap, &state.keymap_variant, &pal, &kb_error, refresh_flag),
  ));
  kb_card.append(&layout_row);
  kb_card.append(&kb_error);
  if let Some(message) = last_error.lock().ok().and_then(|mut slot| slot.take()) {
    let auto_error = markup_label(&message, 12, "normal", "#FF453A");
    auto_error.set_halign(gtk::Align::End);
    auto_error.set_xalign(1.0);
    auto_error.set_wrap(true);
    auto_error.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    kb_card.append(&auto_error);
  }
  detail.append(&kb_card);
}

/// Small section header.
fn section_label(title: &str, secondary: &str) -> gtk::Widget {
  let section = markup_label(title, 12, "normal", secondary);
  section.set_halign(gtk::Align::Start);
  section.set_margin_bottom(2);
  section.upcast()
}

/// The Language & Region detail page, directly on the screen.
pub(crate) fn build_page() -> gtk::Widget {
  let detail = gtk::Box::new(gtk::Orientation::Vertical, 0);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  let refresh_flag = Arc::new(AtomicBool::new(false));
  let last_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

  // Refresh poller for toggle/menu changes; stops with the page.
  let weak = detail.downgrade();
  let raised = Arc::clone(&refresh_flag);
  let rendered = Arc::clone(&refresh_flag);
  let error_rendered = Arc::clone(&last_error);
  glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
    match weak.upgrade() {
      Some(detail) => {
        if raised.swap(false, Ordering::SeqCst) {
          render(&detail, &rendered, &error_rendered);
        }
        glib::ControlFlow::Continue
      }
      None => glib::ControlFlow::Break,
    }
  });

  render(&detail, &refresh_flag, &last_error);
  detail.upcast()
}
