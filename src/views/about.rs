//! About detail page for SystemSettings, pushed from the General About
//! row.
//!
//! Device group (hostname, processor, memory, Linux kernel, installed app
//! count), TontooOS group (versioned OS logo, display name, dynamic
//! version from the daemon) and a storage group (device plus used/total
//! from `df`). All values read live with the localized Unknown fallback.

use std::path::{Path, PathBuf};

use crate::daemon;
use crate::lang;
use crate::views::{
  detail_header, Nav, PageView, Skin, BLOCK_GAP,
};
use crate::TontooUI::elements::{
  Align, BasicText, FileImage, Form, FormRow, FormSection, HStack, ImageFit, SFSymbolImage,
  TextAlignment, VStack,
};

/// Bundled OS logo used when no versioned asset exists.
pub(crate) fn bundled_logo() -> String {
  format!("{}/Resources/app_icon.png", env!("CARGO_MANIFEST_DIR"))
}

/// Bundled device artwork (`Resources/laptop.png`).
pub(crate) fn laptop_png() -> String {
  format!("{}/Resources/laptop.png", env!("CARGO_MANIFEST_DIR"))
}

/// OS logo for a version: the CoreIcon version asset (`TontooOS_Icon.png`
/// for the matching version), else the bundled logo, else nothing.
pub(crate) fn os_logo_for_version(version: &str) -> Option<String> {
  let path = crate::CoreIcon::os_version::os_version_path(version, "TontooOS_Icon.png");
  if Path::new(&path).is_file() {
    return Some(path);
  }
  let bundled = bundled_logo();
  Path::new(&bundled).is_file().then_some(bundled)
}

/// Hostname from a `hostname(5)`-style file, empty when unreadable.
pub(crate) fn hostname_in(path: &Path) -> String {
  std::fs::read_to_string(path)
    .map(|content| content.trim().to_string())
    .unwrap_or_default()
}

/// First `model name` from cpuinfo, empty when missing.
pub(crate) fn processor_in(path: &Path) -> String {
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

/// Total RAM from meminfo as `16 GB` (binary, like the macOS About
/// window), empty when unreadable.
pub(crate) fn memory_in(path: &Path) -> String {
  let content = match std::fs::read_to_string(path) {
    Ok(content) => content,
    Err(_) => return String::new(),
  };
  for line in content.lines() {
    let mut parts = line.split_whitespace();
    if parts.next() == Some("MemTotal:") {
      if let Some(kb) = parts.next().and_then(|value| value.parse::<f64>().ok()) {
        return human_gib((kb * 1024.0) as u64);
      }
    }
  }
  String::new()
}

/// Kernel release, empty when unreadable.
pub(crate) fn kernel_in(path: &Path) -> String {
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
    format!("{value:.0} {unit}")
  } else {
    let text = format!("{value:.1}");
    format!("{} {unit}", text.trim_end_matches(".0"))
  }
}

/// Installed `.app` bundles below the given directories.
pub(crate) fn apps_in(dirs: &[&Path]) -> usize {
  collect_apps(&dirs.iter().map(PathBuf::from).collect::<Vec<_>>()).len()
}

/// Per-user application folders (`<user>/Applications`) below a users
/// root, one per directory entry.
pub(crate) fn user_app_dirs(users_root: &Path) -> Vec<PathBuf> {
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
pub(crate) fn collect_apps(dirs: &[PathBuf]) -> Vec<PathBuf> {
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
      seen.insert(std::fs::canonicalize(&path).unwrap_or(path));
    }
  }
  let mut out: Vec<PathBuf> = seen.into_iter().collect();
  out.sort();
  out
}

/// Live installed app count: `/Applications`, every user's
/// `~/Applications` below `/Users`, plus `/System/Applications`.
pub(crate) fn apps_live() -> usize {
  let mut dirs = vec![
    PathBuf::from("/Applications"),
    PathBuf::from("/System/Applications"),
  ];
  dirs.extend(user_app_dirs(Path::new("/Users")));
  apps_in(&dirs.iter().map(PathBuf::as_path).collect::<Vec<_>>())
}

/// `(source, used, total)` for `/` from `df -B1` output, if parseable.
pub(crate) fn storage_from_df(output: &str) -> Option<(String, String, String)> {
  for line in output.lines().skip(1) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 || parts[5] != "/" {
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

/// Value or the localized fallback when empty.
fn none_if_empty(value: &str, fallback: &str) -> String {
  if value.is_empty() {
    fallback.to_string()
  } else {
    value.to_string()
  }
}

/// Header row: device artwork plus the hostname as the page headline.
/// Stacks take sized views only, so the artwork arm is chosen here.
fn device_header(hostname: &str, unknown: &str) -> HStack {
  let bundled = laptop_png();
  let mut row = HStack::new().spacing(16.0).align(Align::Leading);
  if Path::new(&bundled).is_file() {
    row = row.child(FileImage::new(bundled, 72.0, 72.0).radius(16.0).fit(ImageFit::Fit));
  } else {
    row = row.child(SFSymbolImage::new("laptopcomputer").size(48.0));
  }
  row.child(
    BasicText::new(none_if_empty(hostname, unknown))
      .size(17.0)
      .weight(600.0)
      .alignment(TextAlignment::Leading),
  )
}

/// TontooOS row: the versioned OS logo plus the display name.
fn os_row(version: &str) -> HStack {
  let mut row = HStack::new().spacing(14.0).align(Align::Leading);
  match os_logo_for_version(version) {
    Some(path) => row = row.child(FileImage::new(path, 44.0, 44.0).radius(22.0).fit(ImageFit::Fit)),
    None => row = row.child(SFSymbolImage::new("gear").size(32.0)),
  }
  row.child(
    BasicText::new(daemon::get_os().unwrap_or_default().display_name)
      .size(13.0)
      .alignment(TextAlignment::Leading),
  )
}

/// Build the About detail page.
pub(crate) fn build(_skin: &Skin, nav: &Nav) -> PageView {
  let unknown = lang::t("about.unknown");
  let hostname = hostname_in(Path::new("/etc/hostname"));
  let processor = processor_in(Path::new("/proc/cpuinfo"));
  let memory = memory_in(Path::new("/proc/meminfo"));
  let kernel = kernel_in(Path::new("/proc/sys/kernel/osrelease"));
  let os = daemon::get_os().unwrap_or_default();

  let device = Form::new().section(
    FormSection::new()
      .row(FormRow::text(
        lang::t("about.chip"),
        none_if_empty(&processor, &unknown),
      ))
      .row(FormRow::text(
        lang::t("about.memory"),
        none_if_empty(&memory, &unknown),
      ))
      .row(FormRow::text(
        lang::t("about.kernel"),
        none_if_empty(&kernel, &unknown),
      ))
      .row(FormRow::text(
        lang::t("about.apps"),
        apps_live().to_string(),
      )),
  );

  let (source, usage) = match storage_live() {
    Some((source, used, total)) => (source, format!("{} / {}", used, total)),
    None => (String::new(), unknown.clone()),
  };
  let storage = Form::new().section(
    FormSection::titled(lang::t("about.storage")).row(FormRow::text(
      if source.is_empty() {
        lang::t("about.name")
      } else {
        source
      },
      usage,
    )),
  );

  let body = VStack::new()
    .spacing(BLOCK_GAP)
    .align(Align::Leading)
    .child(device_header(&hostname, &unknown))
    .child(device)
    .child(os_row(&os.version))
    .child(storage);

  let header = detail_header(
    nav,
    "questionmark",
    &lang::t(crate::views::hidden_title(crate::views::ABOUT_HIDDEN)),
  );
  crate::views::page_shell(header, body)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;

  fn fixture(name: &str, body: &str) -> PathBuf {
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
    assert!(hostname_in(Path::new("/nonexistent-about-test")).is_empty());
  }

  #[test]
  fn processor_reads_model_name() {
    let path = fixture(
      "cpuinfo",
      "processor\t: 0\nmodel name\t: Apple M1 Pro\nmodel name\t: Apple M1 Pro\n",
    );
    assert_eq!(processor_in(&path), "Apple M1 Pro");
    assert!(processor_in(Path::new("/nonexistent-about-test")).is_empty());
  }

  #[test]
  fn memory_formats_gibibytes() {
    let path = fixture("meminfo", "MemTotal:       16777216 kB\n");
    assert_eq!(memory_in(&path), "16 GB");
    assert!(memory_in(Path::new("/nonexistent-about-test")).is_empty());
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
    assert!(user_app_dirs(Path::new("/nonexistent-about-test")).is_empty());
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
    assert!(storage_from_df("nothing useful here").is_none());
  }

  #[test]
  fn logo_prefers_versioned_asset() {
    let base = std::env::temp_dir().join("systemsettings-osversion-test");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("99.9.9")).unwrap();
    std::fs::write(base.join("99.9.9").join("TontooOS_Icon.png"), b"fake").unwrap();
    std::env::set_var("COREICON_OS_VERSION_DIR", &base);
    let resolved = os_logo_for_version("99.9.9").unwrap();
    assert!(resolved.ends_with("99.9.9/TontooOS_Icon.png"));
    std::env::remove_var("COREICON_OS_VERSION_DIR");
    // Unknown version falls back to the bundled logo.
    assert!(os_logo_for_version("0.0.0-missing").is_some());
    let _ = std::fs::remove_dir_all(&base);
  }

  #[test]
  fn laptop_artwork_resolves_to_resources() {
    assert!(laptop_png().ends_with("Resources/laptop.png"));
  }
}
