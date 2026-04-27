use std::path::Path;
use tokio::fs;

pub async fn count() -> Option<u64> {
    if Path::new("/var/lib/dpkg/status").exists() {
        return count_dpkg().await;
    }
    if Path::new("/var/lib/pacman/local").exists() {
        return count_pacman().await;
    }
    if Path::new("/lib/apk/db/installed").exists() {
        return count_apk().await;
    }
    if Path::new("/var/lib/rpm").exists() {
        return count_rpm().await;
    }
    None
}

async fn count_dpkg() -> Option<u64> {
    let raw = fs::read_to_string("/var/lib/dpkg/status").await.ok()?;
    let mut n = 0u64;
    let mut installed = false;
    let mut has_pkg = false;
    for line in raw.lines() {
        if line.is_empty() {
            if has_pkg && installed { n += 1; }
            installed = false;
            has_pkg = false;
        } else if line.starts_with("Package:") {
            has_pkg = true;
        } else if let Some(v) = line.strip_prefix("Status:") {
            installed = v.contains("installed") && !v.contains("not-installed");
        }
    }
    if has_pkg && installed { n += 1; }
    Some(n)
}

async fn count_pacman() -> Option<u64> {
    let mut rd = fs::read_dir("/var/lib/pacman/local").await.ok()?;
    let mut n = 0u64;
    while let Ok(Some(e)) = rd.next_entry().await {
        if e.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
            n += 1;
        }
    }
    if n > 0 { n -= 1; } // ALPM_DB_VERSION sentinel
    Some(n)
}

async fn count_apk() -> Option<u64> {
    let raw = fs::read_to_string("/lib/apk/db/installed").await.ok()?;
    Some(raw.lines().filter(|l| l.starts_with("P:")).count() as u64)
}

async fn count_rpm() -> Option<u64> {
    let out = tokio::process::Command::new("rpm")
        .args(["-qa", "--queryformat", ".\n"])
        .output().await.ok()?;
    if !out.status.success() { return None; }
    Some(out.stdout.iter().filter(|&&b| b == b'\n').count() as u64)
}
