//! General settings page for SystemSettings.
//!
//! A grouped navigation list built from the native
//! `BasicOutlineGroup`: every row is tappable, the About, Date & Time and
//! Language & Region rows push their hidden detail page on top of the
//! list (the sidebar title follows), everything else is display only.
//! The detail pages themselves live in [`crate::views::about`],
//! [`crate::views::datetime`] and [`crate::views::locale`].

use crate::lang;
use crate::views::{
  header_subtitle, Nav, PageView, Skin, ABOUT_HIDDEN, BLOCK_GAP, DATETIME_HIDDEN, GENERAL,
  LOCALE_HIDDEN,
};
use crate::TontooUI::elements::{Align, BasicOutlineGroup, OutlineNode, VStack};

/// One General row: label lang key and the SF Symbol shown next to it.
pub(crate) struct GeneralRow {
  pub key: &'static str,
  pub symbol: &'static str,
}

/// Rows in mockup order.
pub(crate) const ROWS: &[GeneralRow] = &[
  GeneralRow { key: "general.about", symbol: "questionmark" },
  GeneralRow { key: "general.software_update", symbol: "arrow.triangle.2.circlepath" },
  GeneralRow { key: "general.storage", symbol: "internaldrive.fill" },
  GeneralRow { key: "general.airdrop", symbol: "square.and.arrow.up" },
  GeneralRow { key: "general.datetime", symbol: "clock.fill" },
  GeneralRow { key: "general.language", symbol: "globe" },
  GeneralRow { key: "general.sharing", symbol: "person.2.circle.fill" },
  GeneralRow { key: "general.startup_disk", symbol: "internaldrive.fill" },
  GeneralRow { key: "general.device_management", symbol: "checkmark.seal.fill" },
  GeneralRow { key: "general.transfer_reset", symbol: "arrow.triangle.swap" },
];

/// Row group boundaries in mockup order: first 3 together, AirDrop
/// alone, the next 4 together, the last 2 alone.
pub(crate) const GROUPS: &[usize] = &[3, 1, 4, 2];

/// The hidden detail a row pushes, if any.
pub(crate) fn target_for(index: usize) -> Option<usize> {
  match index {
    0 => Some(ABOUT_HIDDEN),
    4 => Some(DATETIME_HIDDEN),
    5 => Some(LOCALE_HIDDEN),
    _ => None,
  }
}

/// Row boundaries of every group.
pub(crate) fn group_bounds() -> Vec<(usize, usize)> {
  let mut bounds = Vec::new();
  let mut start = 0;
  for size in GROUPS.iter().copied().chain(std::iter::once(usize::MAX)) {
    if start >= ROWS.len() {
      break;
    }
    let end = start.saturating_add(size).min(ROWS.len());
    bounds.push((start, end));
    start = end;
  }
  bounds
}

/// One tappable row group. Rows without a target are display only, so
/// the group is not selectable at all.
fn row_group(start: usize, end: usize, nav: Nav) -> BasicOutlineGroup {
  let nodes: Vec<OutlineNode> = ROWS[start..end]
    .iter()
    .map(|row| OutlineNode::file(lang::t(row.key)).icon(row.symbol))
    .collect();
  let targets: Vec<Option<usize>> = (start..end).map(target_for).collect();
  let navigable = targets.iter().any(|target| target.is_some());
  let group = BasicOutlineGroup::new(nodes)
    .selectable(navigable)
    .trailing_chevron(true);
  if !navigable {
    return group;
  }
  group.on_select(move |path| {
    let Some(&row) = path.first() else {
      return;
    };
    if let Some(Some(target)) = targets.get(row) {
      nav.push(*target);
    }
  })
}

/// Build the General page: either the navigation list or the pushed
/// hidden detail.
pub(crate) fn build(skin: &Skin, nav: &Nav) -> PageView {
  if nav.current().is_some() {
    return crate::views::build_hidden(nav.current().unwrap(), skin, nav);
  }

  let mut body = VStack::new().spacing(BLOCK_GAP).align(Align::Leading);
  for (start, end) in group_bounds() {
    body = body.child(row_group(start, end, nav.clone()));
  }

  crate::views::page_shell(
    crate::views::page_header(
      crate::views::header_symbol(GENERAL),
      &header_subtitle(GENERAL),
    ),
    body,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rows_match_mockup() {
    assert_eq!(ROWS.len(), 10);
    let keys: Vec<&str> = ROWS.iter().map(|row| row.key).collect();
    assert_eq!(keys[0], "general.about");
    assert_eq!(keys[3], "general.airdrop");
    assert_eq!(keys[4], "general.datetime");
    assert_eq!(keys[5], "general.language");
    assert_eq!(keys[9], "general.transfer_reset");
    for row in ROWS {
      assert!(row.key.starts_with("general."));
      assert!(!row.symbol.is_empty());
    }
  }

  #[test]
  fn groups_cover_all_rows() {
    assert_eq!(GROUPS, &[3, 1, 4, 2]);
    assert_eq!(GROUPS.iter().sum::<usize>(), ROWS.len());
    let bounds = group_bounds();
    assert_eq!(bounds.len(), GROUPS.len());
    assert_eq!(bounds[0], (0, 3));
    assert_eq!(bounds[1], (3, 4));
    assert_eq!(bounds[2], (4, 8));
    assert_eq!(bounds[3], (8, 10));
  }

  #[test]
  fn only_about_datetime_and_language_navigate() {
    let targets: Vec<Option<usize>> = (0..ROWS.len()).map(target_for).collect();
    assert_eq!(targets[0], Some(ABOUT_HIDDEN));
    assert_eq!(targets[4], Some(DATETIME_HIDDEN));
    assert_eq!(targets[5], Some(LOCALE_HIDDEN));
    let navigable = targets.iter().filter(|target| target.is_some()).count();
    assert_eq!(navigable, 3);
  }
}
