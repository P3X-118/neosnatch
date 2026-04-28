//! Privileged-data snapshot, written by `neosnatch --snapshot=PATH` (typically
//! root via systemd timer) and consumed at login by the unprivileged banner.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub const DEFAULT_PATH: &str = "/var/cache/neosnatch/snapshot.json";
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema: u32,
    pub captured_at: String, // RFC3339
    pub by_uid: u32,
    pub listeners: Vec<SnapshotListener>,
    /// bridge interface name → docker network name
    pub docker_networks: HashMap<String, String>,
    pub failed_units: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotListener {
    pub proto: String,
    pub addr: String,
    pub port: u16,
    pub process: Option<String>,
}

pub fn read(path: &Path) -> Option<Snapshot> {
    let raw = std::fs::read_to_string(path).ok()?;
    let snap: Snapshot = serde_json::from_str(&raw).ok()?;
    if snap.schema != SCHEMA_VERSION { return None; }
    Some(snap)
}

pub fn write_atomic(path: &Path, snap: &Snapshot) -> Result<()> {
    let parent = path.parent()
        .ok_or_else(|| anyhow::anyhow!("snapshot path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create {}", parent.display()))?;
    let raw = serde_json::to_string_pretty(snap)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, raw).with_context(|| format!("write {}", tmp.display()))?;
    let mut perms = std::fs::metadata(&tmp)?.permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&tmp, perms)?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

pub fn age_secs(snap: &Snapshot) -> Option<u64> {
    let captured: DateTime<Utc> = snap.captured_at.parse().ok()?;
    let age = Utc::now().signed_duration_since(captured);
    age.num_seconds().try_into().ok()
}

pub async fn generate(out: &Path) -> Result<()> {
    use crate::collect::{docker, ports, systemd};

    let listeners_live = ports::list().unwrap_or_default();
    let docker_map = docker::lookup().await;
    let failed = systemd::failed_units().await.unwrap_or_default();

    let listeners = listeners_live.into_iter().map(|l| SnapshotListener {
        proto: l.proto.to_string(),
        addr: l.addr.to_string(),
        port: l.port,
        process: l.process,
    }).collect();

    let snap = Snapshot {
        schema: SCHEMA_VERSION,
        captured_at: Utc::now().to_rfc3339(),
        by_uid: nix::unistd::getuid().as_raw(),
        listeners,
        docker_networks: docker_map.by_bridge,
        failed_units: failed,
    };
    write_atomic(out, &snap)?;
    Ok(())
}
