//! Language & Region detail page for SystemSettings, pushed from the
//! General Language & Region row.
//!
//! System language list (English/German with a checkmark on the active
//! one plus the localized "more soon" note), a region dropdown with the
//! full country list and a keyboard group with the Auto Detect switch,
//! the layout dropdown and the variant dropdown of the picked layout.
//! Everything applies system-wide through the daemon (`localectl`).

use crate::daemon;
use crate::lang;
use crate::views::{caption, detail_header, note, Nav, PageView, Skin, BLOCK_GAP, LOCALE_HIDDEN};
use crate::TontooUI::elements::{
  Align, BasicOutlineGroup, Form, FormRow, FormSection, OutlineNode, VStack,
};

/// One entry of the system language list.
fn language_nodes(state: &daemon::LocaleState) -> Vec<OutlineNode> {
  state
    .languages
    .iter()
    .map(|entry| {
      let icon = if entry.code == state.language {
        "checkmark"
      } else {
        "globe"
      };
      OutlineNode::file(entry.name.clone()).icon(icon)
    })
    .collect()
}

/// Tappable system language list. Picking a row applies it system-wide.
fn language_list(nav: Nav) -> BasicOutlineGroup {
  let state = daemon::locale_get().unwrap_or_default();
  let codes: Vec<String> = state
    .languages
    .iter()
    .map(|entry| entry.code.clone())
    .collect();
  BasicOutlineGroup::new(language_nodes(&state))
    .selectable(true)
    .trailing_chevron(false)
    .on_select(move |path| {
      let Some(&index) = path.first() else {
        return;
      };
      let Some(code) = codes.get(index) else {
        return;
      };
      if let Err(err) = daemon::locale_set_language(code) {
        println!("Language & Region language save failed: {err}");
      }
      nav.touch();
    })
}

/// Region section: the full country list as a dropdown.
fn region_section() -> FormSection {
  let state = daemon::locale_get().unwrap_or_default();
  let names: Vec<String> = state.regions.iter().map(|entry| entry.name.clone()).collect();
  let codes: Vec<String> = state.regions.iter().map(|entry| entry.code.clone()).collect();
  let selected = state
    .regions
    .iter()
    .position(|entry| entry.code == state.region)
    .unwrap_or(0);
  if names.is_empty() {
    return FormSection::titled(lang::t("locale.region"))
      .footnote(lang::t("locale.failed"));
  }
  let row = FormRow::picker(lang::t("locale.region"), names, selected).on_pick(move |index| {
    let Some(code) = codes.get(index) else {
      return;
    };
    if let Err(err) = daemon::locale_set_region(code) {
      println!("Language & Region region save failed: {err}");
    }
  });
  FormSection::titled(lang::t("locale.region")).row(row)
}

/// Keyboard section: Auto Detect switch, the layout dropdown and, when
/// the active layout has variants, the variant dropdown. Picking a layout
/// clears the variant and asks for a rebuild so the variant list follows.
fn keyboard_section(nav: Nav) -> FormSection {
  let state = daemon::locale_get().unwrap_or_default();
  let keymaps = state.keymaps.clone();
  let mut section = FormSection::titled(lang::t("locale.keyboard")).row(
    FormRow::toggle(lang::t("locale.auto_detect"), state.auto_keymap).on_toggle(move |on| {
      if let Err(err) = daemon::locale_set_auto_keymap(on) {
        println!("Language & Region auto keymap save failed: {err}");
      }
    }),
  );

  if !keymaps.is_empty() {
    let layout_index = keymaps
      .iter()
      .position(|map| *map == state.keymap)
      .unwrap_or(0);
    let layouts = keymaps.clone();
    let rebuild = nav.clone();
    section = section.row(
      FormRow::picker(lang::t("locale.keyboard"), keymaps, layout_index).on_pick(move |index| {
        let Some(layout) = layouts.get(index) else {
          return;
        };
        if let Err(err) = daemon::locale_set_keymap(layout, None) {
          println!("Language & Region keymap save failed: {err}");
        }
        rebuild.touch();
      }),
    );

    if !state.keymap.is_empty() {
      let mut variants = vec![lang::t("locale.default_variant")];
      variants.extend(daemon::locale_keymap_variants(&state.keymap).unwrap_or_default());
      if variants.len() > 1 {
        let selected = if state.keymap_variant.is_some() { 1 } else { 0 };
        let layout = state.keymap.clone();
        let applied = variants.clone();
        section = section.row(
          FormRow::picker(lang::t("locale.variant"), variants, selected).on_pick(move |index| {
            let variant = if index == 0 {
              None
            } else {
              applied.get(index).cloned()
            };
            if let Err(err) = daemon::locale_set_keymap(&layout, variant.as_deref()) {
              println!("Language & Region variant save failed: {err}");
            }
          }),
        );
      }
    }
  }

  section
}

/// Build the Language & Region detail page.
pub(crate) fn build(_skin: &Skin, nav: &Nav) -> PageView {
  let body = VStack::new()
    .spacing(BLOCK_GAP)
    .align(Align::Leading)
    .child(caption(&lang::t("locale.system")))
    .child(language_list(nav.clone()))
    .child(note(&lang::t("locale.more_soon")))
    .child(
      Form::new()
        .section(region_section())
        .section(keyboard_section(nav.clone())),
    );

  crate::views::page_shell(
    detail_header(
      nav,
      "globe",
      &lang::t(crate::views::hidden_title(LOCALE_HIDDEN)),
    ),
    body,
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::daemon::{LanguageEntry, LocaleState, RegionEntry};

  fn state() -> LocaleState {
    LocaleState {
      language: "de".to_string(),
      languages: vec![
        LanguageEntry { code: "en".to_string(), name: "English".to_string() },
        LanguageEntry { code: "de".to_string(), name: "Deutsch".to_string() },
      ],
      region: "DE".to_string(),
      regions: vec![RegionEntry { code: "DE".to_string(), name: "Germany".to_string() }],
      keymap: "de".to_string(),
      keymap_variant: None,
      keymaps: vec!["de".to_string(), "us".to_string()],
      auto_keymap: true,
    }
  }

  #[test]
  fn active_language_gets_the_checkmark() {
    let state = state();
    let nodes = language_nodes(&state);
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].label(), "English");
    assert_eq!(nodes[1].label(), "Deutsch");
  }

  #[test]
  fn keyboard_section_reports_the_active_layout() {
    let state = state();
    assert_eq!(state.keymaps.iter().position(|m| *m == state.keymap), Some(0));
  }
}
