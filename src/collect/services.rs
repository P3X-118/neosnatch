//! "Non-default" enabled systemd services: any .service whose unit file is
//! either dropped under /etc/systemd/system (admin-installed) or owned by a
//! manually-installed dpkg package. The base distro's enabled units (cron,
//! ssh, rsyslog, dbus, etc.) all come from auto-installed packages and so
//! are filtered out automatically.
//!
//! Walks /etc/systemd/system/<target>.wants/ for enabled symlinks and a
//! dpkg path→package index built from /var/lib/dpkg/info/*.list. Both
//! sources are world-readable; no caps required.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ServiceUnit {
    pub name: String,
    #[allow(dead_code)] // surfaced when verbose service listing is enabled
    pub package: Option<String>,
}

const WANTS_DIRS: &[&str] = &[
    "/etc/systemd/system/multi-user.target.wants",
    "/etc/systemd/system/default.target.wants",
    "/etc/systemd/system/sockets.target.wants",
    "/etc/systemd/system/timers.target.wants",
];

pub fn non_default(manual_pkgs: &[String]) -> Vec<ServiceUnit> {
    let manual: HashSet<&str> = manual_pkgs.iter().map(String::as_str).collect();
    let path_to_pkg = build_dpkg_path_index();
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<ServiceUnit> = Vec::new();

    for wants in WANTS_DIRS {
        let Ok(rd) = fs::read_dir(wants) else { continue; };
        for ent in rd.flatten() {
            let path = ent.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue; };
            if !name.ends_with(".service") { continue; }
            if seen.contains(name) { continue; }

            let resolved = resolve_link(&path);
            let pkg = path_to_pkg.get(resolved.to_string_lossy().as_ref()).cloned();

            let admin_added = resolved.starts_with("/etc/systemd/system")
                && !resolved.to_string_lossy().contains(".wants/");
            let manual_pkg = pkg.as_deref().map(|p| manual.contains(p)).unwrap_or(false);
            if !admin_added && !manual_pkg { continue; }

            seen.insert(name.to_string());
            out.push(ServiceUnit {
                name: name.trim_end_matches(".service").to_string(),
                package: pkg,
            });
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn resolve_link(p: &std::path::Path) -> PathBuf {
    match fs::read_link(p) {
        Ok(t) if t.is_absolute() => t,
        Ok(t) => p.parent().map(|d| d.join(&t)).unwrap_or(t)
                    .canonicalize().unwrap_or_else(|_| p.to_path_buf()),
        Err(_) => p.to_path_buf(),
    }
}

fn build_dpkg_path_index() -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(rd) = fs::read_dir("/var/lib/dpkg/info") else { return map; };
    for ent in rd.flatten() {
        let path = ent.path();
        let Some(file) = path.file_name().and_then(|n| n.to_str()) else { continue; };
        let Some(stem) = file.strip_suffix(".list") else { continue; };
        let pkg = stem.split(':').next().unwrap_or(stem).to_string();
        let Ok(content) = fs::read_to_string(&path) else { continue; };
        for line in content.lines() {
            if line.ends_with(".service") {
                map.insert(line.to_string(), pkg.clone());
            }
        }
    }
    map
}
