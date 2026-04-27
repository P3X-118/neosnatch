use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("neosnatch"))
}

pub fn read<T: DeserializeOwned>(key: &str, ttl: Duration) -> Option<T> {
    let dir = cache_dir()?;
    let path = dir.join(format!("{key}.json"));
    let meta = std::fs::metadata(&path).ok()?;
    let mtime = meta.modified().ok()?;
    if SystemTime::now().duration_since(mtime).ok()? > ttl {
        return None;
    }
    let raw = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn write<T: Serialize>(key: &str, value: &T) -> Result<()> {
    let Some(dir) = cache_dir() else { return Ok(()); };
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{key}.json"));
    let raw = serde_json::to_string(value)?;
    std::fs::write(path, raw)?;
    Ok(())
}
