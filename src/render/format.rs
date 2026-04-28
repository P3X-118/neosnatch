use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr};

use crate::collect::{net::IfaceInfo, ports::Listener};

pub fn human_bytes(b: u64) -> String {
    const U: [&str; 5] = ["B", "K", "M", "G", "T"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 { v /= 1024.0; i += 1; }
    if v >= 100.0 || i == 0 { format!("{:.0}{}", v, U[i]) }
    else { format!("{:.1}{}", v, U[i]) }
}

/// Strip vendor prefix from a model string. e.g. ("Dell Inc.", "Dell PowerEdge R610") → "PowerEdge R610".
pub fn strip_vendor(vendor: &str, model: &str) -> String {
    let v_words: Vec<&str> = vendor.split_whitespace()
        .filter(|w| !matches!(*w, "Inc." | "Inc" | "Corp." | "Corp" | "LLC" | "Ltd" | "Ltd."))
        .collect();
    if v_words.is_empty() { return model.to_string(); }
    let m_words: Vec<&str> = model.split_whitespace().collect();
    if m_words.starts_with(&v_words[..]) {
        m_words[v_words.len()..].join(" ")
    } else if !v_words.is_empty() && m_words.first() == Some(&v_words[0]) {
        m_words[1..].join(" ")
    } else {
        model.to_string()
    }
}

pub fn clean_cpu(model: &str) -> String {
    // CPU strings often have repeated whitespace, "(R)", "(TM)", "CPU".
    let mut s = model
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("(r)", "")
        .replace("(tm)", "");
    s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    s = s.replace(" CPU @", " @").replace("CPU @", "@");
    // Drop redundant " CPU " in the middle of e.g. "Intel Xeon CPU E5540 @ 2.53GHz"
    s = s.replace(" CPU ", " ");
    s
}

/// Collapse interfaces into (primary list, docker-bridge summary line, public stays separate).
/// Returns (primary_ifaces, docker_summary_or_none).
pub fn collapse_ifaces(ifs: &[IfaceInfo]) -> (Vec<&IfaceInfo>, Option<String>) {
    let mut primary = Vec::new();
    let mut docker_v4: Vec<Ipv4Addr> = Vec::new();
    let mut docker_count = 0usize;

    for ifi in ifs {
        let is_docker_bridge = ifi.name.starts_with("br-") || ifi.name == "docker0";
        if is_docker_bridge {
            docker_count += 1;
            for a in &ifi.addrs {
                if let IpAddr::V4(v4) = a { docker_v4.push(*v4); }
            }
        } else {
            primary.push(ifi);
        }
    }

    let summary = if docker_count == 0 {
        None
    } else {
        docker_v4.sort();
        let range = match (docker_v4.first(), docker_v4.last()) {
            (Some(lo), Some(hi)) if lo == hi => format!("{lo}"),
            (Some(lo), Some(hi)) => format!("{lo} – {hi}"),
            _ => String::new(),
        };
        let plural = if docker_count == 1 { "bridge" } else { "bridges" };
        Some(if range.is_empty() {
            format!("{docker_count} {plural}")
        } else {
            format!("{docker_count} {plural} ({range})")
        })
    };

    (primary, summary)
}

/// Group listeners into (public_ports, local_ports). Dedupe by port. Sort numerically.
pub fn group_ports(ls: &[Listener]) -> (Vec<u16>, Vec<u16>) {
    let mut pub_ports: BTreeMap<u16, ()> = BTreeMap::new();
    let mut loc_ports: BTreeMap<u16, ()> = BTreeMap::new();
    for l in ls {
        let unspec = match l.addr {
            IpAddr::V4(v4) => v4.is_unspecified(),
            IpAddr::V6(v6) => v6.is_unspecified(),
        };
        let loop_ = match l.addr {
            IpAddr::V4(v4) => v4.is_loopback(),
            IpAddr::V6(v6) => v6.is_loopback(),
        };
        if unspec { pub_ports.insert(l.port, ()); }
        else if loop_ { loc_ports.insert(l.port, ()); }
        else { pub_ports.insert(l.port, ()); }  // bound to specific iface — still externally reachable
    }
    (pub_ports.keys().copied().collect(), loc_ports.keys().copied().collect())
}

pub fn join_ports(ports: &[u16]) -> String {
    ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")
}

pub fn iface_addrs(ifi: &IfaceInfo) -> String {
    ifi.addrs.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ")
}

pub fn gpu_label(g: &crate::collect::gpu::Gpu) -> String {
    // model is "vendor:device" hex; sometimes a name parenthetical (in demo).
    if g.model.contains('(') {
        // model already includes a friendly name
        let pretty = g.model.split('(').nth(1)
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(&g.model)
            .trim();
        format!("{} {}", g.vendor, pretty)
    } else {
        format!("{} GPU", g.vendor)
    }
}
