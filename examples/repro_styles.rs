//! Temporary reproduction: render the four Appearance icon styles.
use sdk::CoreIcon;

fn main() {
  let out = std::env::temp_dir().join("style_repro");
  std::fs::create_dir_all(&out).unwrap();
  let icon = format!("{}/Resources/app_icon.png", env!("CARGO_MANIFEST_DIR"));
  let green = CoreIcon::Color::from_hex("#34C759").unwrap();

  let cases: &[(&str, CoreIcon::generator::AppIcon)] = &[
    ("default", CoreIcon::generator::AppIcon::from_file(&icon).light()),
    ("dark", CoreIcon::generator::AppIcon::from_file(&icon).dark()),
    (
      "tinted_light",
      CoreIcon::generator::AppIcon::from_file(&icon).tint(green),
    ),
    (
      "tinted_dark",
      CoreIcon::generator::AppIcon::from_file(&icon)
        .dark()
        .tint(green),
    ),
  ];
  for (name, app_icon) in cases {
    let path = out.join(format!("{}.png", name));
    match app_icon.save(&path) {
      Ok(()) => println!("saved {}", path.display()),
      Err(e) => println!("FAILED {}: {}", name, e),
    }
  }
}
