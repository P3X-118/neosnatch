use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct Config {
    pub logo: LogoCfg,
    pub network: NetworkCfg,
    pub thresholds: Thresholds,
    pub show: Show,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LogoCfg {
    pub path: Option<PathBuf>,
    pub width: u16,
    pub height: u16,
    pub symbols: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkCfg {
    pub public_ip_url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Thresholds {
    pub disk_warn: u8,
    pub disk_crit: u8,
    pub mem_warn: u8,
    pub mem_crit: u8,
    pub load_warn: f32,
    pub load_crit: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Show {
    pub host: bool,
    pub os: bool,
    pub kernel: bool,
    pub uptime: bool,
    pub load: bool,
    pub cpu: bool,
    pub memory: bool,
    pub swap: bool,
    pub disk: bool,
    pub network: bool,
    pub public_ip: bool,
    pub sessions: bool,
    pub last_login: bool,
    pub failed_units: bool,
    pub listening_ports: bool,
    pub advisories: bool,
    pub packages: bool,
    pub model: bool,
    pub virt: bool,
    pub gpu: bool,
    pub shell: bool,
}

impl Default for LogoCfg {
    fn default() -> Self {
        Self {
            path: None,
            width: 24,
            height: 12,
            symbols: "block".into(),
        }
    }
}

impl Default for NetworkCfg {
    fn default() -> Self {
        Self {
            public_ip_url: "http://ip.sgc.ai".into(),
            timeout_ms: 1500,
        }
    }
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            disk_warn: 80,
            disk_crit: 92,
            mem_warn: 80,
            mem_crit: 92,
            load_warn: 1.0,
            load_crit: 2.0,
        }
    }
}

impl Default for Show {
    fn default() -> Self {
        Self {
            host: true,
            os: true,
            kernel: true,
            uptime: true,
            load: true,
            cpu: true,
            memory: true,
            swap: true,
            disk: true,
            network: true,
            public_ip: true,
            sessions: true,
            last_login: true,
            failed_units: true,
            listening_ports: true,
            advisories: true,
            packages: true,
            model: true,
            virt: true,
            gpu: true,
            shell: true,
        }
    }
}

pub fn load(override_path: Option<&Path>) -> Result<Config> {
    let path = match override_path {
        Some(p) => Some(p.to_path_buf()),
        None => default_path(),
    };
    let Some(path) = path else {
        return Ok(Config::default());
    };
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read config {}", path.display()))?;
    let cfg: Config =
        toml::from_str(&raw).with_context(|| format!("parse config {}", path.display()))?;
    Ok(cfg)
}

fn default_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("neosnatch/config.toml"))
}
