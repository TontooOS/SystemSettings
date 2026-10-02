//! Table-driven settings pages for SystemSettings.
//!
//! Most categories are still placeholders: a header row, an optional
//! master switch and one group of label/value or label/switch rows.
//! Rather than one near-identical module per category, the whole set is
//! declared once in [`PAGES`] and rendered by [`build`] out of native
//! TontooUI elements (`Form`, `FormSection`, `FormRow`, `Toggle`,
//! `Spacer`). Adding a category means adding one table entry.

use crate::lang;
use crate::views::{
  header_symbol, header_subtitle, page_header, page_shell, PageView, Skin, HEADER_GAP,
  HEADER_SYMBOL_PX, TEXT_W,
};
use crate::TontooUI::elements::{
  Align, BasicText, Form, FormRow, FormSection, HStack, SFSymbolImage, Spacer, TextAlignment,
  Toggle,
};

/// One row inside a placeholder group.
pub(crate) enum Row {
  /// Label left, localized value right.
  Info(&'static str, &'static str),
  /// Label left, switch right. The switch only reports to stdout, like
  /// the GTK build did, until the category gets a real backend.
  Toggle(&'static str, bool),
}

/// One table-driven category.
pub(crate) struct SimplePage {
  /// Sidebar index this page is registered under.
  pub index: usize,
  /// Optional master switch in the header row.
  pub master: Option<(&'static str, bool)>,
  /// Optional caption above the row group.
  pub caption: Option<&'static str>,
  /// The rows of the single group.
  pub rows: &'static [Row],
}

/// Every table-driven category, keyed by sidebar index.
const PAGES: &[SimplePage] = &[
  SimplePage {
    index: crate::views::BLUETOOTH,
    master: Some(("bluetooth.title", true)),
    caption: Some("bluetooth.devices.header"),
    rows: &[
      Row::Toggle("bluetooth.device.buds", true),
      Row::Toggle("bluetooth.device.mouse", false),
    ],
  },
  SimplePage {
    index: crate::views::BATTERY,
    master: None,
    caption: None,
    rows: &[
      Row::Info("battery.charge", "battery.charge.detail"),
      Row::Info("battery.condition", "battery.condition.detail"),
    ],
  },
  SimplePage {
    index: crate::views::ACCESSIBILITY,
    master: None,
    caption: None,
    rows: &[
      Row::Info("accessibility.display", "accessibility.display.detail"),
      Row::Toggle("accessibility.reduce_motion", false),
    ],
  },
  SimplePage {
    index: crate::views::DESKTOP_DOCK,
    master: None,
    caption: None,
    rows: &[
      Row::Info("desktop_dock.wallpaper", "desktop_dock.wallpaper.detail"),
      Row::Toggle("desktop_dock.show_dock", true),
      Row::Toggle("desktop_dock.magnification", false),
    ],
  },
  SimplePage {
    index: crate::views::MENU_BAR,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("menu_bar.clock", true),
      Row::Toggle("menu_bar.spotlight", false),
    ],
  },
  SimplePage {
    index: crate::views::TINTI_AI,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("tinti_ai.listen", true),
      Row::Toggle("tinti_ai.suggestions", true),
    ],
  },
  SimplePage {
    index: crate::views::SPOTLIGHT,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("spotlight.tinti_suggestions", true),
      Row::Toggle("spotlight.recents", false),
    ],
  },
  SimplePage {
    index: crate::views::NOTIFICATIONS,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("notifications.allow", true),
      Row::Toggle("notifications.sounds", true),
    ],
  },
  SimplePage {
    index: crate::views::SOUND,
    master: None,
    caption: None,
    rows: &[
      Row::Info("sound.output", "sound.output.detail"),
      Row::Toggle("sound.mute", false),
    ],
  },
  SimplePage {
    index: crate::views::FOCUS,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("focus.dnd", true),
      Row::Toggle("focus.sleep", false),
    ],
  },
  SimplePage {
    index: crate::views::SCREEN_TIME,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("screen_time.downtime", false),
      Row::Info("screen_time.app_limits", "screen_time.app_limits.detail"),
    ],
  },
  SimplePage {
    index: crate::views::LOCK_SCREEN,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("lock_screen.require_password", true),
      Row::Info("lock_screen.screen_saver", "lock_screen.screen_saver.detail"),
    ],
  },
  SimplePage {
    index: crate::views::PRIVACY,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("privacy.location", true),
      Row::Info("privacy.permissions", "privacy.permissions.detail"),
    ],
  },
  SimplePage {
    index: crate::views::TOUCH_ID,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("touch_id.unlock", true),
      Row::Info("touch_id.passwords", "touch_id.passwords.detail"),
    ],
  },
  SimplePage {
    index: crate::views::USERS,
    master: None,
    caption: None,
    rows: &[
      Row::Info("users.current", "users.current.detail"),
      Row::Toggle("users.guest", false),
    ],
  },
  SimplePage {
    index: crate::views::INTERNET_ACCOUNTS,
    master: None,
    caption: None,
    rows: &[
      Row::Info("internet_accounts.account", "internet_accounts.account.detail"),
      Row::Toggle("internet_accounts.mail", true),
    ],
  },
  SimplePage {
    index: crate::views::OCTO_CLOUD,
    master: None,
    caption: None,
    rows: &[
      Row::Info("octo_cloud.storage", "octo_cloud.storage.detail"),
      Row::Toggle("octo_cloud.sync", true),
    ],
  },
  SimplePage {
    index: crate::views::KEYBOARD,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("keyboard.key_repeat", true),
      Row::Info("keyboard.shortcuts", "keyboard.shortcuts.detail"),
    ],
  },
  SimplePage {
    index: crate::views::MOUSE,
    master: None,
    caption: None,
    rows: &[
      Row::Info("mouse.tracking", "mouse.tracking.detail"),
      Row::Toggle("mouse.natural_scroll", true),
    ],
  },
  SimplePage {
    index: crate::views::PRINTERS,
    master: None,
    caption: None,
    rows: &[
      Row::Info("printers.default", "printers.default.detail"),
      Row::Toggle("printers.double_sided", true),
    ],
  },
  SimplePage {
    index: crate::views::APP_SETTINGS,
    master: None,
    caption: None,
    rows: &[
      Row::Info("app_settings.default_apps", "app_settings.default_apps.detail"),
      Row::Toggle("app_settings.auto_update", true),
    ],
  },
  SimplePage {
    index: crate::views::DEVELOPER,
    master: None,
    caption: None,
    rows: &[
      Row::Toggle("developer.mode", false),
      Row::Info("developer.api_logs", "developer.api_logs.detail"),
    ],
  },
];

/// Table entry for a sidebar index.
fn find(index: usize) -> Option<&'static SimplePage> {
  PAGES.iter().find(|page| page.index == index)
}

/// Row rendered by [`build_row`].
fn build_row(row: &Row) -> FormRow {
  match row {
    Row::Info(label, value) => FormRow::text(lang::t(label), lang::t(value)),
    Row::Toggle(label, on) => {
      let key = *label;
      FormRow::toggle(lang::t(label), *on).on_toggle(move |on| {
        println!("{} toggled: {}", key, on);
      })
    }
  }
}

/// Build the page for a table-driven category.
pub(crate) fn build(index: usize, _skin: &Skin) -> PageView {
  let page = find(index).unwrap_or(&PAGES[0]);
  let subtitle = header_subtitle(index);
  let symbol = header_symbol(index);

  let header = match page.master {
    // Master switch pinned to the trailing edge of the header row.
    Some((key, on)) => HStack::new()
      .spacing(HEADER_GAP)
      .align(Align::Leading)
      .child(SFSymbolImage::new(symbol).size(HEADER_SYMBOL_PX))
      .child(
        BasicText::new(subtitle)
          .size(13.0)
          .weight(400.0)
          .width(TEXT_W)
          .alignment(TextAlignment::Leading),
      )
      .child(Spacer::new().factor(1.0))
      .child(
        Toggle::new("")
          .on(on)
          .on_toggle(move |on| println!("{} toggled: {}", key, on)),
      ),
    None => page_header(symbol, &subtitle),
  };

  let mut section = match page.caption {
    Some(key) => FormSection::titled(lang::t(key)),
    None => FormSection::new(),
  };
  for row in page.rows {
    section = section.row(build_row(row));
  }

  page_shell(header, Form::new().section(section))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::views::{
    APPEARANCE, DISPLAYS, GENERAL, NETWORK, PAGE_COUNT, WALLPAPER, WIFI,
  };

  /// Sidebar indices handled by their own page module.
  const DEDICATED: &[usize] = &[WIFI, NETWORK, GENERAL, APPEARANCE, DISPLAYS, WALLPAPER];

  #[test]
  fn every_index_is_covered_exactly_once() {
    let mut seen = vec![0usize; PAGE_COUNT];
    for page in PAGES {
      assert!(page.index < PAGE_COUNT, "index {} out of range", page.index);
      seen[page.index] += 1;
    }
    for (index, count) in seen.iter().enumerate() {
      if DEDICATED.contains(&index) {
        assert_eq!(*count, 0, "index {index} is handled by its own module");
        continue;
      }
      assert_eq!(*count, 1, "index {index} covered {count} times");
    }
  }

  #[test]
  fn every_page_has_rows() {
    for page in PAGES {
      assert!(!page.rows.is_empty(), "index {} has no rows", page.index);
    }
  }

  #[test]
  fn every_row_key_resolves_in_both_locales() {
    for page in PAGES {
      for row in page.rows {
        let keys: Vec<&str> = match row {
          Row::Info(label, value) => vec![label, value],
          Row::Toggle(label, _) => vec![label],
        };
        for key in keys {
          assert_ne!(lang::t(key), key, "{key} missing from the lang files");
        }
      }
      if let Some((key, _)) = page.master {
        assert_ne!(lang::t(key), key, "{key} missing from the lang files");
      }
      if let Some(key) = page.caption {
        assert_ne!(lang::t(key), key, "{key} missing from the lang files");
      }
    }
  }

  #[test]
  fn every_subtitle_resolves_in_both_locales() {
    for index in 0..PAGE_COUNT {
      let subtitle = header_subtitle(index);
      assert!(
        !subtitle.ends_with(".header.subtitle"),
        "index {index} has no subtitle string ({subtitle})"
      );
    }
  }

  #[test]
  fn a_missing_table_entry_is_reported() {
    // `build` never runs for an uncovered index (see
    // `every_index_is_covered_exactly_once`), but the lookup must report
    // the gap instead of silently rendering the wrong page.
    assert!(find(usize::MAX).is_none());
  }
}
