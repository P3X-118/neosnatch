use anyhow::Result;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct IfaceInfo {
    pub name: String,
    pub addrs: Vec<IpAddr>,
}

pub fn list() -> Result<Vec<IfaceInfo>> {
    use nix::ifaddrs::getifaddrs;
    use nix::sys::socket::SockaddrLike;
    let mut map: std::collections::BTreeMap<String, Vec<IpAddr>> = Default::default();
    for ifa in getifaddrs()? {
        if ifa.interface_name == "lo" {
            continue;
        }
        let Some(sa) = ifa.address else {
            continue;
        };
        let fam = sa.family();
        let ip = if fam == Some(nix::sys::socket::AddressFamily::Inet) {
            sa.as_sockaddr_in().map(|s| IpAddr::V4(s.ip()))
        } else if fam == Some(nix::sys::socket::AddressFamily::Inet6) {
            sa.as_sockaddr_in6().map(|s| IpAddr::V6(s.ip()))
        } else {
            None
        };
        if let Some(ip) = ip {
            if is_unwanted(&ip) {
                continue;
            }
            map.entry(ifa.interface_name).or_default().push(ip);
        }
    }
    Ok(map
        .into_iter()
        .map(|(name, addrs)| IfaceInfo { name, addrs })
        .collect())
}

fn is_unwanted(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback() || (v6.segments()[0] & 0xffc0) == 0xfe80,
    }
}
