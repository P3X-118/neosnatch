use crate::config::Config;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn render(cfg: &Config, override_path: Option<&Path>) -> Result<Vec<String>> {
    let path = resolve_path(cfg, override_path);
    let Some(path) = path else { return Ok(Vec::new()); };
    if !path.exists() { return Ok(Vec::new()); }

    let size = format!("{}x{}", cfg.logo.width, cfg.logo.height);
    let out = Command::new("chafa")
        .args([
            "--format=symbols",
            "--symbols", &cfg.logo.symbols,
            "--size", &size,
        ])
        .arg(&path)
        .output();

    let out = match out {
        Ok(o) if o.status.success() => o.stdout,
        _ => return Ok(Vec::new()),
    };

    let s = String::from_utf8_lossy(&out);
    Ok(s.lines().map(str::to_owned).collect())
}

fn resolve_path(cfg: &Config, override_path: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = override_path { return Some(p.to_path_buf()); }
    if let Some(p) = &cfg.logo.path { return Some(p.clone()); }
    // default: $XDG_CONFIG_HOME/neosnatch/logo.png, then /etc/neosnatch/logo.png
    if let Some(d) = dirs::config_dir() {
        let p = d.join("neosnatch/logo.png");
        if p.exists() { return Some(p); }
    }
    let etc = PathBuf::from("/etc/neosnatch/logo.png");
    if etc.exists() { return Some(etc); }
    None
}
