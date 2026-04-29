use std::collections::BTreeMap;
use std::net::{IpAddr, Ipv4Addr};

use crate::collect::{docker::NetworkMap, net::IfaceInfo, ports::Listener};

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

/// One Docker bridge interface with optional friendly name and primary IP.
#[derive(Debug, Clone)]
pub struct DockerBridge {
    pub iface: String,
    pub name: Option<String>,
    pub ip: Option<IpAddr>,
}

impl DockerBridge {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.iface)
    }
}

/// Split interfaces into (non-docker primary, docker bridges sorted by display name).
pub fn split_network<'a>(
    ifs: &'a [IfaceInfo],
    nets: &NetworkMap,
) -> (Vec<&'a IfaceInfo>, Vec<DockerBridge>) {
    let mut primary = Vec::new();
    let mut bridges = Vec::new();

    for ifi in ifs {
        let is_bridge = ifi.name.starts_with("br-") || ifi.name == "docker0";
        if is_bridge {
            bridges.push(DockerBridge {
                iface: ifi.name.clone(),
                name: nets.by_bridge.get(&ifi.name).cloned(),
                ip: ifi.addrs.first().copied(),
            });
        } else {
            primary.push(ifi);
        }
    }

    bridges.sort_by(|a, b| {
        // Named bridges first (alphabetical), then anonymous ones by iface name.
        match (&a.name, &b.name) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.iface.cmp(&b.iface),
        }
    });
    let _ = Ipv4Addr::UNSPECIFIED; // keep import used
    (primary, bridges)
}

/// One service entry: the process name and the ports it listens on, sorted ascending.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ServicePorts {
    pub name: String,
    pub ports: Vec<u16>,
}

/// Group listeners into (public services, local services).
/// "public" = bound to 0.0.0.0/:: or a specific non-loopback IP (externally reachable).
/// "local" = bound to a loopback address.
/// Dedupe by (service, port). Within each scope, sort by lowest port number.
#[allow(dead_code)]
pub fn group_ports_by_service(ls: &[Listener]) -> (Vec<ServicePorts>, Vec<ServicePorts>) {
    let mut pub_map: BTreeMap<String, BTreeMap<u16, ()>> = BTreeMap::new();
    let mut loc_map: BTreeMap<String, BTreeMap<u16, ()>> = BTreeMap::new();

    for l in ls {
        let loop_ = match l.addr {
            IpAddr::V4(v4) => v4.is_loopback(),
            IpAddr::V6(v6) => v6.is_loopback(),
        };
        let key = l.process.clone().unwrap_or_else(|| "?".into());
        let bucket = if loop_ { &mut loc_map } else { &mut pub_map };
        bucket.entry(key).or_default().insert(l.port, ());
    }

    let to_vec = |m: BTreeMap<String, BTreeMap<u16, ()>>| -> Vec<ServicePorts> {
        let mut v: Vec<ServicePorts> = m.into_iter().map(|(name, ports)| ServicePorts {
            name, ports: ports.keys().copied().collect()
        }).collect();
        // Sort by lowest port (ops scan "what's on 22, 80, 443, ..." first).
        v.sort_by_key(|s| s.ports.first().copied().unwrap_or(u16::MAX));
        v
    };

    (to_vec(pub_map), to_vec(loc_map))
}

#[allow(dead_code)]
pub fn fmt_ports(ports: &[u16]) -> String {
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
