use anyhow::Result;
use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone)]
pub struct Listener {
    pub proto: &'static str,
    pub addr: IpAddr,
    pub port: u16,
}

const TCP_LISTEN: &str = "0A";

pub fn list() -> Result<Vec<Listener>> {
    let mut seen: BTreeSet<(&'static str, u16)> = BTreeSet::new();
    let mut out = Vec::new();

    for (path, proto, v6) in [
        ("/proc/net/tcp",  "tcp",  false),
        ("/proc/net/tcp6", "tcp6", true),
    ] {
        let Ok(raw) = std::fs::read_to_string(path) else { continue; };
        for line in raw.lines().skip(1) {
            let cols: Vec<&str> = line.split_whitespace().collect();
            if cols.len() < 4 { continue; }
            if cols[3] != TCP_LISTEN { continue; }
            let Some((addr, port)) = parse_addr(cols[1], v6) else { continue; };
            if !seen.insert((proto, port)) { continue; }
            out.push(Listener { proto, addr, port });
        }
    }
    Ok(out)
}

fn parse_addr(s: &str, v6: bool) -> Option<(IpAddr, u16)> {
    let (ip_hex, port_hex) = s.split_once(':')?;
    let port = u16::from_str_radix(port_hex, 16).ok()?;
    if v6 {
        if ip_hex.len() != 32 { return None; }
        let mut bytes = [0u8; 16];
        // /proc/net/tcp6 stores 4 little-endian u32 chunks
        for i in 0..4 {
            let chunk = &ip_hex[i * 8..i * 8 + 8];
            let n = u32::from_str_radix(chunk, 16).ok()?;
            bytes[i * 4..i * 4 + 4].copy_from_slice(&n.to_le_bytes());
        }
        Some((IpAddr::V6(Ipv6Addr::from(bytes)), port))
    } else {
        if ip_hex.len() != 8 { return None; }
        let n = u32::from_str_radix(ip_hex, 16).ok()?;
        let bytes = n.to_le_bytes();
        Some((IpAddr::V4(Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3])), port))
    }
}
