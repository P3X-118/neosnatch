use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OsInfo {
    pub pretty_name: String,
    #[allow(dead_code)] // available for distro-specific behavior (e.g. advisory adapter dispatch)
    pub id: String,
    #[allow(dead_code)] // exposed for renderer overrides; pretty_name covers the default case
    pub version: Option<String>,
}

pub fn detect() -> Result<OsInfo> {
    let raw = std::fs::read_to_string("/etc/os-release").context("read /etc/os-release")?;
    let map = parse(&raw);
    Ok(OsInfo {
        pretty_name: map
            .get("PRETTY_NAME")
            .cloned()
            .or_else(|| map.get("NAME").cloned())
            .unwrap_or_else(|| "Linux".into()),
        id: map.get("ID").cloned().unwrap_or_else(|| "linux".into()),
        version: map.get("VERSION_ID").cloned(),
    })
}

fn parse(s: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let v = v.trim().trim_matches('"').trim_matches('\'');
        out.insert(k.trim().to_string(), v.to_string());
    }
    out
}
