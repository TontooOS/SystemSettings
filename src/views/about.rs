//! About detail page for SystemSettings (hidden page behind the
//! General About row, with back/forward history support).
//!
//! Device card (hostname, processor, memory, Linux kernel, installed app
//! count), TontooOS card (versioned OS logo with rounded corners,
//! display name, dynamic version) and a storage card (used/total).
//! All values read live with "Unknown" fallbacks; no buttons. All text
//! uses SF Pro Display and both `en_us` and `de_de` strings.

use super::{markup_label, palette, sidebar_style_icon_path};
use crate::daemon;
use crate::lang;
use gtk::prelude::*;

const DEVICE_ICON_PX: i32 = 64;
const LOGO_PX: i32 = 48;
const ROW_ICON_PX: i32 = 28;
const DEVICE_GRAY: (u8, u8, u8) = (142, 142, 147);

/// Bundled fallback OS logo.
pub(crate) fn bundled_logo() -> String {
  format!(
    "{}/Resources/app_icon.png",
    env!("CARGO_MANIFEST_DIR")
  )
}

/// Bundled device artwork (`Resources/laptop.png`).
pub(crate) fn laptop_png() -> String {
  format!(
    "{}/Resources/laptop.png",
    env!("CARGO_MANIFEST_DIR")
  )
}

/// OS logo for a version: CoreIcon version asset (`TontooOS_Icon.png`
/// for the matching version), else the bundled logo, else nothing.
pub(crate) fn os_logo_for_version(version: &str) -> Option<String> {
  let path = crate::CoreIcon::os_version::os_version_path(version, "TontooOS_Icon.png");
  if std::path::Path::new(&path).is_file() {
    return Some(path);
  }
  let bundled = bundled_logo();
  if std::path::Path::new(&bundled).is_file() {
    return Some(bundled);
  }
  None
}

/// Hostname from a `hostname(5)`-style file, empty when unreadable.
pub(crate) fn hostname_in(path: &std::path::Path) -> String {
  std::fs::read_to_string(path)
    .map(|content| content.trim().to_string())
    .unwrap_or_default()
}

/// First `model name` from cpuinfo, empty when missing.
pub(crate) fn processor_in(path: &std::path::Path) -> String {
  let content = match std::fs::read_to_string(path) {
    Ok(content) => content,
    Err(_) => return String::new(),
  };
  for line in content.lines() {
    if let Some((key, value)) = line.split_once(':') {
      if key.trim() == "model name" {
        return value.trim().to_string();
      }
    }
  }
  String::new()
}

/// Total RAM from meminfo as `16 GB` (binary, like macOS About),
/// empty when unreadable.
pub(crate) fn memory_in(path: &std::path::Path) -> String {
  let content = match std::fs::read_to_string(path) {
    Ok(content) => content,
    Err(_) => return String::new(),
  };
  for line in content.lines() {
    let mut parts = line.split_whitespace();
    if parts.next() == Some("MemTotal:") {
      if let Some(kb) = parts.next().and_then(|v| v.parse::<f64>().ok()) {
        return human_gib((kb * 1024.0) as u64);
      }
    }
  }
  String::new()
}

/// Kernel release, empty when unreadable.
pub(crate) fn kernel_in(path: &std::path::Path) -> String {
  std::fs::read_to_string(path)
    .map(|content| content.trim().to_string())
    .unwrap_or_default()
}

/// `512 GB`, `1 TB`: decimal (drive makers, macOS Finder), one decimal
/// below 10, none above.
pub(crate) fn human_bytes(bytes: u64) -> String {
  let tb = bytes as f64 / 1000.0_f64.powi(4);
  if tb >= 1.0 {
    return human_unit(tb, "TB");
  }
  human_unit(bytes as f64 / 1000.0_f64.powi(3), "GB")
}

/// `16 GB`: binary (macOS About memory style).
pub(crate) fn human_gib(bytes: u64) -> String {
  human_unit(bytes as f64 / 1024.0_f64.powi(3), "GB")
}

fn human_unit(value: f64, unit: &str) -> String {
  if value >= 10.0 {
    format!("{:.0} {}", value, unit)
  } else {
    let text = format!("{:.1}", value);
    format!("{} {}", text.trim_end_matches(".0"), unit)
  }
}

/// Installed `.app` bundles below the given directories.
pub(crate) fn apps_in(dirs: &[&std::path::Path]) -> usize {
  collect_apps(&dirs.iter().map(std::path::PathBuf::from).collect::<Vec<_>>()).len()
}

/// Per-user application folders (`<user>/Applications`) below a users
/// root (`/Users`), one per directory entry.
pub(crate) fn user_app_dirs(users_root: &std::path::Path) -> Vec<std::path::PathBuf> {
  let mut dirs = Vec::new();
  let entries = match std::fs::read_dir(users_root) {
    Ok(entries) => entries,
    Err(_) => return dirs,
  };
  for entry in entries.flatten() {
    let path = entry.path().join("Applications");
    if path.is_dir() {
      dirs.push(path);
    }
  }
  dirs.sort();
  dirs
}

/// Files and folders ending in `.app`, deduplicated by canonical path so
/// symlinked bundles (e.g. system links below `/Applications`) count once.
pub(crate) fn collect_apps(dirs: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
  let mut seen = std::collections::HashSet::new();
  for dir in dirs {
    let entries = match std::fs::read_dir(dir) {
      Ok(entries) => entries,
      Err(_) => continue,
    };
    for entry in entries.flatten() {
      let path = entry.path();
      let is_app = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(".app"))
        .unwrap_or(false);
      if !is_app {
        continue;
      }
      seen.insert(
        std::fs::canonicalize(&path).unwrap_or(path),
      );
    }
  }
  let mut out: Vec<std::path::PathBuf> = seen.into_iter().collect();
  out.sort();
  out
}

/// Live installed app count: `/Applications`, every user's
/// `~/Applications` below `/Users`, plus `/System/Applications`.
pub(crate) fn apps_live() -> usize {
  let mut dirs = vec![
    std::path::PathBuf::from("/Applications"),
    std::path::PathBuf::from("/System/Applications"),
  ];
  dirs.extend(user_app_dirs(std::path::Path::new("/Users")));
  collect_apps(&dirs).len()
}

/// `(source, used, total)` for `/` from `df -B1` output, if parseable.
pub(crate) fn storage_from_df(output: &str) -> Option<(String, String, String)> {
  for line in output.lines().skip(1) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
      continue;
    }
    let mount = parts[5];
    if mount != "/" {
      continue;
    }
    let total: u64 = parts[1].parse().ok()?;
    let used: u64 = parts[2].parse().ok()?;
    return Some((
      parts[0].to_string(),
      human_bytes(used),
      human_bytes(total),
    ));
  }
  None
}

/// Live root filesystem usage via `df`, if runnable and parseable.
pub(crate) fn storage_live() -> Option<(String, String, String)> {
  let output = std::process::Command::new("df")
    .arg("-B1")
    .arg("/")
    .output()
    .ok()?;
  if !output.status.success() {
    return None;
  }
  storage_from_df(&String::from_utf8_lossy(&output.stdout))
}

/// Rounded card container in the page palette color.
fn card(pal_card: &str) -> gtk::Box {
  let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
  card.set_hexpand(true);
  crate::UIKit::apply_css(
    &card,
    &format!(
      "box {{ background-color: {}; border-radius: 12px; padding: 12px 16px; }}",
      pal_card
    ),
  );
  card
}

/// Small secondary section label.
fn section_label(text: &str, pal_secondary: &str) -> gtk::Label {
  let label = markup_label(text, 13, "normal", pal_secondary);
  label.set_halign(gtk::Align::Start);
  label.set_xalign(0.0);
  label
}

/// Info row: label on the left, value on the right.
fn info_row(label: &str, value: &str, pal_fg: &str, pal_secondary: &str, last: bool) -> gtk::Box {
  let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
  row.set_hexpand(true);
  row.set_margin_top(5);
  row.set_margin_bottom(5);

  let name = markup_label(label, 13, "normal", pal_fg);
  name.set_halign(gtk::Align::Start);
  name.set_xalign(0.0);
  name.set_hexpand(true);
  name.set_ellipsize(gtk::pango::EllipsizeMode::End);
  row.append(&name);

  let detail = markup_label(value, 13, "normal", pal_secondary);
  detail.set_halign(gtk::Align::End);
  row.append(&detail);

  if !last {
    crate::UIKit::apply_css(
      &row,
      "box { border-bottom: 1px solid rgba(128,128,128,0.25); }",
    );
  }
  row
}

/// Fixed-size picture with rounded corners: the file is pre-scaled to
/// `px` (`GtkPicture` sizes from the texture and ignores size requests,
/// so the raw file would render at full texture size). Falls back to the
/// source file when caching fails.
fn fixed_picture(path: &str, px: i32, radius: i32, class: &str) -> Option<gtk::Picture> {
  let file = super::wallpaper::cached_thumb_fit(std::path::Path::new(path), px, px)
    .unwrap_or_else(|| std::path::PathBuf::from(path));
  let picture = gtk::Picture::for_filename(file);
  picture.set_content_fit(gtk::ContentFit::Cover);
  picture.set_hexpand(false);
  picture.set_vexpand(false);
  picture.set_can_shrink(true);
  picture.set_size_request(px, px);
  picture.add_css_class(class);
  crate::UIKit::apply_css(
    &picture,
    &format!(
      "picture.{} {{ border-radius: {}px; }}",
      class, radius
    ),
  );
  Some(picture)
}

/// Rounded OS logo picture, if resolvable.
fn logo_picture(version: &str) -> Option<gtk::Picture> {
  let path = os_logo_for_version(version)?;
  fixed_picture(&path, LOGO_PX, 24, "about-logo")
}

/// The About detail page (directly on the screen).
pub(crate) fn build_page() -> gtk::Widget {
  let pal = palette(super::is_dark());
  let fg: &'static str = pal.fg;
  let secondary: &'static str = pal.secondary;
  let unknown = lang::t("about.unknown");

  let detail = gtk::Box::new(gtk::Orientation::Vertical, 8);
  detail.set_hexpand(true);
  detail.set_vexpand(true);
  detail.set_margin_top(20);
  detail.set_margin_bottom(20);
  detail.set_margin_start(24);
  detail.set_margin_end(24);

  // Device header: laptop artwork plus hostname.
  let hostname = hostname_in(std::path::Path::new("/etc/hostname"));
  let header = gtk::Box::new(gtk::Orientation::Vertical, 8);
  header.set_hexpand(true);
  let laptop = laptop_png();
  if std::path::Path::new(&laptop).is_file() {
    if let Some(icon) = fixed_picture(&laptop, DEVICE_ICON_PX, 16, "about-device") {
      icon.set_halign(gtk::Align::Center);
      header.append(&icon);
    }
  } else if let Some(icon_path) =
    sidebar_style_icon_path("laptopcomputer", "about-device", DEVICE_GRAY)
  {
    if let Some(icon) = fixed_picture(&icon_path, DEVICE_ICON_PX, 16, "about-device") {
      icon.set_halign(gtk::Align::Center);
      header.append(&icon);
    }
  }
  let title = markup_label(
    if hostname.is_empty() {
      &unknown
    } else {
      &hostname
    },
    17,
    "bold",
    fg,
  );
  title.set_halign(gtk::Align::Center);
  title.set_xalign(0.5);
  header.append(&title);
  detail.append(&header);

  // Device card: name, chip, memory, kernel, apps.
  let processor = processor_in(std::path::Path::new("/proc/cpuinfo"));
  let memory = memory_in(std::path::Path::new("/proc/meminfo"));
  let kernel = kernel_in(std::path::Path::new("/proc/sys/kernel/osrelease"));
  let apps = apps_live();
  let device_card = card(pal.card);
  let device_rows = [
    (lang::t("about.name"), none_if_empty(&hostname, &unknown)),
    (lang::t("about.chip"), none_if_empty(&processor, &unknown)),
    (lang::t("about.memory"), none_if_empty(&memory, &unknown)),
    (lang::t("about.kernel"), none_if_empty(&kernel, &unknown)),
    (lang::t("about.apps"), apps.to_string()),
  ];
  for (i, (label, value)) in device_rows.iter().enumerate() {
    device_card.append(&info_row(label, value, fg, secondary, i + 1 == device_rows.len()));
  }
  detail.append(&device_card);

  // OS card: versioned logo, display name, dynamic version.
  let os = daemon::get_os().unwrap_or_default();
  detail.append(&section_label("TontooOS", secondary));
  let os_card = card(pal.card);
  let os_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  os_row.set_hexpand(true);
  os_row.set_valign(gtk::Align::Center);
  if let Some(logo) = logo_picture(&os.version) {
    logo.set_valign(gtk::Align::Center);
    os_row.append(&logo);
  }
  let os_name = markup_label(&os.display_name, 13, "normal", fg);
  os_name.set_halign(gtk::Align::Start);
  os_name.set_xalign(0.0);
  os_name.set_hexpand(true);
  os_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
  os_row.append(&os_name);
  let os_version = markup_label(
    &format!("{} {}", lang::t("about.version"), os.version),
    13,
    "normal",
    secondary,
  );
  os_version.set_halign(gtk::Align::End);
  os_row.append(&os_version);
  os_card.append(&os_row);
  detail.append(&os_card);

  // Storage card: device plus used/total, no buttons.
  detail.append(&section_label(&lang::t("about.storage"), secondary));
  let storage_card = card(pal.card);
  let storage_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
  storage_row.set_hexpand(true);
  storage_row.set_valign(gtk::Align::Center);
  if let Some(icon_path) =
    sidebar_style_icon_path("internaldrive.fill", "about-storage", DEVICE_GRAY)
  {
    if let Some(icon) = fixed_picture(&icon_path, ROW_ICON_PX, 8, "about-drive") {
      icon.set_valign(gtk::Align::Center);
      storage_row.append(&icon);
    }
  }
  let (device, usage) = match storage_live() {
    Some((source, used, total)) => (source, format!("{} / {}", used, total)),
    None => (String::new(), unknown.clone()),
  };
  let device_label = markup_label(&device, 13, "normal", fg);
  device_label.set_halign(gtk::Align::Start);
  device_label.set_xalign(0.0);
  device_label.set_hexpand(true);
  device_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
  storage_row.append(&device_label);
  let usage_label = markup_label(&usage, 13, "normal", secondary);
  usage_label.set_halign(gtk::Align::End);
  storage_row.append(&usage_label);
  storage_card.append(&storage_row);
  detail.append(&storage_card);

  detail.upcast()
}

/// Value or fallback when empty.
fn none_if_empty(value: &str, fallback: &str) -> String {
  if value.is_empty() {
    fallback.to_string()
  } else {
    value.to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;

  fn fixture(name: &str, body: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("systemsettings-about-test");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(name);
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(body.as_bytes()).unwrap();
    path
  }

  #[test]
  fn hostname_trims_and_misses() {
    let path = fixture("hostname", "tontoo-pc\n");
    assert_eq!(hostname_in(&path), "tontoo-pc");
    assert!(hostname_in(std::path::Path::new("/nonexistent-about-test")).is_empty());
  }

  #[test]
  fn processor_reads_model_name() {
    let path = fixture(
      "cpuinfo",
      "processor\t: 0\nmodel name\t: Apple M1 Pro\nmodel name\t: Apple M1 Pro\n",
    );
    assert_eq!(processor_in(&path), "Apple M1 Pro");
    assert!(processor_in(std::path::Path::new("/nonexistent-about-test")).is_empty());
  }

  #[test]
  fn memory_formats_gibibytes() {
    let path = fixture("meminfo", "MemTotal:       16777216 kB\n");
    assert_eq!(memory_in(&path), "16 GB");
    assert!(memory_in(std::path::Path::new("/nonexistent-about-test")).is_empty());
  }

  #[test]
  fn kernel_trims() {
    let path = fixture("osrelease", "6.12.1-arch1-1\n");
    assert_eq!(kernel_in(&path), "6.12.1-arch1-1");
  }

  #[test]
  fn human_bytes_picks_units() {
    assert_eq!(human_bytes(432 * 1000 * 1000 * 1000), "432 GB");
    assert_eq!(human_bytes(1000 * 1000 * 1000 * 1000), "1 TB");
    assert_eq!(human_bytes(1500 * 1000 * 1000 * 1000), "1.5 TB");
    assert_eq!(human_gib(16 * 1024 * 1024 * 1024), "16 GB");
  }

  #[test]
  fn apps_counts_files_and_folders_once() {
    let dir = std::env::temp_dir().join("systemsettings-apps-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Terminal.app")).unwrap();
    std::fs::write(dir.join("Tool.app"), b"x").unwrap();
    std::fs::create_dir_all(dir.join("Notes")).unwrap();
    std::fs::write(dir.join("readme.txt"), b"x").unwrap();
    // Symlinked bundle counts once (system-wide links).
    #[cfg(unix)]
    std::os::unix::fs::symlink(dir.join("Terminal.app"), dir.join("TermLink.app")).unwrap();
    let found = collect_apps(&[dir.clone()]);
    // Terminal.app dir + Tool.app file; the TermLink.app symlink resolves
    // to Terminal.app and counts once.
    assert_eq!(found.len(), 2);
    assert_eq!(apps_in(&[dir.as_path()]), 2);
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn user_app_dirs_lists_per_user_folders() {
    let root = std::env::temp_dir().join("systemsettings-users-test");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("tontoo").join("Applications")).unwrap();
    std::fs::create_dir_all(root.join("guest")).unwrap();
    std::fs::write(root.join("stray.txt"), b"x").unwrap();
    assert_eq!(
      user_app_dirs(&root),
      vec![root.join("tontoo").join("Applications")]
    );
    assert!(user_app_dirs(std::path::Path::new("/nonexistent-about-test")).is_empty());
    let _ = std::fs::remove_dir_all(&root);
  }

  #[test]
  fn df_parses_root_line() {
    let output = "Filesystem     1B-blocks         Used    Available Use% Mounted on\n\
                  /dev/vda1    1000000000000 432000000000 568000000000  44% /\n\
                  tmpfs                8192            0         8192   0% /dev\n";
    let parsed = storage_from_df(output).unwrap();
    assert_eq!(parsed.0, "/dev/vda1");
    assert_eq!(parsed.1, "432 GB");
    assert_eq!(parsed.2, "1 TB");
  }

  #[test]
  fn logo_prefers_versioned_asset() {
    // Versioned asset via the CoreIcon override wins over the bundle.
    let base = std::env::temp_dir().join("systemsettings-osversion-test");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("99.9.9")).unwrap();
    std::fs::write(base.join("99.9.9").join("TontooOS_Icon.png"), b"fake").unwrap();
    std::env::set_var("COREICON_OS_VERSION_DIR", &base);
    let resolved = os_logo_for_version("99.9.9").unwrap();
    assert!(resolved.ends_with("99.9.9/TontooOS_Icon.png"));
    std::env::remove_var("COREICON_OS_VERSION_DIR");
    // Unknown version falls back to the bundled logo.
    let fallback = os_logo_for_version("0.0.0-missing");
    assert!(fallback.is_some());
    let _ = std::fs::remove_dir_all(&base);
  }

  #[test]
  fn laptop_artwork_resolves_to_resources() {
    assert!(laptop_png().ends_with("Resources/laptop.png"));
  }
}
