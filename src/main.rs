//! SystemSettings for TontooOS, built on the TontooUI renderer.
//!
//! One full-bleed window: the TontooUI `Sidebar` owns the navigation
//! column (traffic lights, search field, the avatar pill, one row per
//! settings category) plus the detail page of the selected row. Every
//! page is a `ScrollView` over a header row and native TontooUI
//! elements (`Form` groups, `BasicOutlineGroup` lists, `Slider`,
//! `SegmentedPicker`, `BasicSheet` dialogs), so the app follows the live
//! system color scheme through `ThemeWatcher` and only uses the
//! mandated TontooOS background/text tokens. Strings come from
//! `lang/en_us.json` and `lang/de_de.json` via `lang`.

mod app;
mod daemon;
mod lang;
mod views;

sdk::preinclude!();

use TontooUI::renderer::window::run;

fn main() {
  lang::init();
  if let Err(err) = run(
    &lang::t("app.title"),
    views::WINDOW_W,
    views::WINDOW_H,
    app::SettingsApp::new(),
  ) {
    eprintln!("systemsettings: {err}");
    std::process::exit(1);
  }
}
