use std::path::Path;
use tokio::fs;

#[derive(Default, Debug, Clone)]
pub struct Stats {
    pub count: u64,
    /// Sum of declared installed-size in KB (dpkg only; None on other distros).
    pub total_kb: Option<u64>,
    /// Manually-installed packages (not pulled in as dependencies). Sorted.
    pub manual: Vec<String>,
    pub manager: Option<&'static str>,
}

pub async fn count() -> Option<Stats> {
    if Path::new("/var/lib/dpkg/status").exists() {
        return stats_dpkg().await;
    }
    if Path::new("/var/lib/pacman/local").exists() {
        return count_pacman().await.map(|c| Stats { count: c, total_kb: None, manual: vec![], manager: Some("pacman") });
    }
    if Path::new("/lib/apk/db/installed").exists() {
        return count_apk().await.map(|c| Stats { count: c, total_kb: None, manual: vec![], manager: Some("apk") });
    }
    if Path::new("/var/lib/rpm").exists() {
        return count_rpm().await.map(|c| Stats { count: c, total_kb: None, manual: vec![], manager: Some("rpm") });
    }
    None
}

async fn stats_dpkg() -> Option<Stats> {
    let raw = fs::read_to_string("/var/lib/dpkg/status").await.ok()?;
    let mut count = 0u64;
    let mut total_kb = 0u64;
    let mut installed_names: Vec<String> = Vec::new();

    let mut name = String::new();
    let mut installed = false;
    let mut has_pkg = false;
    let mut size_kb: u64 = 0;
    for line in raw.lines() {
        if line.is_empty() {
            if has_pkg && installed {
                count += 1;
                total_kb = total_kb.saturating_add(size_kb);
                if !name.is_empty() { installed_names.push(std::mem::take(&mut name)); }
            }
            installed = false;
            has_pkg = false;
            size_kb = 0;
            name.clear();
        } else if let Some(v) = line.strip_prefix("Package:") {
            has_pkg = true;
            name = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("Status:") {
            installed = v.contains("installed") && !v.contains("not-installed");
        } else if let Some(v) = line.strip_prefix("Installed-Size:") {
            size_kb = v.trim().parse().unwrap_or(0);
        }
    }
    if has_pkg && installed {
        count += 1;
        total_kb = total_kb.saturating_add(size_kb);
        if !name.is_empty() { installed_names.push(name); }
    }

    // Apt extended_states: any package with Auto-Installed: 1 is a dep.
    // Manual packages = installed minus auto.
    let auto = parse_auto_installed().await;
    let mut manual: Vec<String> = installed_names.into_iter()
        .filter(|n| !auto.contains(n))
        .collect();
    manual.sort();
    manual.dedup();

    Some(Stats { count, total_kb: Some(total_kb), manual, manager: Some("apt") })
}

async fn parse_auto_installed() -> std::collections::HashSet<String> {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let Ok(raw) = fs::read_to_string("/var/lib/apt/extended_states").await else { return set; };
    let mut name = String::new();
    let mut auto = false;
    for line in raw.lines() {
        if line.is_empty() {
            if auto && !name.is_empty() { set.insert(std::mem::take(&mut name)); }
            name.clear();
            auto = false;
        } else if let Some(v) = line.strip_prefix("Package:") {
            name = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("Auto-Installed:") {
            auto = v.trim() == "1";
        }
    }
    if auto && !name.is_empty() { set.insert(name); }
    set
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
