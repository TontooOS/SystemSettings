//! SystemSettings: TontooOS Settings basis built with TontooUI.
//!
//! Sidebar on the left (single Wi-Fi/WLAN category with a blue CoreIcon
//! SF Symbol), example Wi-Fi page on the right. Follows the live system
//! color scheme (Dark `#1d1d1d`, Light `#ececec`).

mod daemon;
mod lang;
mod views;

sdk::preinclude!();

use UIKit::prelude::*;

struct SettingsDelegate;

impl AppDelegate for SettingsDelegate {
  fn view(&self) -> Box<dyn Widget> {
    Box::new(views::root::SettingsRoot::new())
  }
}

fn main() {
  lang::init();
  let mut app = App::with_delegate(lang::t("app.title"), 900, 600, SettingsDelegate);
  app.auto_color_scheme();
  app.run();
}
