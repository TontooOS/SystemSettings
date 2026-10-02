//! Network settings page for SystemSettings.
//!
//! DNS group with an editable server field plus an Apply button (both
//! applied system-wide through the daemon, empty means DHCP) and one info
//! group per connected wired interface.

use std::cell::RefCell;
use std::rc::Rc;

use crate::daemon;
use crate::lang;
use crate::views::{
  header_subtitle, Nav, PageView, Skin, BLOCK_GAP, NETWORK,
};
use crate::TontooUI::elements::{
  Align, Button, ButtonStyle, Form, FormRow, FormSection, VStack,
};

/// Suggested manual servers when switching away from DHCP.
pub(crate) const DEFAULT_DNS_INPUT: &str = "1.1.1.1, 8.8.8.8";

/// User-facing text for a `dns_set` failure: validation errors get the
/// format hint, a missing NetworkManager or active connection gets the
/// unavailable note, anything else passes through raw.
pub(crate) fn dns_error_text(err: &str) -> String {
  if err.contains("invalid IPv4") {
    format!("{} ({})", lang::t("network.dns.invalid"), err)
  } else if err.contains("not available") || err.contains("no active connection") {
    format!("{} ({})", lang::t("network.dns.unavailable"), err)
  } else {
    err.to_string()
  }
}

/// Current DNS servers as the field shows them: the manual list, or the
/// Automatic placeholder while the daemon reports DHCP.
pub(crate) fn dns_value(state: &daemon::DnsState) -> String {
  if state.manual && !state.servers.is_empty() {
    state.servers.join(", ")
  } else {
    lang::t("network.dns.automatic")
  }
}

/// Apply a DNS server list through the daemon. Empty input means DHCP.
/// Returns the servers the daemon reports back.
fn apply_dns(input: &str) -> Result<String, String> {
  daemon::dns_set(input)
    .map(|state| state.servers.join(", "))
    .map_err(|err| dns_error_text(&err))
}

/// DNS group: the editable server field, an Apply button that commits the
/// draft, and the DHCP hint as the group footnote.
fn dns_group() -> Form {
  let state = daemon::dns_get().unwrap_or(daemon::DnsState {
    servers: Vec::new(),
    manual: false,
  });
  let draft: Rc<RefCell<String>> = Rc::new(RefCell::new(dns_value(&state)));

  let feeding = draft.clone();
  let field = FormRow::text(lang::t("network.dns"), dns_value(&state))
    .placeholder(DEFAULT_DNS_INPUT)
    .on_change(move |text| {
      *feeding.borrow_mut() = text.to_string();
    });

  let applying = draft.clone();
  let apply = Button::new(lang::t("network.dns.apply"))
    .style(ButtonStyle::BorderedProminent)
    .on_press(move || match apply_dns(applying.borrow().trim()) {
      Ok(applied) => println!("DNS set: {applied}"),
      Err(err) => println!("DNS set failed: {err}"),
    });

  Form::new().section(
    FormSection::new()
      .row(field)
      .row(FormRow::buttons(vec![apply]))
      .footnote(lang::t("network.dns.hint")),
  )
}

/// One info group per connected wired interface. Unknown fields are
/// skipped, and a machine without a wired link gets the empty note.
fn wired_groups() -> Form {
  let interfaces = daemon::wired_list().unwrap_or_default();
  if interfaces.is_empty() {
    return Form::new().section(FormSection::new().footnote(lang::t("network.wired.none")));
  }
  let mut form = Form::new();
  for info in interfaces.iter() {
    let name = if info.connection.is_empty() {
      info.interface.clone()
    } else {
      info.connection.clone()
    };
    let mut section = FormSection::titled(name).row(FormRow::text(
      lang::t("network.wired.info.state"),
      info.state.clone(),
    ));
    for (key, value) in detail_lines(info) {
      section = section.row(FormRow::text(lang::t(&key), value));
    }
    form = form.section(section);
  }
  form
}

/// Known detail lines for one interface; unknown fields are skipped.
fn detail_lines(info: &daemon::WiredInfo) -> Vec<(&'static str, String)> {
  let mut lines: Vec<(&'static str, String)> = vec![
    ("network.wired.info.interface", info.interface.clone()),
    ("network.wired.info.connection", info.connection.clone()),
  ];
  if !info.state.is_empty() {
    lines.push(("network.wired.info.state", info.state.clone()));
  }
  if !info.ipv4_addrs.is_empty() {
    lines.push((
      "network.wired.info.ip",
      info.ipv4_addrs.join(", "),
    ));
  }
  if let Some(gateway) = &info.gateway {
    lines.push(("network.wired.info.gateway", gateway.clone()));
  }
  if !info.mac.is_empty() {
    lines.push(("network.wired.info.mac", info.mac.clone()));
  }
  if let Some(speed) = info.speed_mbps {
    lines.push((
      "network.wired.info.speed",
      format!("{} Mb/s", speed),
    ));
  }
  if let Some(mtu) = info.mtu {
    lines.push(("network.wired.info.mtu", mtu.to_string()));
  }
  if let Some(driver) = &info.driver {
    lines.push(("network.wired.info.driver", driver.clone()));
  }
  lines
}

/// Build the Network detail page.
pub(crate) fn build(_skin: &Skin, _nav: &Nav) -> PageView {
  let body = VStack::new()
    .spacing(BLOCK_GAP)
    .align(Align::Leading)
    .child(dns_group())
    .child(wired_groups());

  crate::views::page_shell(
    crate::views::page_header(crate::views::header_symbol(NETWORK), &header_subtitle(NETWORK)),
    body,
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dns_validation_error_gets_format_hint() {
    let text = dns_error_text("dns set failed: invalid IPv4 address: nope");
    assert!(text.contains(&lang::t("network.dns.invalid")));
    assert!(text.contains("invalid IPv4 address: nope"));
  }

  #[test]
  fn dns_missing_tool_error_gets_unavailable_note() {
    let text = dns_error_text("dns set failed: Network hardware or tool not available");
    assert!(text.contains(&lang::t("network.dns.unavailable")));
    assert!(!text.contains(&lang::t("network.dns.invalid")));
  }

  #[test]
  fn dns_no_connection_error_gets_unavailable_note() {
    let text = dns_error_text("dns set failed: no active connection");
    assert!(text.contains(&lang::t("network.dns.unavailable")));
  }

  #[test]
  fn dns_other_errors_pass_through() {
    assert_eq!(dns_error_text("boom"), "boom");
  }

  #[test]
  fn dns_value_shows_manual_servers_or_automatic() {
    let manual = daemon::DnsState {
      servers: vec!["1.1.1.1".into(), "8.8.8.8".into()],
      manual: true,
    };
    assert_eq!(dns_value(&manual), "1.1.1.1, 8.8.8.8");
    let dhcp = daemon::DnsState {
      servers: vec!["1.1.1.1".into()],
      manual: false,
    };
    assert_eq!(dns_value(&dhcp), lang::t("network.dns.automatic"));
    let empty = daemon::DnsState {
      servers: Vec::new(),
      manual: true,
    };
    assert_eq!(dns_value(&empty), lang::t("network.dns.automatic"));
  }

  #[test]
  fn detail_lines_skip_unknown_fields() {
    let info = daemon::WiredInfo {
      interface: "eth0".to_string(),
      connection: "Wired".to_string(),
      state: String::new(),
      ipv4_addrs: Vec::new(),
      gateway: None,
      mac: String::new(),
      speed_mbps: None,
      mtu: None,
      driver: None,
    };
    let lines = detail_lines(&info);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].1, "eth0");
    assert_eq!(lines[1].1, "Wired");
  }
}
