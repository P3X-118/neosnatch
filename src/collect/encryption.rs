//! LUKS / dm-crypt detection for `/` and `/boot`.
//! Reads /proc/mounts and /sys/block/<dm>/dm/uuid (both world-readable).

use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    /// Encrypted; carries the kind label parsed from /sys/block/<dm>/dm/uuid
    /// (e.g. "LUKS1", "LUKS2", "PLAIN").
    Encrypted(String),
    Unencrypted,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct MountStatus {
    pub mount: String,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct Encryption {
    pub mounts: Vec<MountStatus>,
}

pub fn detect() -> Encryption {
    let raw = fs::read_to_string("/proc/mounts").unwrap_or_default();
    let mut out: Vec<MountStatus> = Vec::new();
    for target in ["/", "/boot"] {
        if let Some(dev) = source_for(target, &raw) {
            out.push(MountStatus { mount: target.into(), status: classify(&dev) });
        }
    }
    Encryption { mounts: out }
}

fn source_for(target: &str, mounts: &str) -> Option<String> {
    for line in mounts.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() >= 2 && cols[1] == target {
            return Some(cols[0].to_string());
        }
    }
    None
}

fn classify(dev: &str) -> Status {
    let Some(name) = dev.strip_prefix("/dev/mapper/") else {
        return Status::Unencrypted;
    };
    let Some(uuid) = lookup_dm_uuid(name) else { return Status::Unknown };
    if let Some(rest) = uuid.strip_prefix("CRYPT-") {
        let kind = rest.split('-').next().unwrap_or("CRYPT").to_string();
        Status::Encrypted(kind)
    } else {
        Status::Unencrypted
    }
}

fn lookup_dm_uuid(name: &str) -> Option<String> {
    let rd = fs::read_dir("/sys/block").ok()?;
    for ent in rd.flatten() {
        let p = ent.path();
        let bn = p.file_name()?.to_str()?.to_string();
        if !bn.starts_with("dm-") { continue; }
        let nm = fs::read_to_string(p.join("dm/name")).unwrap_or_default();
        if nm.trim() == name {
            return Some(fs::read_to_string(p.join("dm/uuid")).unwrap_or_default().trim().to_string());
        }
    }
    None
}
