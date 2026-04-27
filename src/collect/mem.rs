use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct MemInfo {
    pub total_kb: u64,
    pub available_kb: u64,
    pub swap_total_kb: u64,
    pub swap_free_kb: u64,
}

impl MemInfo {
    pub fn used_kb(&self) -> u64 { self.total_kb.saturating_sub(self.available_kb) }
    pub fn used_pct(&self) -> u8 {
        if self.total_kb == 0 { return 0; }
        ((self.used_kb() * 100) / self.total_kb) as u8
    }
    pub fn swap_used_kb(&self) -> u64 { self.swap_total_kb.saturating_sub(self.swap_free_kb) }
    pub fn swap_used_pct(&self) -> u8 {
        if self.swap_total_kb == 0 { return 0; }
        ((self.swap_used_kb() * 100) / self.swap_total_kb) as u8
    }
}

pub fn read() -> Result<MemInfo> {
    let raw = std::fs::read_to_string("/proc/meminfo").context("read /proc/meminfo")?;
    let mut map: HashMap<&str, u64> = HashMap::new();
    for line in raw.lines() {
        let Some((k, v)) = line.split_once(':') else { continue; };
        let v = v.trim().split_whitespace().next().unwrap_or("0");
        if let Ok(n) = v.parse::<u64>() { map.insert(k, n); }
    }
    Ok(MemInfo {
        total_kb: map.get("MemTotal").copied().unwrap_or(0),
        available_kb: map.get("MemAvailable").copied().unwrap_or(0),
        swap_total_kb: map.get("SwapTotal").copied().unwrap_or(0),
        swap_free_kb: map.get("SwapFree").copied().unwrap_or(0),
    })
}
