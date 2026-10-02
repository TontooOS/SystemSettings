//! Modal sheets for SystemSettings.
//!
//! One sheet host owns every modal the app shows, so the app only has to
//! draw, route input to and drain a single overlay. `open` swaps in
//! freshly built content for the requested [`Kind`]; its buttons report
//! through [`Action`], which the app drains once per frame and applies to
//! the settings daemon.
//!
//! Searchable long lists (regions, keyboard layouts, timezones) do not
//! live here: they use the native `FormRow::picker` dropdown inside the
//! page instead.

use std::cell::RefCell;
use std::rc::Rc;

use crate::daemon;
use crate::lang;
use crate::views::Skin;
use crate::TontooUI::elements::{
  Align, BasicSheet, BasicText, Button, ButtonStyle, FileImage, HStack, SecureField, SegmentedPicker,
  SheetSize, Spacer, TextAlignment, VStack, View,
};
use crate::TontooUI::renderer::window::{Key, Viewport};
use crate::TontooUI::renderer::{FontSystem, ImageLoader};
use crate::TontooUI::Scene;

/// Wallpaper preview width inside the sheet in logical px.
pub(crate) const PREVIEW_W: f32 = 300.0;
/// Wallpaper preview height inside the sheet in logical px.
pub(crate) const PREVIEW_H: f32 = 169.0;
/// Text wrap width inside the sheets in logical px.
const TEXT_W: f32 = 300.0;

/// What the open sheet is asking for.
#[derive(Clone)]
pub(crate) enum Kind {
  /// Join a scanned network. The typed password is empty for open
  /// networks.
  Join { ssid: String, secured: bool },
  /// Apply a premade wallpaper; `variant` indexes
  /// `wallpaper::PREVIEW_ORDER` and starts on Auto.
  Wallpaper { entry: daemon::WallpaperEntry, variant: usize },
  /// Plain notice with a single dismiss button.
  Notice { title: String, message: String },
}

/// What the user did in the sheet.
pub(crate) enum Action {
  /// Confirm a network join with the typed password.
  Join { ssid: String, password: String },
  /// Confirm a wallpaper variant with the daemon kind and id.
  Wallpaper { kind: String, id: String, variant: String },
  /// Dismissed without applying anything.
  Dismiss,
}

/// Buttons report through these two flags.
#[derive(Clone)]
struct Flags {
  cancel: Rc<std::cell::Cell<bool>>,
  submit: Rc<std::cell::Cell<bool>>,
  password: Rc<RefCell<String>>,
  variant: Rc<std::cell::Cell<usize>>,
}

impl Flags {
  fn new() -> Self {
    Self {
      cancel: Rc::new(std::cell::Cell::new(false)),
      submit: Rc::new(std::cell::Cell::new(false)),
      password: Rc::new(RefCell::new(String::new())),
      variant: Rc::new(std::cell::Cell::new(1)),
    }
  }
}

/// One modal sheet plus the state its content needs.
pub(crate) struct Sheets {
  sheet: BasicSheet<VStack>,
  flags: Flags,
  kind: Option<Kind>,
  /// Variant the wallpaper preview currently shows, so a segmented
  /// change swaps the image.
  shown_variant: usize,
  action: Option<Action>,
}

impl Sheets {
  pub(crate) fn new() -> Self {
    Self {
      sheet: BasicSheet::new(VStack::new()).size(SheetSize::Quarter),
      flags: Flags::new(),
      kind: None,
      shown_variant: 0,
      action: None,
    }
  }

  /// True while a sheet is on screen.
  pub(crate) fn is_open(&self) -> bool {
    self.kind.is_some()
  }

  /// Take the action the sheet produced since the last drain.
  pub(crate) fn take_action(&mut self) -> Option<Action> {
    self.action.take()
  }

  /// Show a sheet, rebuilding its content for `kind`.
  pub(crate) fn open(&mut self, kind: Kind, skin: &Skin) {
    self.action = None;
    self.flags = Flags::new();
    let flags = self.flags.clone();
    self.kind = Some(kind.clone());
    self.shown_variant = match &kind {
      Kind::Wallpaper { variant, .. } => *variant,
      _ => 0,
    };
    let content = match kind {
      Kind::Join { ssid, secured } => {
        let mut column = VStack::new()
          .spacing(12.0)
          .align(Align::Leading)
          .child(
            BasicText::new(ssid)
              .size(15.0)
              .weight(600.0)
              .width(TEXT_W)
              .alignment(TextAlignment::Leading),
          );
        if secured {
          let captured = flags.password.clone();
          column = column.child(
            SecureField::new(lang::t("wifi.join.password")).on_change(move |text| {
              *captured.borrow_mut() = text.to_string();
            }),
          );
        }
        column
      }
      Kind::Wallpaper { entry, variant } => {
        flags.variant.set(variant);
        let options: Vec<String> = crate::views::wallpaper::PREVIEW_ORDER
          .iter()
          .map(|id| lang::t(&crate::views::wallpaper::mode_key(id)))
          .collect();
        let selected = flags.variant.clone();
        VStack::new()
          .spacing(14.0)
          .align(Align::Leading)
          .child(
            BasicText::new(entry.name.clone())
              .size(15.0)
              .weight(600.0)
              .width(PREVIEW_W)
              .alignment(TextAlignment::Leading),
          )
          .child(preview_image(&entry, variant))
          .child(SegmentedPicker::new("", options).selected(variant).on_select(move |index| {
            selected.set(index);
          }))
      }
      Kind::Notice { title, message } => VStack::new()
        .spacing(12.0)
        .align(Align::Leading)
        .child(
          BasicText::new(title)
            .size(15.0)
            .weight(600.0)
            .width(TEXT_W)
            .alignment(TextAlignment::Leading),
        )
        .child(
          BasicText::new(message)
            .size(13.0)
            .width(TEXT_W)
            .alignment(TextAlignment::Leading),
        ),
    };
    self.sheet = BasicSheet::new(content.child(buttons(
      lang::t("wallpaper.cancel"),
      lang::t("wallpaper.set"),
      flags.cancel.clone(),
      flags.submit.clone(),
    )))
    .size(SheetSize::Half);
    self.sheet.set_background(skin.bg);
    self.sheet.set_theme(skin.dark);
    self.sheet.show();
  }

  /// Dismiss the sheet without applying anything.
  pub(crate) fn close(&mut self) {
    self.kind = None;
    self.sheet.dismiss();
  }

  /// Translate the button flags into one [`Action`], keeping the
  /// wallpaper preview in sync with the segmented picker.
  pub(crate) fn pump(&mut self) {
    let Some(kind) = self.kind.clone() else {
      return;
    };
    if let Kind::Wallpaper { entry, .. } = &kind {
      let variant = self.flags.variant.get();
      if variant != self.shown_variant {
        self.shown_variant = variant;
        if let Some(stack) = self.sheet.child_mut().child_mut::<VStack>(0) {
          if let Some(image) = stack.child_mut::<FileImage>(1) {
            image.set_path(
              crate::views::wallpaper::preview_file(
                entry,
                crate::views::wallpaper::PREVIEW_ORDER
                  .get(variant)
                  .copied()
                  .unwrap_or("auto"),
              )
              .unwrap_or_default(),
            );
          }
        }
      }
    }
    if self.flags.cancel.get() {
      self.close();
      return;
    }
    if !self.flags.submit.get() {
      return;
    }
    self.action = Some(match kind {
      Kind::Join { ssid, .. } => Action::Join {
        ssid,
        password: self.flags.password.borrow().clone(),
      },
      Kind::Wallpaper { entry, .. } => Action::Wallpaper {
        kind: entry.kind.clone(),
        id: entry.id.clone(),
        variant: crate::views::wallpaper::PREVIEW_ORDER
          .get(self.flags.variant.get())
          .copied()
          .unwrap_or("auto")
          .to_string(),
      },
      Kind::Notice { .. } => Action::Dismiss,
    });
    self.close();
  }

  /// Sheets always need the blur pass for their glass chrome.
  pub(crate) fn wants_backdrop(&self) -> bool {
    self.sheet.is_visible()
  }

  /// True while the password field wants the I-beam cursor.
  pub(crate) fn wants_text_cursor(&mut self) -> bool {
    each_field(self.sheet.child_mut(), |field| field.wants_text_cursor())
  }

  pub(crate) fn draw(
    &mut self,
    scene: &mut Scene,
    fonts: &mut FontSystem,
    images: &mut ImageLoader<'_>,
    viewport: Viewport,
  ) {
    if !self.sheet.is_visible() {
      return;
    }
    self.sheet.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
    self.sheet.draw(scene, fonts, images);
  }

  pub(crate) fn mouse_down(&mut self, x: f64, y: f64) {
    self.sheet.mouse_down(x, y);
  }

  pub(crate) fn mouse_up(&mut self, x: f64, y: f64) {
    self.sheet.mouse_up(x, y);
  }

  pub(crate) fn hover(&mut self, x: f64, y: f64) {
    if let Some(stack) = self.sheet.child_mut().child_mut::<VStack>(0) {
      for index in 0..stack.len() {
        if let Some(picker) = stack.child_mut::<SegmentedPicker>(index) {
          picker.mouse_move(x, y);
        }
      }
    }
  }

  pub(crate) fn type_text(&mut self, text: &str) {
    each_field_mut(self.sheet.child_mut(), |field| {
      field.type_text(text);
      false
    });
  }

  pub(crate) fn key(&mut self, key: Key) -> bool {
    if each_field_mut(self.sheet.child_mut(), |field| field.key(key)) {
      return true;
    }
    if key == Key::Enter {
      self.flags.submit.set(true);
      return true;
    }
    if key == Key::Escape {
      self.flags.cancel.set(true);
      return true;
    }
    false
  }
}

/// Visit every `SecureField` directly inside the sheet column.
fn each_field(body: &mut VStack, mut f: impl FnMut(&SecureField) -> bool) -> bool {
  for index in 0..body.len() {
    if let Some(field) = body.child_mut::<SecureField>(index) {
      if f(field) {
        return true;
      }
    }
  }
  false
}

/// Mutate every `SecureField` directly inside the sheet column, stopping
/// at the first field that consumes the event.
fn each_field_mut(body: &mut VStack, mut f: impl FnMut(&mut SecureField) -> bool) -> bool {
  for index in 0..body.len() {
    if let Some(field) = body.child_mut::<SecureField>(index) {
      if f(field) {
        return true;
      }
    }
  }
  false
}

/// Cancel plus a trailing confirm button.
fn buttons(
  cancel: String,
  confirm: String,
  cancel_flag: Rc<std::cell::Cell<bool>>,
  confirm_flag: Rc<std::cell::Cell<bool>>,
) -> HStack {
  HStack::new()
    .spacing(10.0)
    .align(Align::Trailing)
    .child(Spacer::new().factor(1.0))
    .child(Button::new(cancel).style(ButtonStyle::Bordered).on_press(move || {
      cancel_flag.set(true);
    }))
    .child(Button::new(confirm).style(ButtonStyle::BorderedProminent).on_press(move || {
      confirm_flag.set(true);
    }))
}

/// Wallpaper preview for one variant, falling back to the placeholder box
/// when the file is missing.
fn preview_image(entry: &daemon::WallpaperEntry, variant: usize) -> FileImage {
  let path = crate::views::wallpaper::preview_file(
    entry,
    crate::views::wallpaper::PREVIEW_ORDER
      .get(variant)
      .copied()
      .unwrap_or("auto"),
  )
  .unwrap_or_default();
  FileImage::new(path, PREVIEW_W, PREVIEW_H).radius(12.0)
}
