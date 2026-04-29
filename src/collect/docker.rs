use crate::cache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkMap {
    /// Map of bridge interface name (e.g. "br-3930fae8dd8e", "docker0") to
    /// docker network name (e.g. "proxy-net", "bridge").
    pub by_bridge: HashMap<String, String>,
}

const CACHE_TTL_SECS: u64 = 24 * 3600;
const CACHE_KEY: &str = "docker_networks";

pub async fn lookup() -> NetworkMap {
    if !docker_present() { return NetworkMap::default(); }
    if let Some(c) = cache::read::<NetworkMap>(CACHE_KEY, Duration::from_secs(CACHE_TTL_SECS)) {
        return c;
    }
    let map = query().await.unwrap_or_default();
    if !map.by_bridge.is_empty() {
        let _ = cache::write(CACHE_KEY, &map);
    }
    map
}

fn docker_present() -> bool {
    Path::new("/var/lib/docker").exists() || Path::new("/run/docker.sock").exists()
}

/// Container port mapping derived from `docker ps`. Represents one host-side
/// publish entry: e.g. 0.0.0.0:80 → 80/tcp on container "nginx".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPort {
    pub container: String,
    pub network: String,
    pub host_port: u16,
    pub container_port: u16,
    pub proto: String,
}

pub async fn container_ports() -> Vec<ContainerPort> {
    if !docker_present() { return Vec::new(); }
    let Ok(out) = Command::new("docker")
        .args(["ps", "--no-trunc", "--format", "{{.Names}}\t{{.Networks}}\t{{.Ports}}"])
        .output().await
    else { return Vec::new(); };
    if !out.status.success() { return Vec::new(); }
    let raw = String::from_utf8_lossy(&out.stdout);

    let mut result = Vec::new();
    for line in raw.lines() {
        let cols: Vec<&str> = line.splitn(3, '\t').collect();
        if cols.len() != 3 { continue; }
        let container = cols[0].to_string();
        let network = cols[1].split(',').next().unwrap_or("").trim().to_string();
        for entry in cols[2].split(',') {
            let entry = entry.trim();
            if entry.is_empty() { continue; }
            // "0.0.0.0:80->80/tcp" or "[::]:80->80/tcp" or "8080/tcp" (no host bind)
            let Some((bind, target)) = entry.split_once("->") else { continue };
            let host_port = bind.rsplit(':').next().and_then(|p| p.parse::<u16>().ok());
            let (cport_s, proto) = target.split_once('/').unwrap_or((target, "tcp"));
            let container_port = cport_s.parse::<u16>().ok();
            if let (Some(hp), Some(cp)) = (host_port, container_port) {
                result.push(ContainerPort {
                    container: container.clone(),
                    network: network.clone(),
                    host_port: hp,
                    container_port: cp,
                    proto: proto.to_string(),
                });
            }
        }
    }
    result.sort_by_key(|c| c.host_port);
    result.dedup_by(|a, b| a.host_port == b.host_port && a.container == b.container);
    result
}

async fn query() -> Option<NetworkMap> {
    // Format: "<id>\t<name>\t<driver>". Skip ipvlan/macvlan/null/host —
    // bridge is what produces br-* and docker0 interfaces.
    let out = Command::new("docker")
        .args(["network", "ls", "--format", "{{.ID}}\t{{.Name}}\t{{.Driver}}", "--no-trunc"])
        .output().await.ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8_lossy(&out.stdout);

    let mut map = NetworkMap::default();
    for line in s.lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 3 { continue; }
        let (id, name, driver) = (cols[0], cols[1], cols[2]);
        if driver != "bridge" { continue; }
        if name == "bridge" {
            map.by_bridge.insert("docker0".into(), name.into());
        } else {
            // Bridge iface uses first 12 chars of network ID after "br-".
            let short = id.trim_start_matches("sha256:").chars().take(12).collect::<String>();
            map.by_bridge.insert(format!("br-{short}"), name.into());
        }
    }
    Some(map)
}
