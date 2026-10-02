//! Wi-Fi settings page for SystemSettings.
//!
//! Header with the radio switch, a Known Networks list and a live scan
//! list, all read from the settings daemon. Without a wireless adapter
//! both lists show the no-hardware note; with the radio off only the
//! header stays visible. Picking a network raises a join request the app
//! turns into a modal sheet (with password entry for secured networks);
//! a successful connect is stored as known by the daemon.

use std::fmt::Write as _;

use crate::daemon;
use crate::lang;
use crate::views::{
  caption, header_symbol, header_subtitle, note, Nav, PageView, Skin, BLOCK_GAP, HEADER_GAP,
  HEADER_SYMBOL_PX, TEXT_W, WIFI,
};
use crate::TontooUI::elements::{
  Align, BasicOutlineGroup, BasicText, HStack, OutlineNode, SFSymbolImage, Spacer, TextAlignment,
  Toggle, VStack,
};

/// A network counts as secured when the daemon reports anything other
/// than an empty string or `OPEN`.
pub(crate) fn is_secured(security: &str) -> bool {
  !security.trim().is_empty() && security.trim().to_uppercase() != "OPEN"
}

/// One network in either list.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Row {
  pub ssid: String,
  pub secured: bool,
  pub signal_pct: Option<i32>,
}

/// What the daemon reports about the radio right now.
#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Radio {
  /// Daemon unreachable: example rows so the page stays explorable.
  Unreachable,
  /// No wireless adapter present.
  NoAdapter,
  /// Radio off.
  Off,
  /// Radio on with the resolved lists.
  On { known: Vec<Row>, networks: Vec<Row> },
}

impl Row {
  /// Example rows shown while the daemon is unreachable or the scan
  /// fails, so the page is never blank.
  fn examples() -> Vec<Row> {
    vec![
      Row { ssid: lang::t("wifi.row.home"), secured: true, signal_pct: Some(82) },
      Row { ssid: lang::t("wifi.row.lab"), secured: true, signal_pct: Some(64) },
    ]
  }
}

/// Resolve the radio state plus the known and scanned networks.
pub(crate) fn resolve_state() -> Radio {
  let Ok(state) = daemon::status() else {
    return Radio::Unreachable;
  };
  if !state.available {
    return Radio::NoAdapter;
  }
  if !state.enabled {
    return Radio::Off;
  }
  let known = daemon::known_list()
    .unwrap_or_default()
    .into_iter()
    .map(|entry| Row {
      ssid: entry.ssid,
      secured: is_secured(&entry.security),
      signal_pct: None,
    })
    .collect();
  let networks = match daemon::list() {
    Ok(found) => found
      .into_iter()
      .map(|entry| Row {
        ssid: entry.ssid,
        secured: is_secured(&entry.security),
        signal_pct: Some(entry.signal_pct),
      })
      .collect(),
    Err(_) => Row::examples(),
  };
  Radio::On { known, networks }
}

/// Coarse state fingerprint: the radio state plus the SSID sets. Signal
/// percentages are left out on purpose, so a live scan does not rebuild
/// the page on every poll.
pub(crate) fn fingerprint(state: &Radio) -> String {
  let mut out = String::new();
  match state {
    Radio::Unreachable => out.push_str("unreachable"),
    Radio::NoAdapter => out.push_str("no-adapter"),
    Radio::Off => out.push_str("off"),
    Radio::On { known, networks } => {
      out.push_str("on;known=");
      for row in known {
        let _ = write!(out, "{},{};", row.ssid, row.secured);
      }
      out.push(';');
      out.push_str("networks=");
      for row in networks {
        let _ = write!(out, "{},{};", row.ssid, row.secured);
      }
    }
  }
  out
}

/// Signal strength label for a scanned network.
fn signal_text(pct: i32) -> String {
  format!("{} %", pct.clamp(0, 100))
}

/// Row label: SSID plus the signal for a live scan, the lock marker for a
/// secured network.
fn row_label(row: &Row) -> String {
  let mut label = row.ssid.clone();
  if let Some(pct) = row.signal_pct {
    let _ = write!(label, "  ({})", signal_text(pct));
  }
  if row.secured {
    let _ = write!(label, "  {}", lang::t("wifi.row.locked"));
  }
  label
}

/// Tappable network list. Picking a row asks the app to open the join
/// sheet; the SSIDs are captured by value so the list owns its own data.
fn network_list(rows: &[Row], nav: Nav) -> BasicOutlineGroup {
  let picks: Vec<(String, bool)> = rows
    .iter()
    .map(|row| (row.ssid.clone(), row.secured))
    .collect();
  let nodes: Vec<OutlineNode> = rows
    .iter()
    .map(|row| {
      OutlineNode::file(row_label(row)).icon(if row.secured {
        "lock.fill"
      } else {
        "wifi"
      })
    })
    .collect();
  BasicOutlineGroup::new(nodes)
    .selectable(true)
    .trailing_chevron(false)
    .on_select(move |path| {
      if let Some(&index) = path.first() {
        if let Some((ssid, secured)) = picks.get(index) {
          nav.request_join(ssid, *secured);
        }
      }
    })
}

/// Header row with the radio switch pinned to the trailing edge.
fn radio_header(state: &Radio, nav: Nav) -> HStack {
  HStack::new()
    .spacing(HEADER_GAP)
    .align(Align::Leading)
    .child(SFSymbolImage::new(header_symbol(WIFI)).size(HEADER_SYMBOL_PX))
    .child(
      BasicText::new(header_subtitle(WIFI))
        .size(13.0)
        .weight(400.0)
        .width(TEXT_W)
        .alignment(TextAlignment::Leading),
    )
    .child(Spacer::new().factor(1.0))
    .child(radio_switch(state, nav))
}

/// Header switch reflecting the radio state.
fn radio_switch(state: &Radio, nav: Nav) -> Toggle {
  let (on, live) = match state {
    Radio::Unreachable => (true, true),
    Radio::NoAdapter => (false, false),
    Radio::Off => (false, true),
    Radio::On { .. } => (true, true),
  };
  if !live {
    // No adapter: the switch is off and stays inert.
    return Toggle::new("").on(on).disabled(true);
  }
  Toggle::new("").on(on).on_toggle(move |on| {
    if let Err(err) = daemon::set_enabled(on) {
      println!("Wi-Fi radio toggle failed: {err}");
    }
    nav.touch();
  })
}
/// Build the Wi-Fi detail page.
pub(crate) fn build(_skin: &Skin, nav: &Nav) -> PageView {
  let state = resolve_state();
  let header = radio_header(&state, nav.clone());
  let mut body = VStack::new().spacing(BLOCK_GAP).align(Align::Leading);

  match state {
    Radio::Off => {
      // The header carries the whole page while the radio is off.
    }
    Radio::NoAdapter => {
      body = body
        .child(caption(&lang::t("wifi.known.header")))
        .child(note(&lang::t("wifi.no_adapter")))
        .child(caption(&lang::t("wifi.networks.header")))
        .child(note(&lang::t("wifi.no_adapter")));
    }
    Radio::Unreachable => {
      body = body
        .child(caption(&lang::t("wifi.networks.header")))
        .child(network_list(&Row::examples(), nav.clone()));
    }
    Radio::On { known, networks } => {
      if !known.is_empty() {
        body = body
          .child(caption(&lang::t("wifi.known.header")))
          .child(network_list(&known, nav.clone()));
      }
      body = body.child(caption(&lang::t("wifi.networks.header")));
      if networks.is_empty() {
        body = body.child(note(&lang::t("wifi.no_networks")));
      } else {
        body = body.child(network_list(&networks, nav.clone()));
      }
    }
  }

  crate::views::page_shell(header, body)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn open_networks_have_no_lock() {
    assert!(!is_secured("OPEN"));
    assert!(!is_secured("open"));
    assert!(!is_secured(""));
  }

  #[test]
  fn secured_networks_show_lock() {
    assert!(is_secured("WPA2"));
    assert!(is_secured("WPA3"));
    assert!(is_secured("WEP"));
  }

  #[test]
  fn signal_text_clamps_to_a_percentage() {
    assert_eq!(signal_text(0), "0 %");
    assert_eq!(signal_text(82), "82 %");
    assert_eq!(signal_text(140), "100 %");
    assert_eq!(signal_text(-20), "0 %");
  }

  #[test]
  fn row_labels_carry_signal_and_lock() {
    let row = Row { ssid: "Home".into(), secured: true, signal_pct: Some(82) };
    let label = row_label(&row);
    assert!(label.starts_with("Home"));
    assert!(label.contains("82 %"));
    assert!(label.contains(&lang::t("wifi.row.locked")));
    let open = Row { ssid: "Cafe".into(), secured: false, signal_pct: None };
    assert_eq!(row_label(&open), "Cafe");
  }

  #[test]
  fn fingerprint_ignores_signal_but_tracks_membership() {
    let a = Radio::On {
      known: vec![],
      networks: vec![Row { ssid: "Home".into(), secured: true, signal_pct: Some(30) }],
    };
    let b = Radio::On {
      known: vec![],
      networks: vec![Row { ssid: "Home".into(), secured: true, signal_pct: Some(90) }],
    };
    assert_eq!(fingerprint(&a), fingerprint(&b));
    let c = Radio::On {
      known: vec![],
      networks: vec![Row { ssid: "Home".into(), secured: false, signal_pct: Some(30) }],
    };
    assert_ne!(fingerprint(&a), fingerprint(&c));
    assert_ne!(fingerprint(&Radio::Off), fingerprint(&Radio::NoAdapter));
    assert_ne!(fingerprint(&Radio::Off), fingerprint(&Radio::Unreachable));
  }
}
