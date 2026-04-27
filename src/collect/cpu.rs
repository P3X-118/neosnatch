use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Load {
    pub one: f32,
    pub five: f32,
    pub fifteen: f32,
}

pub fn info() -> Result<CpuInfo> {
    let raw = std::fs::read_to_string("/proc/cpuinfo").context("read /proc/cpuinfo")?;
    let mut model = String::new();
    let mut cores = 0usize;
    for line in raw.lines() {
        if let Some(v) = line.strip_prefix("model name") {
            if model.is_empty() {
                model = v.trim_start_matches(|c: char| c == ':' || c.is_whitespace()).to_string();
            }
        }
        if line.starts_with("processor") { cores += 1; }
    }
    if model.is_empty() { model = "Unknown CPU".into(); }
    Ok(CpuInfo { model, cores })
}

pub fn load() -> Result<Load> {
    let raw = std::fs::read_to_string("/proc/loadavg").context("read /proc/loadavg")?;
    let parts: Vec<&str> = raw.split_whitespace().take(3).collect();
    let parse = |s: &str| s.parse::<f32>().unwrap_or(0.0);
    Ok(Load {
        one: parse(parts.first().copied().unwrap_or("0")),
        five: parse(parts.get(1).copied().unwrap_or("0")),
        fifteen: parse(parts.get(2).copied().unwrap_or("0")),
    })
}
