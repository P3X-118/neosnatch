use crate::cache;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Advisories {
    pub source: String,
    pub critical: u32,
    pub high: u32,
    pub total: u32,
}

pub async fn check(ttl_secs: u64) -> Option<Advisories> {
    let ttl = Duration::from_secs(ttl_secs);
    if let Some(c) = cache::read::<Advisories>("advisories", ttl) {
        return Some(c);
    }
    let result = if which("apt-get").await.is_some() {
        debian().await
    } else if which("dnf").await.is_some() {
        dnf().await
    } else if which("arch-audit").await.is_some() {
        arch().await
    } else if Path::new("/etc/alpine-release").exists() {
        alpine().await
    } else {
        None
    };
    if let Some(ref r) = result {
        let _ = cache::write("advisories", r);
    }
    result
}

async fn which(bin: &str) -> Option<String> {
    let out = Command::new("which").arg(bin).output().await.ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

async fn debian() -> Option<Advisories> {
    // Use the cached file populated by unattended-upgrades / update-notifier.
    // No network calls; reflects what apt last saw.
    let path = "/var/lib/update-notifier/updates-available";
    if let Ok(raw) = tokio::fs::read_to_string(path).await {
        // Format: "N updates can be applied immediately.\nM of these updates are standard security updates."
        let mut total = 0u32;
        let mut sec = 0u32;
        for line in raw.lines() {
            if let Some(n) = first_number(line) {
                if line.contains("security") { sec = n; }
                else if total == 0 { total = n; }
            }
        }
        if total > 0 || sec > 0 {
            return Some(Advisories {
                source: "apt".into(),
                critical: 0,
                high: sec,
                total: total.max(sec),
            });
        }
    }
    // Fallback: parse `apt-get -s -o Debug::NoLocking=true upgrade`
    let out = Command::new("apt-get")
        .args(["-s", "-o", "Debug::NoLocking=true", "upgrade"])
        .output().await.ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    let mut total = 0u32;
    let mut sec = 0u32;
    for line in s.lines() {
        if line.starts_with("Inst ") {
            total += 1;
            if line.contains("-security") { sec += 1; }
        }
    }
    Some(Advisories { source: "apt".into(), critical: 0, high: sec, total })
}

async fn dnf() -> Option<Advisories> {
    let out = Command::new("dnf")
        .args(["-q", "updateinfo", "summary", "--available"])
        .output().await.ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    let mut a = Advisories { source: "dnf".into(), ..Default::default() };
    for line in s.lines() {
        let l = line.trim();
        if let Some(n) = trailing_number(l) {
            let lower = l.to_ascii_lowercase();
            if lower.contains("critical") { a.critical = n; a.total += n; }
            else if lower.contains("important") || lower.contains("high") { a.high = n; a.total += n; }
            else if lower.contains("security") { a.total += n; }
        }
    }
    Some(a)
}

async fn arch() -> Option<Advisories> {
    let out = Command::new("arch-audit").arg("-q").output().await.ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    let mut a = Advisories { source: "arch-audit".into(), ..Default::default() };
    for line in s.lines() {
        if line.is_empty() { continue; }
        a.total += 1;
        let lower = line.to_ascii_lowercase();
        if lower.contains("critical") { a.critical += 1; }
        else if lower.contains("high") { a.high += 1; }
    }
    Some(a)
}

async fn alpine() -> Option<Advisories> {
    // apk doesn't ship a security feed; fall back to upgrade-count signal.
    let out = Command::new("apk")
        .args(["version", "-l", "<"])
        .output().await.ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    let total = s.lines().filter(|l| !l.starts_with("Installed")).count() as u32;
    Some(Advisories { source: "apk".into(), total, ..Default::default() })
}

fn first_number(s: &str) -> Option<u32> {
    let mut digits = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() { digits.push(c); }
        else if !digits.is_empty() { break; }
    }
    digits.parse().ok()
}

fn trailing_number(s: &str) -> Option<u32> {
    s.split_whitespace().next()?.parse().ok()
}
