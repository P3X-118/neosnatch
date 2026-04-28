use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy)]
pub struct Uptime {
    pub secs: u64,
}

impl Uptime {
    pub fn pretty(&self) -> String {
        let s = self.secs;
        let d = s / 86_400;
        let h = (s % 86_400) / 3600;
        let m = (s % 3600) / 60;
        match (d, h, m) {
            (0, 0, m) => format!("{m}m"),
            (0, h, m) => format!("{h}h {m}m"),
            (d, h, m) => format!("{d}d {h}h {m}m"),
        }
    }
}

pub fn read() -> Result<Uptime> {
    let raw = std::fs::read_to_string("/proc/uptime").context("read /proc/uptime")?;
    let secs = raw.split_whitespace().next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;
    Ok(Uptime { secs })
}
