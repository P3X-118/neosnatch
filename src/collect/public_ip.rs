use crate::cache;
use crate::config::NetworkCfg;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct Cached {
    ip: String,
}

pub async fn fetch(cfg: &NetworkCfg, ttl_secs: u64) -> Option<String> {
    let ttl = Duration::from_secs(ttl_secs);
    if let Some(c) = cache::read::<Cached>("public_ip", ttl) {
        return Some(c.ip);
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(cfg.timeout_ms))
        .build()
        .ok()?;
    let resp = client.get(&cfg.public_ip_url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body = resp.text().await.ok()?;
    let ip = body.trim().to_string();
    if ip.is_empty() {
        return None;
    }
    let _ = cache::write("public_ip", &Cached { ip: ip.clone() });
    Some(ip)
}
