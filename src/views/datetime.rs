//! Date & Time settings page for SystemSettings.
//!
//! Four bare cards like the macOS mockup: automatic toggle (locked on
//! for now), live date/time, 24-hour toggle and a real timezone
//! dropdown. State comes from the daemon (`timedatectl`); the zone list
//! never errors. All text uses SF Pro Display and both `en_us` and
//! `de_de` strings.

use super::{Palette, is_dark, markup_label, palette};
use crate::daemon;
use crate::lang;
use crate::TontooUI::Toggle;
use crate::UIKit::apply_css;
use crate::UIKit::prelude::*;
use gtk::prelude::*;
use std::cell::{Cell, RefCell};
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

/// "Sep 12, 2026 at 12:56:08 PM" (12h) or "Sep 12, 2026 at 13:56:08"
/// (24h), in local time. Empty when the clock is unreadable.
fn format_now(use_24h: bool) -> String {
  let now = match glib::DateTime::now_local() {
    Ok(now) => now,
    Err(_) => return String::new(),
  };
  let month = now.format("%b").map(|s| s.to_string()).unwrap_or_default();
  let date = format!("{} {}, {}", month, now.day_of_month(), now.year());
  let time = if use_24h {
    format!(
      "{:02}:{:02}:{:02}",
      now.hour(),
      now.minute(),
      now.seconds() as u32
    )
  } else {
    let hour = now.hour() % 12;
    let hour = if hour == 0 { 12 } else { hour };
    let meridiem = if now.hour() < 12 { "AM" } else { "PM" };
    format!(
      "{}:{:02}:{:02} {}",
      hour,
      now.minute(),
      now.seconds() as u32,
      meridiem
    )
  };
  format!("{} at {}", date, time)
}

/// Searchable timezone menu content: search field plus the filtered
/// zone list. Picking a row applies it through the daemon; failures
/// show in `tz_error` and keep the menu open for another try.
fn timezone_menu(
  zones: &Rc<Vec<String>>,
  current: &str,
  pal: &Palette,
  tz_error: &gtk::Label,
  refresh_flag: &Arc<AtomicBool>,
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
  search.set_placeholder_text(Some("Search"));
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
  for zone in zones.iter() {
    let label = markup_label(zone, 13, "normal", pal.fg);
    label.set_halign(gtk::Align::Start);
    label.set_xalign(0.0);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let row = gtk::ListBoxRow::new();
    row.set_child(Some(&label));
    if zone == current {
      row.add_css_class("tz-current");
      apply_css(
        &row,
        "row.tz-current { background-color: rgba(128,128,128,0.25); border-radius: 6px; }",
      );
    }
    list.append(&row);
    rows.push((row, zone.to_lowercase()));
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

  let zones_pick = Rc::clone(zones);
  let tz_error_pick = tz_error.clone();
  let refresh_applied = Arc::clone(refresh_flag);
  list.connect_row_activated(move |_, row| {
    let index = row.index() as usize;
    let zone = zones_pick.get(index).cloned().unwrap_or_default();
    if zone.is_empty() {
      return;
    }
    match daemon::datetime_set_timezone(&zone) {
      Ok(_) => {
        tz_error_pick.set_visible(false);
        refresh_applied.store(true, Ordering::SeqCst);
      }
      Err(e) => {
        tz_error_pick.set_markup(&span(
          &format!("{} ({})", lang::t("datetime.tz_failed"), e),
          12,
          "normal",
          "#FF453A",
        ));
        tz_error_pick.set_visible(true);
      }
    }
  });

  body
}

/// One card row: label left, control right.
fn card_row(label_key: &str, pal: &Palette) -> (gtk::Box, gtk::Label) {
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
  row.set_hexpand(true);
  row.set_valign(gtk::Align::Center);
  row.set_margin_top(5);
  row.set_margin_bottom(5);
  let label = markup_label(&lang::t(label_key), 13, "normal", pal.fg);
  label.set_halign(gtk::Align::Start);
  label.set_xalign(0.0);
  label.set_hexpand(true);
  label.set_ellipsize(gtk::pango::EllipsizeMode::End);
  row.append(&label);
  (row, label)
}

/// Fill `detail` for the current daemon state. `refresh_flag` is set by
/// the 24-hour toggle (whose handler must be Send + Sync); the poller in
/// `build_page` picks it up and re-renders here. `clock`/`clock_24h`
/// point the 1-second ticker at the fresh date/time label. `last_error`
/// carries a failed 24-hour save into the re-render so it shows instead
/// of silently snapping back.
fn render(
  detail: &gtk::Box,
  refresh_flag: &Arc<AtomicBool>,
  clock: &Rc<RefCell<Option<glib::WeakRef<gtk::Label>>>>,
  clock_24h: &Rc<Cell<bool>>,
  last_error: &Arc<Mutex<Option<String>>>,
) {
  while let Some(child) = detail.first_child() {
    detail.remove(&child);
  }
  let pal = palette(is_dark());
  let state = daemon::datetime_get().unwrap_or_default();
  clock_24h.set(state.use_24h);

  // Automatic toggle: always on, not changeable. The daemon enforces
  // NTP at startup; the UI offers no way to turn it off.
  let auto_card = card(pal.card);
  let (auto_row, _) = card_row("datetime.auto", &pal);
  let auto_toggle = Toggle::new("").value(true).width(52.0);
  let auto_gtk = auto_toggle.to_gtk();
  auto_gtk.set_halign(gtk::Align::End);
  auto_gtk.set_valign(gtk::Align::Center);
  auto_gtk.set_vexpand(false);
  auto_gtk.set_sensitive(false);
  auto_row.append(&auto_gtk);
  auto_card.append(&auto_row);
  detail.append(&auto_card);

  // Live date and time.
  let time_card = card(pal.card);
  let (time_row, _) = card_row("datetime.datetime", &pal);
  let time_value = markup_label(&format_now(state.use_24h), 13, "normal", pal.secondary);
  time_value.set_halign(gtk::Align::End);
  time_value.set_xalign(1.0);
  time_row.append(&time_value);
  time_card.append(&time_row);
  detail.append(&time_card);
  *clock.borrow_mut() = Some(time_value.downgrade());

  // 24-hour toggle: applies through the daemon, then re-renders.
  // Failures surface in the card instead of silently snapping back.
  let day_card = card(pal.card);
  let (day_row, _) = card_row("datetime.use_24h", &pal);
  let refresh_raised = Arc::clone(refresh_flag);
  let error_slot = Arc::clone(last_error);
  let day_toggle = Toggle::new("")
    .value(state.use_24h)
    .width(52.0)
    .on_change(move |on| {
      let result = daemon::datetime_set_24h(on).err();
      if let Ok(mut slot) = error_slot.lock() {
        *slot = result;
      }
      refresh_raised.store(true, Ordering::SeqCst);
    });
  let day_gtk = day_toggle.to_gtk();
  day_gtk.set_halign(gtk::Align::End);
  day_gtk.set_valign(gtk::Align::Center);
  day_gtk.set_vexpand(false);
  day_row.append(&day_gtk);
  day_card.append(&day_row);
  if let Some(message) = last_error.lock().ok().and_then(|mut slot| slot.take()) {
    let day_error = markup_label(&message, 12, "normal", "#FF453A");
    day_error.set_halign(gtk::Align::End);
    day_error.set_xalign(1.0);
    day_error.set_wrap(true);
    day_error.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    day_card.append(&day_error);
  }
  detail.append(&day_card);

  // Timezone menu button with the real zone list.
  let tz_card = card(pal.card);
  let (tz_row, _) = card_row("datetime.timezone", &pal);
  let zones: Rc<Vec<String>> = Rc::new(state.timezones.clone());
  let menu = gtk::MenuButton::new();
  menu.set_halign(gtk::Align::End);
  menu.set_valign(gtk::Align::Center);
  let menu_label = markup_label(&state.timezone, 13, "normal", pal.fg);
  menu_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
  menu_label.set_max_width_chars(26);
  menu.set_child(Some(&menu_label));
  let tz_error = markup_label("", 12, "normal", "#FF453A");
  tz_error.set_halign(gtk::Align::End);
  tz_error.set_xalign(1.0);
  tz_error.set_wrap(true);
  tz_error.set_wrap_mode(gtk::pango::WrapMode::WordChar);
  tz_error.set_visible(false);
  let popover = gtk::Popover::new();
  popover.set_child(Some(&timezone_menu(
    &zones,
    &state.timezone,
    &pal,
    &tz_error,
    refresh_flag,
  )));
  menu.set_popover(Some(&popover));
  tz_row.append(&menu);
  tz_card.append(&tz_row);
  tz_card.append(&tz_error);
  detail.append(&tz_card);
}

/// The Date & Time detail page: four bare cards, directly on the screen.
pub(crate) fn build_page() -> gtk::Widget {
  let detail = gtk::Box::new(gtk::Orientation::Vertical, 8);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  let refresh_flag = Arc::new(AtomicBool::new(false));
  let last_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
  let clock: Rc<RefCell<Option<glib::WeakRef<gtk::Label>>>> = Rc::new(RefCell::new(None));
  let clock_24h: Rc<Cell<bool>> = Rc::new(Cell::new(false));

  // Refresh poller for toggle/menu changes; stops with the page.
  let weak = detail.downgrade();
  let raised = Arc::clone(&refresh_flag);
  let rendered = Arc::clone(&refresh_flag);
  let error_rendered = Arc::clone(&last_error);
  let clock_render = Rc::clone(&clock);
  let clock_24h_render = Rc::clone(&clock_24h);
  glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
    match weak.upgrade() {
      Some(detail) => {
        if raised.swap(false, Ordering::SeqCst) {
          render(&detail, &rendered, &clock_render, &clock_24h_render, &error_rendered);
        }
        glib::ControlFlow::Continue
      }
      None => glib::ControlFlow::Break,
    }
  });

  // 1-second ticker for the date/time label; stops with the page.
  let weak_tick = detail.downgrade();
  let clock_tick = Rc::clone(&clock);
  let clock_24h_tick = Rc::clone(&clock_24h);
  let secondary = palette(is_dark()).secondary;
  glib::timeout_add_local(std::time::Duration::from_secs(1), move || {
    if weak_tick.upgrade().is_none() {
      return glib::ControlFlow::Break;
    }
    if let Some(label) = clock_tick
      .borrow()
      .as_ref()
      .and_then(|weak| weak.upgrade())
    {
      label.set_markup(&span(&format_now(clock_24h_tick.get()), 13, "normal", secondary));
    }
    glib::ControlFlow::Continue
  });

  render(&detail, &refresh_flag, &clock, &clock_24h, &last_error);
  detail.upcast()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clock_formats_both_modes() {
    let twelve = format_now(false);
    assert!(twelve.contains(" at "));
    assert!(twelve.contains("AM") || twelve.contains("PM"));
    let twenty_four = format_now(true);
    assert!(twenty_four.contains(" at "));
    assert!(!twenty_four.contains("AM") && !twenty_four.contains("PM"));
  }
}
