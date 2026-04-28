use std::collections::HashMap;
use std::fs;

/// Walk /proc/*/fd, building a map of socket-inode -> process comm.
/// Without root we can only see our own pids, so the map is best-effort.
pub fn socket_inode_map() -> HashMap<u64, String> {
    let mut out: HashMap<u64, String> = HashMap::new();
    let Ok(rd) = fs::read_dir("/proc") else { return out; };
    for entry in rd.flatten() {
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if !name_s.chars().all(|c| c.is_ascii_digit()) { continue; }

        let pid_path = entry.path();
        let comm = fs::read_to_string(pid_path.join("comm"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("pid:{name_s}"));

        let Ok(fds) = fs::read_dir(pid_path.join("fd")) else { continue; };
        for fd in fds.flatten() {
            let Ok(link) = fs::read_link(fd.path()) else { continue; };
            let s = link.to_string_lossy();
            let Some(rest) = s.strip_prefix("socket:[") else { continue; };
            let Some(inode_s) = rest.strip_suffix(']') else { continue; };
            if let Ok(inode) = inode_s.parse::<u64>() {
                // First writer wins — typically the actual listener, not a forked child.
                out.entry(inode).or_insert_with(|| comm.clone());
            }
        }
    }
    out
}
