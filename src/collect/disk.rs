use anyhow::{Context, Result};
use rustix::fs::{statvfs, StatVfs};

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub mount: String,
    pub fs: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
}

impl DiskInfo {
    pub fn used_pct(&self) -> u8 {
        if self.total_bytes == 0 { return 0; }
        ((self.used_bytes * 100) / self.total_bytes) as u8
    }
}

pub fn list() -> Result<Vec<DiskInfo>> {
    let raw = std::fs::read_to_string("/proc/mounts").context("read /proc/mounts")?;
    let mut out = Vec::new();
    for line in raw.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 { continue; }
        let dev = parts[0];
        let mount = parts[1];
        let fs = parts[2];
        if !is_real_fs(fs, dev) { continue; }
        let Ok(stat) = statvfs(mount) else { continue; };
        let info = from_statvfs(mount, fs, &stat);
        if info.total_bytes == 0 { continue; }
        out.push(info);
    }
    Ok(out)
}

fn from_statvfs(mount: &str, fs: &str, s: &StatVfs) -> DiskInfo {
    let block = s.f_frsize as u64;
    let total = s.f_blocks as u64 * block;
    let avail = s.f_bavail as u64 * block;
    DiskInfo {
        mount: mount.to_string(),
        fs: fs.to_string(),
        total_bytes: total,
        used_bytes: total.saturating_sub(avail),
    }
}

fn is_real_fs(fs: &str, dev: &str) -> bool {
    matches!(fs, "ext4" | "ext3" | "ext2" | "xfs" | "btrfs" | "zfs" | "f2fs"
        | "vfat" | "exfat" | "ntfs" | "ntfs3" | "fuseblk" | "nfs" | "nfs4")
        && !dev.is_empty()
}
