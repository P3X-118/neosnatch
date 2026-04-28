use crate::config::Config;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

// Embedded default — used when no override is configured and no chafa-rendered
// image is available. Uses ANSI cyan + bold for the wordmark.
const DEFAULT_LOGO: &str = concat!(
    "\x1b[36m       _.-._\n",
    "     .-'     '-.\n",
    "    /  .-. .-.  \\\n",
    "    | (   |   ) |\n",
    "    \\  '-' '-'  /\n",
    "     '-.     .-'\n",
    "        '---'\n",
    "\x1b[1;36m   N E O S N A T C H\x1b[22;39m",
);

pub fn render(cfg: &Config, override_path: Option<&Path>) -> Result<Vec<String>> {
    if let Some(lines) = render_image(cfg, override_path) {
        return Ok(lines);
    }
    Ok(DEFAULT_LOGO.lines().map(str::to_owned).collect())
}

fn render_image(cfg: &Config, override_path: Option<&Path>) -> Option<Vec<String>> {
    let path = resolve_image(cfg, override_path)?;
    if !path.exists() { return None; }
    if Command::new("chafa").arg("--version").output().ok()
        .map(|o| !o.status.success()).unwrap_or(true) { return None; }

    let size = format!("{}x{}", cfg.logo.width, cfg.logo.height);
    let out = Command::new("chafa")
        .args([
            "--format=symbols",
            "--symbols", &cfg.logo.symbols,
            "--size", &size,
        ])
        .arg(&path)
        .output().ok()?;
    if !out.status.success() { return None; }

    let s = String::from_utf8_lossy(&out.stdout);
    Some(s.lines().map(str::to_owned).collect())
}

fn resolve_image(cfg: &Config, override_path: Option<&Path>) -> Option<PathBuf> {
    if let Some(p) = override_path { return Some(p.to_path_buf()); }
    if let Some(p) = &cfg.logo.path { return Some(p.clone()); }
    if let Some(d) = dirs::config_dir() {
        let p = d.join("neosnatch/logo.png");
        if p.exists() { return Some(p); }
    }
    let etc = PathBuf::from("/etc/neosnatch/logo.png");
    if etc.exists() { return Some(etc); }
    None
}
